//! 範囲を索引の目印で分け、裏のいくつかのスレッドで並列に畳み込む仕事（ADR 0030 の決定 4・QLT-008）。

use std::sync::Arc;

use nene_ledger_application::wakeup::Wakeup;

use crate::indexed_source::IndexedSource;
use crate::progress::Progress;
use crate::range_job::RangeJob;
use crate::range_outcome::RangeOutcome;
use crate::record_range::RecordRange;
use crate::scan_failure::ScanFailure;
use crate::sparse_index::MARK_INTERVAL;
use crate::split_fold::SplitFold;

/// 1 つの部分の、少なくともこれだけのレコード（これより小さい範囲は分けない）。
const MIN_PART_RECORDS: u64 = 64 * MARK_INTERVAL;

/// 並列に畳み込むスレッドの数の上限。
const MOST_PARTS: usize = 8;

/// この機械で並列に畳み込むのによい部分の数（論理の CPU の数・上限 8・分からなければ 1）。
#[must_use]
pub fn usual_parts() -> usize {
    std::thread::available_parallelism()
        .map_or(1, core::num::NonZeroUsize::get)
        .min(MOST_PARTS)
}

/// 1 つの部分の結果。
type Outcome<F> = Result<RangeOutcome<F>, ScanFailure>;

/// 範囲を索引の目印の境で前から順に分け、部分ごとに `RangeJob` で畳み込む仕事。結果は前の部分から順に
/// `SplitFold::merged` で合わせる。前の部分の畳み込みが「もう要らない」（`RecordFold::is_done`）と言えば、そこで
/// 合わせ終えて後ろの部分はやめる。手放すと全部の部分を中止して合流する（`RangeJob` と同じ）。
#[derive(Debug)]
pub struct SplitJob<F: SplitFold> {
    parts: Vec<RangeJob<F>>,
    results: Vec<Option<Outcome<F>>>,
}

impl<F: SplitFold> SplitJob<F> {
    /// `from` の `range` を、多くても `parts` 個に分けて `fold` の写しで畳み込み始める。索引が無い・範囲が小さければ
    /// 1 つのまま。
    ///
    /// # Errors
    ///
    /// 原本を読む口を作れない・作業のスレッドを作れないとき。
    pub fn spawn(
        from: &IndexedSource,
        range: RecordRange,
        (fold, parts): (F, usize),
        wakeup: &Arc<dyn Wakeup>,
    ) -> Result<Self, ScanFailure> {
        let ranges = split(from, range, parts);
        let jobs = ranges
            .into_iter()
            .map(|part| {
                let source = IndexedSource::new(
                    from.source().try_clone()?,
                    from.index().clone(),
                    from.format(),
                );
                RangeJob::spawn(source, part, fold.clone(), Arc::clone(wakeup))
            })
            .collect::<Result<Vec<_>, ScanFailure>>()?;
        let results = jobs.iter().map(|_| None).collect();
        Ok(Self {
            parts: jobs,
            results,
        })
    }

    /// 分けた部分の数。
    #[must_use]
    pub fn part_count(&self) -> usize {
        self.parts.len()
    }

    /// 全部の部分を合わせた進み具合（畳み込んだレコードの数 / 範囲のレコードの数）。
    #[must_use]
    pub fn progress(&self) -> Progress {
        let (done, total) = self.parts.iter().map(RangeJob::progress).fold(
            (0_u64, 0_u64),
            |(done, total), progress| {
                (
                    done.saturating_add(progress.done()),
                    total.saturating_add(progress.total()),
                )
            },
        );
        Progress::new(done, total, None)
    }

    /// 全部の部分に中止を求める。
    pub fn cancel(&self) {
        for part in &self.parts {
            part.cancel();
        }
    }

    /// 合わせた結果が決まっていれば受け取る（1 度だけ・残りの部分はやめる）。まだなら `None`。待たない。
    pub fn take_result(&mut self) -> Option<Outcome<F>> {
        for (part, slot) in self.parts.iter_mut().zip(&mut self.results) {
            if slot.is_none() {
                *slot = part.take_result();
            }
        }
        if !self.decided() {
            return None;
        }
        self.cancel();
        merge_front(&mut self.results)
    }

    /// 前から見て、結果が決まったか（全部が終わった・前の部分が見つけ終えた・止まった・失敗した）。
    fn decided(&self) -> bool {
        for slot in &self.results {
            match slot {
                None => return false,
                Some(Ok(RangeOutcome::Finished(fold))) if fold.is_done() => return true,
                Some(Ok(RangeOutcome::Finished(_))) => {}
                Some(Ok(RangeOutcome::Cancelled) | Err(_)) => return true,
            }
        }
        true
    }
}

/// 前の部分から順に結果を合わせる。見つけ終えた部分で止め、止まった・失敗した部分があればそれを返す。
fn merge_front<F: SplitFold>(results: &mut [Option<Outcome<F>>]) -> Option<Outcome<F>> {
    let mut merged: Option<F> = None;
    for slot in results {
        let fold = match slot.take()? {
            Ok(RangeOutcome::Finished(fold)) => fold,
            Ok(RangeOutcome::Cancelled) => return Some(Ok(RangeOutcome::Cancelled)),
            Err(failure) => return Some(Err(failure)),
        };
        let done = fold.is_done();
        merged = Some(joined(merged, fold));
        if done {
            break;
        }
    }
    merged.map(|fold| Ok(RangeOutcome::Finished(fold)))
}

/// 前までに合わせた結果 `earlier`（無ければ `fold` そのもの）に、次の部分の結果 `fold` を合わせる。
fn joined<F: SplitFold>(earlier: Option<F>, fold: F) -> F {
    match earlier {
        Some(earlier) => earlier.merged(fold),
        None => fold,
    }
}

/// `range` を、索引の目印の境（`MARK_INTERVAL` の倍数）で前から順に多くても `parts` 個に分ける。最後の部分は範囲の
/// 終わりまで（索引の知る終わりより先も読む）。索引が無ければ分けない。
fn split(from: &IndexedSource, range: RecordRange, parts: usize) -> Vec<RecordRange> {
    let first = range.first();
    let wanted = u64::try_from(range.count()).unwrap_or(u64::MAX);
    let known = from.index().records().saturating_sub(first).min(wanted);
    let most = u64::try_from(parts.max(1)).unwrap_or(1);
    let count = (known / MIN_PART_RECORDS).clamp(1, most);
    if from.index().marks().is_empty() || count <= 1 {
        return vec![range];
    }
    let step = known / count;
    let mut starts: Vec<u64> = (1..count)
        .map(|part| {
            let at = first.saturating_add(step.saturating_mul(part));
            at - at % MARK_INTERVAL
        })
        .filter(|&at| at > first)
        .collect();
    starts.dedup();
    let end = first.saturating_add(wanted);
    let mut ranges = Vec::new();
    let mut start = first;
    for next in starts {
        ranges.push(part_range(start, next));
        start = next;
    }
    ranges.push(part_range(start, end));
    ranges
}

/// レコード `start` から `end` の手前までの範囲。
fn part_range(start: u64, end: u64) -> RecordRange {
    RecordRange::new(
        start,
        usize::try_from(end.saturating_sub(start)).unwrap_or(usize::MAX),
    )
}
