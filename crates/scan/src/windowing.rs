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
    let records = spans
        .into_iter()
        .filter_map(|span| {
            read_one(source, cutting_dialect(index, dialect, span), span).transpose()
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(RecordWindow::new(records))
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
    loop {
        let block = source.read_block(starts.position(), BlockSize::MEBIBYTE)?;
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
