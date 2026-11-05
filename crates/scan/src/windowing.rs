//! ある範囲のレコードを、近い目印から切って返す。

use nene_ledger_core::dialect::Dialect;
use nene_ledger_core::records::Records;
use nene_ledger_core::text_format::TextFormat;
use nene_ledger_core::unclosed_quote::UnclosedQuote;

use crate::block_size::BlockSize;
use crate::mark::Mark;
use crate::record_length::RecordLength;
use crate::record_range::RecordRange;
use crate::record_span::RecordSpan;
use crate::record_starts::RecordStarts;
use crate::record_window::RecordWindow;
use crate::scan_failure::ScanFailure;
use crate::source::Source;
use crate::sparse_index::SparseIndex;
use crate::window_record::{RECORD_READ_LIMIT, WindowRecord};

/// `range` のレコードを、その前で最も近い目印から切って返す（ADR 0017 の決定 2）。
///
/// まず状態機械でレコードの境目（バイト位置）だけを探し、次に 1 レコードにつき `RECORD_READ_LIMIT` までを読む。
/// 記憶は求めた件数 × 上限で抑えられ、ファイルやレコードの大きさに比例しない（BIG-04）。
/// 作り終える前の索引でもよい。目印より先は、最後の目印から切り進める（BIG-02）。目印が無ければ BOM の後ろから。
///
/// # Errors
///
/// 読み出しの失敗。
pub fn read_records(
    source: &Source,
    index: &SparseIndex,
    format: TextFormat,
    range: RecordRange,
) -> Result<RecordWindow, ScanFailure> {
    let mark = nearest_mark(source, index, format, range.first())?;
    read_records_from(source, (index, format), mark, range)
}

/// レコード `record` の前で最も近い目印。目印が無ければ BOM の後ろ。
///
/// # Errors
///
/// 読み出しの失敗。
pub(crate) fn nearest_mark(
    source: &Source,
    index: &SparseIndex,
    format: TextFormat,
    record: u64,
) -> Result<Mark, ScanFailure> {
    match index.mark_at_or_before(record) {
        Some(mark) => Ok(mark),
        None => Ok(Mark::new(0, source.content_start(format.encoding())?)),
    }
}

/// `range` のレコードを、目印 `mark`（`range` より前）から切って返す。`index` は、上限を超えたレコードの
/// 後ろを探すかと、`Literal` で読み直す引用符の位置に使う（目印は引かない）。
///
/// # Errors
///
/// 読み出しの失敗。
pub(crate) fn read_records_from(
    source: &Source,
    (index, format): (&SparseIndex, TextFormat),
    mark: Mark,
    range: RecordRange,
) -> Result<RecordWindow, ScanFailure> {
    let dialect = format.dialect();
    let spans = record_spans(source, (index, dialect), mark, range)?;
    let limit = u64::try_from(RECORD_READ_LIMIT.get())?;
    let mut records = Vec::with_capacity(spans.len());
    for batch in batches(&spans, limit) {
        match batch {
            [span] if !is_short(*span, limit) => {
                records.extend(read_one(
                    source,
                    cutting_dialect(index, dialect, *span),
                    *span,
                )?);
            }
            _ => records.extend(read_batch(source, dialect, batch)?),
        }
    }
    Ok(RecordWindow::new(records))
}

/// 上限までの長さが分かっているレコードか（まとめて読める）。
fn is_short(span: RecordSpan, limit: u64) -> bool {
    match span.length() {
        RecordLength::Exact(length) => length <= limit,
        RecordLength::MoreThan(_) => false,
    }
}

/// レコードの列を、まとめて 1 回で読む組に分ける。組は、上限以下の長さのレコードが続く所（合わせて上限まで）か、
/// 上限を超えたレコード 1 つ。🔴 レコードごとに原本を読むと、短い行の `--dump` の時間の大半が `pread` になる
/// （GitHub Issue #30）。並びは続いている（次のレコードは前の終わりから始まる・`spans`）。
fn batches(spans: &[RecordSpan], limit: u64) -> Vec<&[RecordSpan]> {
    let mut groups = Vec::new();
    let mut rest = spans;
    while let Some((group, after)) = rest.split_at_checked(batch_length(rest, limit)) {
        if group.is_empty() {
            break;
        }
        groups.push(group);
        rest = after;
    }
    groups
}

/// 先頭から 1 つの組に入るレコードの数。先頭が上限を超えるなら 1。空なら 0。
fn batch_length(spans: &[RecordSpan], limit: u64) -> usize {
    let mut total = 0_u64;
    let short = spans
        .iter()
        .take_while(|span| match span.length() {
            RecordLength::Exact(length) if length <= limit => {
                total = total.saturating_add(length);
                total <= limit
            }
            RecordLength::Exact(_) | RecordLength::MoreThan(_) => false,
        })
        .count();
    short.max(spans.len().min(1))
}

