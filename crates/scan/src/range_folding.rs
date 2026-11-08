//! 範囲のレコードを、塊ごとに読み進めて畳み込む。

use core::ops::ControlFlow;

use crate::indexed_source::IndexedSource;
use crate::mark::Mark;
use crate::record_fold::RecordFold;
use crate::record_length::RecordLength;
use crate::record_range::RecordRange;
use crate::scan_failure::ScanFailure;
use crate::windowing::{nearest_mark, visit_records_from};

/// 塊 `chunks`（先頭から続く範囲）のレコードを順に読み、1 つずつ `fold` に渡す。`fold` がもう要らないと言えば
/// （`RecordFold::is_done`）そこで終える。塊を終えるたびに、渡した数を `report` に知らせ、`Break` なら止める。ファイルが範囲より先に終われば、そこで終わる。上限を超えるレコードが
/// あっても、索引が後ろを数えていれば窓は短くならない（GitHub Issue #13）ので、短い窓は索引の知る終わり。
///
/// 次の塊は前の塊の最後のレコードの続きから切り進める（塊ごとに目印から切り直さない）。ただし索引の目印の方が
/// 近ければそちらから。最後のレコードが読み出しの上限を超えて長さが分からなければ、目印から切り直す。
///
/// # Errors
///
/// 読み出しの失敗。
pub fn fold_range<F, C, R>(
    from: &IndexedSource,
    chunks: C,
    fold: &mut F,
    mut report: R,
) -> Result<ControlFlow<()>, ScanFailure>
where
    F: RecordFold,
    C: Iterator<Item = RecordRange>,
    R: FnMut(u64) -> ControlFlow<()>,
{
    let mut carried: Option<Mark> = None;
    let mut done = 0_u64;
    for chunk in chunks {
        let indexed = nearest_mark(from.source(), from.index(), from.format(), chunk.first())?;
        let mark = carried
            .filter(|mark| mark.record() <= chunk.first() && mark.record() >= indexed.record())
            .unwrap_or(indexed);
        // 1 レコードずつ渡して手放す（塊の全部を持たない・ADR 0033 の決定 2・GitHub Issue #22）。
        let mut read = 0_u64;
        let mut tail: Option<(u64, RecordLength)> = None;
        let mut finished = false;
        let mut number = chunk.first();
        visit_records_from(
            from.source(),
            (from.index(), from.format()),
            (mark, chunk),
            |record| {
                fold.record(number, &record);
                number = number.saturating_add(1);
                read = read.saturating_add(1);
                tail = Some((record.offset(), record.length()));
                if fold.is_done() {
                    finished = true;
                    return ControlFlow::Break(());
                }
                ControlFlow::Continue(())
            },
        )?;
        if finished {
            return Ok(ControlFlow::Continue(()));
        }
        if read == 0 {
            break;
        }
        done = done.saturating_add(read);
        carried = tail.and_then(|(offset, length)| match length {
            RecordLength::Exact(length) => Some(Mark::new(
                chunk.first().saturating_add(read),
                offset.saturating_add(length),
            )),
            RecordLength::MoreThan(_) => None,
        });
        if report(done).is_break() {
            return Ok(ControlFlow::Break(()));
        }
        if read < u64::try_from(chunk.count())? {
            break;
        }
    }
    Ok(ControlFlow::Continue(()))
}
