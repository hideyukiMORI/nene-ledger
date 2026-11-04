//! 原本を先頭から塊ごとに読んで、まばらな索引を作る。

use core::ops::ControlFlow;

use nene_ledger_core::text_format::TextFormat;

use crate::block_size::BlockSize;
use crate::index_builder::IndexBuilder;
use crate::index_outcome::IndexOutcome;
use crate::progress::Progress;
use crate::scan_failure::ScanFailure;
use crate::source::Source;

/// 原本を先頭から塊ごとに読んで、まばらな索引を作る（ADR 0014・ADR 0017）。
///
/// 塊を 1 つ読むたびに `report` に進み具合（と、長すぎる引用のセルの知らせ）を渡し、`Break` が返れば止まって
/// 途中の結果を捨てる。裏のスレッドで走らせるときは `report` に `JobControl::report` を渡す。
/// 原本が `format` の文字コードの BOM で始まっていれば、BOM の後ろから切る。
/// 方言が `UnclosedQuote::Literal` なら、入力の終わりで閉じなかった引用符の次から読み直す（ADR 0018 の決定 4）。
///
/// # Errors
///
/// 読み出しの失敗。
pub fn build_index<F>(
    source: &Source,
    format: TextFormat,
    size: BlockSize,
    mut report: F,
) -> Result<IndexOutcome, ScanFailure>
where
    F: FnMut(Progress) -> ControlFlow<()>,
{
    let start = source.content_start(format.encoding())?;
    let mut builder = IndexBuilder::new(format.dialect(), start);
    loop {
        let block = source.read_block(builder.position(), size)?;
        if block.is_empty() {
            if builder.reread_literally().is_some() {
                continue;
            }
            return Ok(IndexOutcome::Finished(builder.finish()));
        }
        builder.feed(&block)?;
        let progress = Progress::new(
            builder.position(),
            source.len(),
            builder.long_quoted_cell()?,
        )
        .with_records(builder.records());
        if report(progress).is_break() {
            return Ok(IndexOutcome::Cancelled);
        }
    }
}
