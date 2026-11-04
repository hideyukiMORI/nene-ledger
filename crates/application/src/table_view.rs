//! 開いたファイルの、スクロールする本体の見えている行。

use alloc::string::String;
use alloc::vec::Vec;

use crate::body_mapping::body_of;
use crate::view_row::ViewRow;

/// 開いたファイルの、スクロールする本体の見えている行と、最初の行が本体の何番目か（見出しの行は含まない・VIEW-01）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableView {
    first_body: u64,
    rows: Vec<ViewRow>,
}

impl TableView {
    /// レコード `first_record` から読んだ行 `rows`（復号したセルの値）から、見出しのレコード `header` を除いて作る。
    #[must_use]
    pub fn from_records(first_record: u64, rows: &[Vec<String>], header: Option<u64>) -> Self {
        let rows = rows
            .iter()
            .zip(first_record..)
            .filter(|&(_, record)| Some(record) != header)
            .map(|(cells, record)| ViewRow::new(record, cells))
            .collect();
        Self {
            first_body: body_of(first_record, header),
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

    fn rows(count: u32) -> Vec<Vec<String>> {
        (0..count).map(|n| vec![alloc::format!("{n}")]).collect()
    }

    fn records(view: &TableView) -> Vec<u64> {
        view.rows().iter().map(ViewRow::record).collect()
    }

    #[test]
    fn rows_keep_their_record_numbers() {
        let view = TableView::from_records(41, &rows(3), None);
        assert_eq!(view.first_body(), 41);
        assert_eq!(records(&view), vec![41, 42, 43]);
        assert_eq!(view.rows()[0].cells(), &[String::from("0")]);
    }

    #[test]
    fn the_header_record_is_left_out() {
        let from_top = TableView::from_records(0, &rows(4), Some(0));
        assert_eq!(from_top.first_body(), 0);
        assert_eq!(records(&from_top), vec![1, 2, 3]);
        // 前書き 2 行の後ろに見出し（レコード 2）。
        let with_preamble = TableView::from_records(1, &rows(4), Some(2));
        assert_eq!(with_preamble.first_body(), 1);
        assert_eq!(records(&with_preamble), vec![1, 3, 4]);
        let after = TableView::from_records(10, &rows(2), Some(2));
        assert_eq!(after.first_body(), 9);
        assert_eq!(records(&after), vec![10, 11]);
    }

    #[test]
    fn cells_become_single_lines() {
        let multi = vec![vec![String::from("東京\n都")]];
        let view = TableView::from_records(0, &multi, None);
        assert_eq!(view.rows()[0].cells(), &[String::from("東京¶都")]);
        assert!(TableView::from_records(7, &[], None).rows().is_empty());
    }
}
