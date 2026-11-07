//! 見えている 1 行。

use alloc::string::String;

use nene_ledger_core::row_id::RowId;

use crate::cell_display::{is_single_line, single_line};
use crate::cell_row::CellRow;

/// 見えている 1 行。行の並びの位置（0 始まり・前書きと見出しも数える・ADR 0027 の決定 1）と、行の同一性と、
/// 元の値（復号したまま）と、表示の文字列（1 行）にしたセル。
///
/// 表示の文字列は、元の値に印に替える字（改行・タブ・見えない字）があるときだけ別に持つ。セルは 1 本の文字列と区切りの
/// 位置で持つ（数万列の表で窓の記憶がセルの数に比例して GB 単位になったため・GitHub Issue #22）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewRow {
    line: u64,
    row: RowId,
    values: CellRow,
    shown: Option<CellRow>,
}

impl ViewRow {
    /// 行を作る。`cells` は復号したセルの値で、元の値と、表示の文字列（1 行）にしたものの両方を引ける（詳細ペイン・VIEW-07）。
    /// 行の同一性は、行を足し消ししていないときの原本のレコード `Source(line)`（[`Self::with_row`] で替える）。
    #[must_use]
    pub fn new<S: AsRef<str>>(line: u64, cells: &[S]) -> Self {
        Self::from_values(line, CellRow::new(cells))
    }

    /// 元の値 `values` から作る。
    #[must_use]
    pub fn from_values(line: u64, values: CellRow) -> Self {
        let shown = (!values.iter().all(is_single_line)).then(|| {
            let lines: alloc::vec::Vec<String> = values.iter().map(single_line).collect();
            CellRow::new(&lines)
        });
        Self {
            line,
            row: RowId::Source(line),
            values,
            shown,
        }
    }

    /// 行の同一性を `row` にした行。
    #[must_use]
    pub fn with_row(self, row: RowId) -> Self {
        Self { row, ..self }
    }

    /// 行の並びの位置（0 始まり）。行を足し消ししていなければ、ファイルの中のレコードの番号と同じ。
    #[must_use]
    pub fn line(&self) -> u64 {
        self.line
    }

    /// 行の同一性（書き換えの差分を引く鍵・ADR 0024 の決定 1）。
    #[must_use]
    pub fn row(&self) -> RowId {
        self.row
    }

    /// 表示の文字列。
    #[must_use]
    pub fn cells(&self) -> &CellRow {
        self.shown.as_ref().unwrap_or(&self.values)
    }

    /// 元の値（復号したまま）。
    #[must_use]
    pub fn values(&self) -> &CellRow {
        &self.values
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 印に替える字が無ければ表示の文字列は元の値そのもので、写しを持たない。あれば表示の文字列を別に持つ。
    #[test]
    fn the_shown_text_is_kept_apart_only_when_it_differs() {
        let plain = ViewRow::new(3, &["a", "東京", ""]);
        assert!(plain.shown.is_none());
        assert_eq!(plain.cells().get(1), Some("東京"));
        let marked = ViewRow::new(3, &["a", "東京\r\n都"]);
        assert_eq!(marked.cells().get(1), Some("東京¶都"));
        assert_eq!(marked.values().get(1), Some("東京\r\n都"));
        assert_eq!(marked.cells().get(0), Some("a"));
    }
}
