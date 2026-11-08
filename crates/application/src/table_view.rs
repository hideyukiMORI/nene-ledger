//! 開いたファイルの、スクロールする本体の見えている行。

use alloc::vec::Vec;

use nene_ledger_core::header_rows::HeaderRows;

use crate::body_mapping::body_of;
use crate::cell_row::CellRow;
use crate::view_row::ViewRow;

/// 開いたファイルの、スクロールする本体の見えている行と、最初の行が本体の何番目か（見出しの行は含まない・VIEW-01）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableView {
    first_body: u64,
    rows: Vec<ViewRow>,
}

impl TableView {
    /// 行の並びの位置 `first_record` から読んだ行 `rows`（復号したセルの値）から、見出しの行 `header` を除いて作る。
    /// 行の同一性は原本のレコード（行を足し消ししていないとき）。
    #[must_use]
    pub fn from_records(first_record: u64, rows: Vec<CellRow>, header: Option<HeaderRows>) -> Self {
        let rows = rows
            .into_iter()
            .zip(first_record..)
            .map(|(cells, line)| ViewRow::from_values(line, cells))
            .collect();
        Self::from_rows(first_record, rows, header)
    }

    /// 位置 `first_line` からの行 `rows` から、見出しの行 `header` を除いて作る。
    #[must_use]
    pub fn from_rows(first_line: u64, rows: Vec<ViewRow>, header: Option<HeaderRows>) -> Self {
        let rows = rows
            .into_iter()
            .filter(|row| !header.is_some_and(|header| header.contains(row.line())))
            .collect();
        Self {
            first_body: body_of(first_line, header),
            rows,
        }
    }

    /// 最初の行が本体の何番目か（0 始まり）。
    #[must_use]
    pub fn first_body(&self) -> u64 {
        self.first_body
    }

    /// 見えている行。
    #[must_use]
    pub fn rows(&self) -> &[ViewRow] {
        &self.rows
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    fn rows(count: u32) -> Vec<CellRow> {
        (0..count)
            .map(|n| CellRow::new(&[alloc::format!("{n}")]))
            .collect()
    }

    fn records(view: &TableView) -> Vec<u64> {
        view.rows().iter().map(ViewRow::line).collect()
    }

    #[test]
    fn rows_keep_their_record_numbers() {
        let view = TableView::from_records(41, rows(3).clone(), None);
        assert_eq!(view.first_body(), 41);
        assert_eq!(records(&view), vec![41, 42, 43]);
        assert_eq!(view.rows()[0].cells().to_strings(), ["0"]);
    }

    #[test]
    fn the_header_record_is_left_out() {
        let from_top = TableView::from_records(0, rows(4).clone(), Some(HeaderRows::single(0)));
        assert_eq!(from_top.first_body(), 0);
        assert_eq!(records(&from_top), vec![1, 2, 3]);
        // 前書き 2 行の後ろに見出し（レコード 2）。
        let with_preamble =
            TableView::from_records(1, rows(4).clone(), Some(HeaderRows::single(2)));
        assert_eq!(with_preamble.first_body(), 1);
        assert_eq!(records(&with_preamble), vec![1, 3, 4]);
        let after = TableView::from_records(10, rows(2).clone(), Some(HeaderRows::single(2)));
        assert_eq!(after.first_body(), 9);
        assert_eq!(records(&after), vec![10, 11]);
        // 2 行の見出し（レコード 1・2）。
        let two = TableView::from_records(0, rows(5).clone(), HeaderRows::new(1, 2));
        assert_eq!(two.first_body(), 0);
        assert_eq!(records(&two), vec![0, 3, 4]);
    }

    #[test]
    fn cells_become_single_lines() {
        let multi = vec![CellRow::new(&["東京\n都"])];
        let view = TableView::from_records(0, multi.clone(), None);
        assert_eq!(view.rows()[0].cells().to_strings(), ["東京¶都"]);
        assert!(
            TableView::from_records(7, Vec::new(), None)
                .rows()
                .is_empty()
        );
    }
}