/// 上限以下の長さのレコードが続く組を、1 回で読んで 1 つずつ切る。切り方は `read_one` と同じ。
fn read_batch(
    source: &Source,
    dialect: Dialect,
    batch: &[RecordSpan],
) -> Result<Vec<WindowRecord>, ScanFailure> {
    let lengths = batch
        .iter()
        .map(|span| match span.length() {
            RecordLength::Exact(length) | RecordLength::MoreThan(length) => usize::try_from(length),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let (Some(first), Some(size)) = (
        batch.first(),
        BlockSize::new(
            lengths
                .iter()
                .fold(0_usize, |sum, &length| sum.saturating_add(length)),
        ),
    ) else {
        return Ok(Vec::new());
    };
    let bytes = source.read_block(first.start(), size)?;
    let mut records = Vec::with_capacity(batch.len());
    let mut at = 0_usize;
    for (span, length) in batch.iter().zip(lengths) {
        let end = at.saturating_add(length);
        let piece = bytes.get(at..end.min(bytes.len())).unwrap_or_default();
        if let Some(record) = Records::new(piece, dialect).next() {
            records.push(WindowRecord::new(
                span.start(),
                span.length(),
                piece.to_vec(),
                record,
            ));
        }
        at = end;
    }
    Ok(records)
}

/// `mark` から切り進めて、`range` の各レコードの始まりと長さ。範囲がファイルの外なら空。
///
/// 終わりは次のレコードの始まりか、ファイルの終わり。範囲の中のレコードが `RECORD_READ_LIMIT` を超えても、
/// 索引がその後ろにレコードを数えていれば、終わりまで切り進めて後ろも返す（読むのは上限まで・GitHub Issue #13）。
/// 🔴 索引が後ろを知らない（作り終える前・最後のレコード）なら、その先は探さずに止める（そのレコードは
/// `MoreThan`）。閉じない引用符でファイルの残り全部が 1 つのレコードになっても、手間は上限で抑えられる
/// （BIG-01・公開前の Issue #23）。手間は索引が既に読んだ長さを超えない。
/// 方言が `UnclosedQuote::Literal` なら、索引が覚えている閉じなかった引用符はその場で、まだ覚えていなければ
/// ファイルの終わりで、引用符の次から読み直す（ADR 0018 の決定 4）。
fn record_spans(
    source: &Source,
    (index, dialect): (&SparseIndex, Dialect),
    mark: Mark,
    range: RecordRange,
) -> Result<Vec<RecordSpan>, ScanFailure> {
    if range.count() == 0 {
        return Ok(Vec::new());
    }
    let limit = u64::try_from(RECORD_READ_LIMIT.get())?;
    let skip = range.first().saturating_sub(mark.record());
    let last = skip.saturating_add(u64::try_from(range.count())?);
    let mut starts = RecordStarts::new(dialect, mark.offset());
    let mut wanted: Vec<u64> = Vec::new();
    let mut seen = 0_u64;
    let mut end = mark.offset();
    // 🔴 塊は 64 KiB から倍にしていく（1 MiB まで）。毎回 1 MiB を読むと、空行だけのファイルの `--dump` で
    // 窓ごとに 1 MiB を写す（GitHub Issue #30）。
    let mut size = BlockSize::SIXTY_FOUR_KIBIBYTES;
    loop {
        let block = source.read_block(starts.position(), size)?;
        size = size.doubled_within(BlockSize::MEBIBYTE);
        if block.is_empty() {
            end = end.max(starts.position());
            if starts.resume_literally().is_some() {
                continue;
            }
            // UTF-16 で奇数の長さの最後の 1 バイトがレコードの頭なら、それも 1 つのレコード（参照実装と同じ）。
            if let Some(start) = starts.finish_input()
                && seen >= skip
                && seen < last
            {
                wanted.push(start);
            }
            return Ok(spans(&wanted, RecordLength::Exact(end)));
        }
        // 🔴 始まりを 1 つずつ取り出し、要る数がそろったらそこで返す。塊の全部を先に切ると、窓ごとに 1 MiB を切る
        // 手間がかかる（空行だけの 100 MB の `--dump` で 575 s・GitHub Issue #23）。
        for start in block.iter().filter_map(|&byte| starts.step(byte)) {
            if seen == last {
                return Ok(spans(&wanted, RecordLength::Exact(start)));
            }
            if seen >= skip {
                wanted.push(start);
            }
            seen = seen.saturating_add(1);
        }
        if reread_known_literal(&mut starts, index) {
            continue;
        }
        let long = wanted
            .last()
            .is_some_and(|&start| starts.position().saturating_sub(start) > limit);
        // 今のレコードの番号は mark.record() + seen - 1。索引がその次を数えていなければ止める。
        if long && index.records() <= mark.record().saturating_add(seen) {
            return Ok(spans(&wanted, RecordLength::MoreThan(limit)));
        }
    }
}

/// 引用の中にいて、その引用符を索引が「入力の終わりまで閉じなかった」と覚えていれば、方言が `Literal` なら
/// 入力の終わりを待たずに引用符の次から読み直す（読み直したら `true`）。
///
/// 索引も同じ状態機械で切っており、閉じなかった引用符から先は入力の終わりまで引用の中にいてレコードの始まりが
/// 無いので、ここで読み直しても、終わりで読み直すのと同じレコードになる。`ToEnd` では何もしない。
fn reread_known_literal(starts: &mut RecordStarts, index: &SparseIndex) -> bool {
    let known = starts
        .unclosed_quote()
        .is_some_and(|quote| index.unclosed_quotes().binary_search(&quote).is_ok());
    known && starts.resume_literally().is_some()
}

/// 始まりの列から、レコードの位置と長さの列を作る。最後のレコードは `tail` で終わる
/// （`Exact` なら次の始まりかファイルの終わりの位置、`MoreThan` なら長さそのもの）。
fn spans(starts: &[u64], tail: RecordLength) -> Vec<RecordSpan> {
    let following = starts.iter().skip(1).map(|&next| RecordLength::Exact(next));
    starts
        .iter()
        .zip(following.chain(core::iter::once(tail)))
        .map(|(&start, end)| match end {
            RecordLength::Exact(end) => {
                RecordSpan::new(start, RecordLength::Exact(end.saturating_sub(start)))
            }
            RecordLength::MoreThan(length) => {
                RecordSpan::new(start, RecordLength::MoreThan(length))
            }
        })
        .collect()
}

/// 1 レコードを、上限までだけ読んで切る。
fn read_one(
    source: &Source,
    dialect: Dialect,
    span: RecordSpan,
) -> Result<Option<WindowRecord>, ScanFailure> {
    let limit = u64::try_from(RECORD_READ_LIMIT.get())?;
    let wanted = match span.length() {
        RecordLength::Exact(length) => length.min(limit),
        RecordLength::MoreThan(_) => limit,
    };
    let Some(size) = BlockSize::new(usize::try_from(wanted)?) else {
        return Ok(None);
    };
    let bytes = source.read_block(span.start(), size)?;
    let record = Records::new(&bytes, dialect).next();
    Ok(record.map(|record| WindowRecord::new(span.start(), span.length(), bytes, record)))
}

/// レコード `span` を切る方言。🔴 上限を超えて先頭だけを読むレコードは、閉じた引用のセルでも切った所に閉じる引用符が
/// 無いので、`Literal` のままだと引用符を文字とみなしてセルの中の区切りで割れる（GitHub Issue #34・210 万字のセルが
/// 349,526 のセルになった）。切り詰めるレコードは厳格（`ToEnd`）に切る。ただし、索引が入力の終わりまで閉じなかったと
/// 覚えている引用符がそのレコードの中にあれば、利用者の選んだ `Literal` のまま切る。
fn cutting_dialect(index: &SparseIndex, dialect: Dialect, span: RecordSpan) -> Dialect {
    let limit = u64::try_from(RECORD_READ_LIMIT.get()).unwrap_or(u64::MAX);
    let end = match span.length() {
        RecordLength::Exact(length) if length <= limit => return dialect,
        RecordLength::Exact(length) => span.start().saturating_add(length),
        RecordLength::MoreThan(_) => u64::MAX,
    };
    let quotes = index.unclosed_quotes();
    let after_start = quotes.partition_point(|&quote| quote < span.start());
    let unclosed_inside = quotes.get(after_start).is_some_and(|&quote| quote < end);
    match dialect.unclosed_quote() {
        UnclosedQuote::Literal if !unclosed_inside => {
            dialect.with_unclosed_quote(UnclosedQuote::ToEnd)
        }
        UnclosedQuote::Literal | UnclosedQuote::ToEnd => dialect,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exact(start: u64, length: u64) -> RecordSpan {
        RecordSpan::new(start, RecordLength::Exact(length))
    }

    fn group_sizes(spans: &[RecordSpan], limit: u64) -> Vec<usize> {
        batches(spans, limit)
            .iter()
            .map(|group| group.len())
            .collect()
    }

    #[test]
    fn short_records_are_grouped_up_to_the_limit() {
        let spans = [exact(0, 4), exact(4, 4), exact(8, 4), exact(12, 4)];
        assert_eq!(group_sizes(&spans, 8), vec![2, 2]);
        assert_eq!(group_sizes(&spans, 16), vec![4]);
        assert_eq!(group_sizes(&spans, 15), vec![3, 1]);
    }

    #[test]
    fn a_record_over_the_limit_stands_alone() {
        let spans = [
            exact(0, 2),
            exact(2, 20),
            RecordSpan::new(22, RecordLength::MoreThan(10)),
            exact(40, 2),
        ];
        assert_eq!(group_sizes(&spans, 10), vec![1, 1, 1, 1]);
        assert!(batches(&[], 10).is_empty());
    }
}
