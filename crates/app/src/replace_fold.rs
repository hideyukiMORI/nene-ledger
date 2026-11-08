//! 全部の置換で、原本のレコードの当たりを裏の仕事で数える畳み込み（ADR 0030 の決定 6）。

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use nene_ledger_application::find_scope::FindScope;
use nene_ledger_core::cell_change::CellChange;
use nene_ledger_core::header_rows::HeaderRows;
use nene_ledger_core::row_id::RowId;
use nene_ledger_core::row_order::RowOrder;
use nene_ledger_core::text_format::TextFormat;
use nene_ledger_core::value::value;
use nene_ledger_scan::decoding::decode;
use nene_ledger_scan::record_fold::RecordFold;
use nene_ledger_scan::split_fold::SplitFold;
use nene_ledger_scan::window_record::WindowRecord;

/// 原本のレコードを順に照らし、当たったセルを置き換える書き換え（と、そのセルのファイルの値）を集める。見出しのレコードと、
/// 並びから消したレコードは照らさない（検索と同じ）。1 MiB で切り詰めたレコードは値の全体が見えないので照らさずに数える。
/// 上限 `limit` を越えたら集めるのをやめる。数は並列の部分の間で分け合う（`found`・どれかの部分で全体が上限を越えれば
/// 全部の部分がやめる。全体が上限を越えたときだけやめるので、答えは分け方に依らない・QLT-008）。足した行は application が
/// 数える（原本に無い）。
#[derive(Debug, Clone)]
pub(crate) struct ReplaceFold {
    scope: FindScope,
    replacement: String,
    format: TextFormat,
    header: Option<HeaderRows>,
    rows: RowOrder,
    limit: usize,
    found: Arc<AtomicUsize>,
    changes: Vec<(CellChange, String)>,
    skipped: u64,
    too_many: bool,
}

impl ReplaceFold {
    /// 問いと列と書き換え `scope` の当たりを `replacement` に置き換える。読み方は `format`・見出しのレコードは `header`・
    /// 行の並びは `rows`。`limit` セルを越えたらやめる。
    pub(crate) fn new(
        (scope, replacement): (FindScope, String),
        (format, header): (TextFormat, Option<HeaderRows>),
        rows: RowOrder,
        limit: usize,
    ) -> Self {
        Self {
            scope,
            replacement,
            format,
            header,
            rows,
            limit,
            found: Arc::new(AtomicUsize::new(0)),
            changes: Vec::new(),
            skipped: 0,
            too_many: false,
        }
    }

    /// 集めた書き換えと、切り詰めたので飛ばしたレコードの数。上限を越えたかどうか。
    pub(crate) fn result(self) -> ((Vec<(CellChange, String)>, u64), bool) {
        ((self.changes, self.skipped), self.too_many)
    }
}

impl RecordFold for ReplaceFold {
    fn record(&mut self, number: u64, record: &WindowRecord) {
        if self.too_many || self.found.load(Ordering::Relaxed) > self.limit {
            self.too_many = true;
            self.changes.clear();
            return;
        }
        if self.header.is_some_and(|header| header.contains(number)) {
            return;
        }
        let row = RowId::Source(number);
        if !self.rows.is_whole() && self.rows.position_of(row).is_none() {
            return;
        }
        if record.is_truncated() {
            self.skipped = self.skipped.saturating_add(1);
            return;
        }
        if self.format.encoding().keeps_ascii()
            && self
                .scope
                .skips_ascii_record(row, record.bytes(), &self.format.dialect())
        {
            return;
        }
        let dialect = self.format.dialect();
        let encoding = self.format.encoding();
        let fields = record.record().fields();
        let source = |index: usize| {
            fields.get(index).map(|&field| {
                let raw = value(record.bytes(), field, &dialect);
                String::from(decode(encoding, &raw).text())
            })
        };
        for display in 0..self.scope.width() {
            let Some(change) = self
                .scope
                .replaced_cell((row, display), &source, &self.replacement)
            else {
                continue;
            };
            self.changes.push(change);
            let total = self.found.fetch_add(1, Ordering::Relaxed).saturating_add(1);
            if total > self.limit {
                self.too_many = true;
                self.changes.clear();
                return;
            }
        }
    }

