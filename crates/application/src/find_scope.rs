//! 何を探すか（ADR 0030 の決定 2）：見えている値で、行の中の列を左から右へ照らす。

use alloc::string::String;
use alloc::vec::Vec;
use core::ops::Range;

use nene_ledger_core::cell_change::CellChange;
use nene_ledger_core::cell_edits::CellEdits;
use nene_ledger_core::cell_value::CellValue;
use nene_ledger_core::column_id::ColumnId;
use nene_ledger_core::dialect::Dialect;
use nene_ledger_core::find_query::FindQuery;
use nene_ledger_core::row_id::RowId;

/// 探す問いと、表示の順の列（同一性）と、書き換えの写し。裏の検索のスレッドにも渡す（原本の行は scan が読む）。
/// 書き換えたセルは書き換えた値で照らし、元の値では当たらない。「セルが無い」にしたセルと、足した列の書き換えていない
/// セル（空）は当たらない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindScope {
    query: FindQuery,
    columns: Vec<ColumnId>,
    edits: CellEdits,
}

impl FindScope {
    /// 問い `query` を、表示の順の列 `columns` と書き換え `edits` で照らす。
    #[must_use]
    pub fn new(query: FindQuery, columns: Vec<ColumnId>, edits: CellEdits) -> Self {
        Self {
            query,
            columns,
            edits,
        }
    }

    /// 問い。
    #[must_use]
    pub const fn query(&self) -> &FindQuery {
        &self.query
    }

    /// 表示の列の数。
    #[must_use]
    pub fn width(&self) -> usize {
        self.columns.len()
    }

    /// 原本の行 `row` を、方言 `dialect` で書いた ASCII だけのレコードのバイト列 `raw` から照らさずに除けるか（書き換えた
    /// 行は除かない）。
    #[must_use]
    pub fn skips_ascii_record(&self, row: RowId, raw: &[u8], dialect: &Dialect) -> bool {
        !self.edits.touches_row(row) && raw.is_ascii() && self.query.rules_out_ascii(raw, dialect)
    }

    /// 行 `row` の表示の列 `columns` のうち、左から見て最初に当たる列。`source` は原本の `n` 列目の値（無ければ `None`）。
    pub fn first_in_row<S: Fn(usize) -> Option<String>>(
        &self,
        row: RowId,
        columns: Range<usize>,
        source: S,
    ) -> Option<usize> {
        columns
            .into_iter()
            .find(|&display| self.hits(row, display, &source))
    }

    /// 行 `row` の表示の列 `columns` のうち、右から見て最初に当たる列（前へ探すとき）。
    pub fn last_in_row<S: Fn(usize) -> Option<String>>(
        &self,
        row: RowId,
        columns: Range<usize>,
        source: S,
    ) -> Option<usize> {
        columns
            .into_iter()
            .rev()
            .find(|&display| self.hits(row, display, &source))
    }

    /// 行 `row` の表示の `display` 列目のセルの当たった所を `replacement` に置き換える書き換えと、そのセルのファイルの今の値
    /// （置換・ADR 0030 の決定 6）。`source` は原本の `n` 列目の値。当たらないか、置き換えても見えている値が変わらなければ
    /// `None`。照らすセルは `first_in_row` と同じ（書き換えた値・「セルが無い」と足した列の書き換えていないセルは当たらない）。
    pub fn replaced_cell<S: Fn(usize) -> Option<String>>(
        &self,
        (row, display): (RowId, usize),
        source: &S,
        replacement: &str,
    ) -> Option<(CellChange, String)> {
        let &column = self.columns.get(display)?;
        let file = match column {
            ColumnId::Source(index) => source(index),
            ColumnId::Added(_) => None,
        };
        let shown = match self.edits.get(row, column) {
            Some(CellValue::Text(value)) => value.clone(),
            Some(CellValue::Absent) => return None,
            None => match column {
                ColumnId::Source(_) => file.clone()?,
                ColumnId::Added(_) => return None,
            },
        };
        let after = self.query.replaced(&shown, replacement)?;
        (after != shown).then(|| {
            (
                CellChange::new(row, column, shown, after),
                file.unwrap_or_default(),
            )
        })
    }

    /// 行 `row` の表示の `display` 列目のセルが当たるか。
    fn hits<S: Fn(usize) -> Option<String>>(&self, row: RowId, display: usize, source: &S) -> bool {
        let Some(&column) = self.columns.get(display) else {
            return false;
        };
        match self.edits.get(row, column) {
            Some(CellValue::Text(value)) => self.query.matches(value),
            Some(CellValue::Absent) => false,
            None => match column {
                ColumnId::Source(index) => {
                    source(index).is_some_and(|value| self.query.matches(&value))
                }
                ColumnId::Added(_) => false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use nene_ledger_core::find_options::FindOptions;

    fn scope(text: &str, edits: CellEdits) -> FindScope {
        FindScope::new(
            FindQuery::new(text, FindOptions::standard()),
            vec![ColumnId::Source(1), ColumnId::Source(0), ColumnId::Added(0)],
            edits,
        )
    }

    fn cells(index: usize) -> Option<String> {
        ["tokyo", "osaka"]
            .get(index)
            .map(|&value| String::from(value))
    }

    #[test]
    fn rows_are_searched_in_the_shown_order_with_edits_over_the_source() {
        let row = RowId::Source(4);
        // 表示の順は osaka・tokyo・足した列。
        assert_eq!(
            scope("o", CellEdits::empty()).first_in_row(row, 0..3, cells),
            Some(0)
        );
        assert_eq!(
            scope("o", CellEdits::empty()).last_in_row(row, 0..3, cells),
            Some(1)
        );
        assert_eq!(
            scope("tok", CellEdits::empty()).first_in_row(row, 0..1, cells),
            None
        );
        let mut edits = CellEdits::empty();
        let _kyoto = edits.set(
            row,
            ColumnId::Source(0),
            CellValue::Text(String::from("kyoto")),
        );
        let _tokyo = edits.set(
            row,
            ColumnId::Added(0),
            CellValue::Text(String::from("Tokyo")),
        );
        // 書き換えた値で当たり、元の値（tokyo）では当たらない。
        let edited = scope("tok", edits.clone());
        assert_eq!(edited.first_in_row(row, 0..3, cells), Some(2));
        assert_eq!(
            scope("kyoto", edits).first_in_row(row, 0..3, cells),
            Some(1)
        );
    }

    /// 置き換えは当たるセルと同じセルに、見えている値（書き換えた値）を元に書く。ファイルの値も添える。
    #[test]
    fn replacing_writes_over_the_shown_value_of_matching_cells() {
        let row = RowId::Source(4);
        let plain = scope("o", CellEdits::empty());
        let (change, file) = plain.replaced_cell((row, 0), &cells, "0").unwrap();
        assert_eq!(
            change,
            CellChange::new(
                row,
                ColumnId::Source(1),
                String::from("osaka"),
                String::from("0saka")
            )
        );
        assert_eq!(file, "osaka");
        // 足した列の書き換えていないセルと、置き換えても変わらないセルは書かない。
        assert_eq!(plain.replaced_cell((row, 2), &cells, "0"), None);
        assert_eq!(plain.replaced_cell((row, 0), &cells, "o"), None);
        let mut edits = CellEdits::empty();
        let _kyoto = edits.set(
            row,
            ColumnId::Source(0),
            CellValue::Text(String::from("kyoto")),
        );
        let (edited, original) = scope("kyo", edits)
            .replaced_cell((row, 1), &cells, "X")
            .unwrap();
        assert_eq!(edited.after(), "Xto");
        assert_eq!(original, "tokyo");
    }
}
