//! 原本のレコードを裏の仕事で探す畳み込み（ADR 0030 の決定 2・4）。

use nene_ledger_application::find_scope::FindScope;
use nene_ledger_core::row_id::RowId;
use nene_ledger_core::row_order::RowOrder;
use nene_ledger_core::text_format::TextFormat;
use nene_ledger_core::value::value;
use nene_ledger_scan::decoding::decode;
use nene_ledger_scan::record_fold::RecordFold;
use nene_ledger_scan::window_record::WindowRecord;

use nene_ledger_application::find_bound::FindBound;

/// 原本のレコードを順に照らす。境（`FindBound`）より後ろを探すときは最初に当たった所で止まり、前を探すときは最後まで
/// 読んで最後に当たった所を残す。見出しのレコードと、
/// 並びから消したレコードは照らさない。足した行は application が探す（原本に無い）。
#[derive(Debug)]
pub(crate) struct FindFold {
    scope: FindScope,
    format: TextFormat,
    header: Option<u64>,
    rows: RowOrder,
    bound: FindBound,
    found: Option<(u64, usize)>,
}

impl FindFold {
    /// 問いと列と書き換え `scope` を、読み方 `format` で照らす。見出しのレコードは `header`、行の並びは `rows`。
    pub(crate) fn new(
        scope: FindScope,
        (format, header): (TextFormat, Option<u64>),
        rows: RowOrder,
        bound: FindBound,
    ) -> Self {
        Self {
            scope,
            format,
            header,
            rows,
            bound,
            found: None,
        }
    }

    /// 当たったレコードの番号と表示の列（無ければ `None`）。
    pub(crate) fn found(&self) -> Option<(u64, usize)> {
        self.found
    }
}

impl RecordFold for FindFold {
    fn record(&mut self, number: u64, record: &WindowRecord) {
        if Some(number) == self.header {
            return;
        }
        let row = RowId::Source(number);
        if self.format.encoding().keeps_ascii()
            && self.scope.skips_ascii_record(row, record.bytes())
        {
            return;
        }
        if !self.rows.is_whole() && self.rows.position_of(row).is_none() {
            return;
        }
        let width = self.scope.width();
        let dialect = self.format.dialect();
        let encoding = self.format.encoding();
        let fields = record.record().fields();
        let source = |index: usize| {
            fields.get(index).map(|&field| {
                let raw = value(record.bytes(), field, &dialect);
                String::from(decode(encoding, &raw).text())
            })
        };
        match self.bound {
            FindBound::After(after) => {
                let first = match after {
                    Some((at, column)) if at == number => column.saturating_add(1),
                    Some(_) | None => 0,
                };
                if let Some(column) = self.scope.first_in_row(row, first..width, source) {
                    self.found = Some((number, column));
                }
            }
            FindBound::Before(before) => {
                let end = match before {
                    Some((at, column)) if at == number => column,
                    Some(_) | None => width,
                };
                if let Some(column) = self.scope.last_in_row(row, 0..end, source) {
                    self.found = Some((number, column));
                }
            }
        }
    }

    fn is_done(&self) -> bool {
        match self.bound {
            FindBound::After(_) => self.found.is_some(),
            FindBound::Before(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::opening::guess_format;
    use core::ops::ControlFlow;
    use nene_ledger_core::cell_edits::CellEdits;
    use nene_ledger_core::cell_value::CellValue;
    use nene_ledger_core::column_id::ColumnId;
    use nene_ledger_core::find_options::FindOptions;
    use nene_ledger_core::find_query::FindQuery;
    use nene_ledger_scan::indexed_source::IndexedSource;
    use nene_ledger_scan::range_folding::fold_range;
    use nene_ledger_scan::record_range::RecordRange;
    use nene_ledger_scan::source::Source;
    use nene_ledger_scan::sparse_index::SparseIndex;
    use std::path::PathBuf;

    /// 見本 `shift-jis-pipe.csv`（`Shift_JIS`・`|` 区切り・見出しの `コード|品名|単価` ＋ 3 行）を探す。
    fn find(
        text: &str,
        (edits, rows): (CellEdits, RowOrder),
        bound: FindBound,
    ) -> Option<(u64, usize)> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join("shift-jis-pipe.csv");
        let source = Source::open(&path).unwrap();
        let format = guess_format(&source).unwrap().format();
        let columns = vec![
            ColumnId::Source(0),
            ColumnId::Source(1),
            ColumnId::Source(2),
        ];
        let scope = FindScope::new(
            FindQuery::new(text, FindOptions::standard()),
            columns,
            edits,
        );
        let from = IndexedSource::new(source, SparseIndex::empty(), format);
        // 塊の大きさに依らず同じ答え（QLT-008・ADR 0030 の強制）。
        let answers: Vec<Option<(u64, usize)>> = [1_usize, 2, 100]
            .iter()
            .map(|&size| {
                let mut fold = FindFold::new(scope.clone(), (format, Some(0)), rows.clone(), bound);
                let flow = fold_range(
                    &from,
                    RecordRange::new(0, 100).chunks(size),
                    &mut fold,
                    |_| ControlFlow::Continue(()),
                );
                assert_eq!(flow.unwrap(), ControlFlow::Continue(()));
                fold.found()
            })
            .collect();
        assert!(
            answers.windows(2).all(|pair| pair.first() == pair.last()),
            "{answers:?}"
        );
        answers.first().copied().flatten()
    }

    fn plain() -> (CellEdits, RowOrder) {
        (CellEdits::empty(), RowOrder::whole())
    }

    #[test]
    fn the_first_match_after_the_bound_is_found_in_shown_values() {
        // 見出しの「品名」は探さない。「表」は 1 行目の 2 列目と 2 行目の 2 列目にある。
        assert_eq!(find("品名", plain(), FindBound::After(None)), None);
        assert_eq!(find("表", plain(), FindBound::After(None)), Some((1, 1)));
        assert_eq!(
            find("表", plain(), FindBound::After(Some((1, 1)))),
            Some((2, 1))
        );
        // 全角半角を区別しない（原本は全角の「ソース」・問いは半角）。
        assert_eq!(find("ｿｰｽ", plain(), FindBound::After(None)), Some((3, 1)));
        assert_eq!(find("表", plain(), FindBound::Before(None)), Some((2, 1)));
        assert_eq!(
            find("表", plain(), FindBound::Before(Some((2, 1)))),
            Some((1, 1))
        );
    }

    #[test]
    fn edited_values_and_removed_rows_are_respected() {
        let mut edits = CellEdits::empty();
        let _old = edits.set(
            RowId::Source(1),
            ColumnId::Source(1),
            CellValue::Text(String::from("机")),
        );
        // 書き換えた 1 行目は元の「表計算ソフト」では当たらず、2 行目が先に当たる。
        assert_eq!(
            find(
                "表",
                (edits.clone(), RowOrder::whole()),
                FindBound::After(None)
            ),
            Some((2, 1))
        );
        assert_eq!(
            find("机", (edits, RowOrder::whole()), FindBound::After(None)),
            Some((1, 1))
        );
        // 2 行目（レコード 2）を並びから消すと、当たらない。
        let mut rows = RowOrder::whole();
        let _removed = rows.remove(2, 1);
        assert_eq!(
            find("能力", (CellEdits::empty(), rows), FindBound::After(None)),
            None
        );
    }
}