    fn is_done(&self) -> bool {
        self.too_many
    }
}

impl SplitFold for ReplaceFold {
    /// 前の部分の書き換えに後ろの部分を続ける（並びはレコードの順・分け方に依らない・QLT-008）。
    fn merged(mut self, later: Self) -> Self {
        self.skipped = self.skipped.saturating_add(later.skipped);
        self.too_many = self.too_many
            || later.too_many
            || self.changes.len().saturating_add(later.changes.len()) > self.limit;
        if self.too_many {
            self.changes.clear();
        } else {
            self.changes.extend(later.changes);
        }
        self
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

    /// 見本 `shift-jis-pipe.csv`（`Shift_JIS`・`|` 区切り・見出しの `コード|品名|単価` ＋ 3 行）の当たりを数える。
    /// 塊の大きさを変えても、2 つに分けて合わせても同じ答え。
    fn count(
        text: &str,
        (edits, rows): (CellEdits, RowOrder),
        limit: usize,
    ) -> ((Vec<(CellChange, String)>, u64), bool) {
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
        let fold = || {
            ReplaceFold::new(
                (scope.clone(), String::from("■")),
                (format, Some(HeaderRows::single(0))),
                rows.clone(),
                limit,
            )
        };
        let run = |mut folding: ReplaceFold, range: RecordRange, size: usize| {
            let flow = fold_range(&from, range.chunks(size), &mut folding, |_| {
                ControlFlow::Continue(())
            });
            assert_eq!(flow.unwrap(), ControlFlow::Continue(()));
            folding
        };
        let answers: Vec<_> = [1_usize, 2, 100]
            .iter()
            .map(|&size| run(fold(), RecordRange::new(0, 100), size).result())
            .collect();
        // 裏の仕事と同じく、1 つの畳み込みの写しを部分ごとに走らせる（数を分け合う）。
        let parts = fold();
        let split = run(parts.clone(), RecordRange::new(0, 2), 1)
            .merged(run(parts, RecordRange::new(2, 98), 1))
            .result();
        assert!(answers.iter().all(|answer| *answer == split), "{answers:?}");
        split
    }

    #[test]
    fn matching_cells_are_collected_in_record_order_over_shown_values() {
        let ((changes, skipped), too_many) =
            count("表", (CellEdits::empty(), RowOrder::whole()), 10);
        assert!(!too_many);
        assert_eq!(skipped, 0);
        let cells: Vec<(RowId, ColumnId)> = changes
            .iter()
            .map(|(change, _)| (change.row(), change.column()))
            .collect();
        assert_eq!(
            cells,
            [
                (RowId::Source(1), ColumnId::Source(1)),
                (RowId::Source(2), ColumnId::Source(1))
            ]
        );
        let (first, file) = changes.first().unwrap();
        assert_eq!(first.after(), first.before().replace('表', "■"));
        assert_eq!(file, first.before());
        // 見出しの「品名」は置き換えない。
        assert!(
            count("品名", (CellEdits::empty(), RowOrder::whole()), 10)
                .0
                .0
                .is_empty()
        );
    }

    #[test]
    fn edits_removed_rows_and_the_limit_are_respected() {
        let mut edits = CellEdits::empty();
        let _desk = edits.set(
            RowId::Source(1),
            ColumnId::Source(1),
            CellValue::Text(String::from("机")),
        );
        let ((edited, _), _) = count("表", (edits, RowOrder::whole()), 10);
        assert_eq!(edited.len(), 1);
        let mut rows = RowOrder::whole();
        let _removed = rows.remove(2, 1);
        let ((kept, _), _) = count("表", (CellEdits::empty(), rows), 10);
        assert_eq!(kept.len(), 1);
        let ((none, _), too_many) = count("表", (CellEdits::empty(), RowOrder::whole()), 1);
        assert!(too_many && none.is_empty());
    }
}
