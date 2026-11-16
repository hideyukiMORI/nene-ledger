//! 方言を変えた書き出し（ADR 0036）。今見えている表の全部の行を、選んだ文字コードと方言で別のファイルへ書く。
//! 書いて置き換えるのは保存と同じ裏の仕事（一時ファイルに書いて名前を変える・ADR 0025）。原本と窓の差分は変えない。

use std::path::Path;

use nene_ledger_application::export_planning::{count_cells, write_export};
use nene_ledger_application::export_shape::ExportShape;
use nene_ledger_application::save_plan_failure::SavePlanFailure;
use nene_ledger_application::table_changes::TableChanges;
use nene_ledger_core::plan_sink::PlanSink;
use nene_ledger_core::text_encoding::TextEncoding;
use nene_ledger_core::text_format::TextFormat;
use nene_ledger_scan::decoding::decode_into;
use nene_ledger_scan::scan_failure::ScanFailure;
use nene_ledger_scan::source::Source;
use nene_ledger_scan::sparse_index::SparseIndex;
use nene_ledger_store::encoding::encoded;
use nene_ledger_store::locating::same_file;
use nene_ledger_store::save_failure::SaveFailure;
use nene_ledger_store::save_job::SaveJob;
use nene_ledger_store::stamping::target_state;

use crate::open_failure::OpenFailure;
use crate::reader::Reader;
use crate::saving::{SavingFailure, source_unchanged};
use crate::scan_bytes::ScanBytes;
use crate::scan_lookup::ScanLookup;

/// BOM の字。BOM の付く文字コードで符号化すると、その BOM のバイト列になる。
const BYTE_ORDER_MARK: &str = "\u{FEFF}";

/// 書き出しを始める。差分 `changes` を重ねた表を、文字コード `encoding`・形 `shape` で `target` へ書く。
///
/// `target` は今のファイルと別でなければならない（ADR 0036 の決定 2）。既にあれば今の印と比べてから置き換える（上書きは
/// 呼ぶ側が確かめてある）。表せない字のあるセルがあれば、書き始める前に数えて止める（決定 5）。
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "ADR 0036: 入口（パレットの「形式を変えて書き出す…」）は次の Issue で足す"
    )
)]
pub(crate) fn start_exporting(
    changes: TableChanges<'_>,
    reader: &Reader,
    target: &Path,
    (encoding, shape): (TextEncoding, ExportShape),
) -> Result<SaveJob, SavingFailure> {
    if same_file(reader.path(), target) {
        return Err(SavingFailure::Save(SaveFailure::SameAsSource));
    }
    let expected =
        target_state(target).map_err(|kind| SavingFailure::Save(SaveFailure::Target(kind)))?;
    let source = reader.source_copy().map_err(OpenFailure::from)?;
    // 書いている間に原本が外から変われば、置き換えの直前に止める（名前を付けて保存と同じ・GitHub Issue #126）。
    let bytes = match (source_unchanged(reader)?, reader.path()) {
        (Some(opened), Some(path)) => ScanBytes::guarded(source, path.to_path_buf(), opened),
        (None, _) | (Some(_), None) => ScanBytes::new(source),
    };
    let (rows, columns, edits, added) = (
        changes.order().clone(),
        changes.columns().clone(),
        changes.edits().clone(),
        changes.shape(),
    );
    let table = Table {
        source: reader.source_copy().map_err(OpenFailure::from)?,
        index: reader.index_copy(),
        format: reader.format(),
    };
    let produce = move |sink: &mut dyn PlanSink| {
        let owned = TableChanges::new((&rows, &columns), &edits, added);
        table.export(owned, (encoding, shape), sink)
    };
    #[cfg(windows)]
    let replacing = nene_ledger_adapters_windows::file_replacing::FileReplacing;
    #[cfg(not(windows))]
    let replacing = nene_ledger_store::rename_replacing::RenameReplacing;
    SaveJob::spawn_streamed(
        target.to_path_buf(),
        (encoding, expected),
        (bytes, replacing, produce),
        reader.wakeup(),
    )
    .map_err(SavingFailure::Save)
}

/// 裏の仕事が読む、原本と索引と読み方の写し。
struct Table {
    source: Source,
    index: SparseIndex,
    format: TextFormat,
}

impl Table {
    /// 表せない字のあるセルを数え、無ければ BOM と全部の行を `sink` へ書く。
    fn export(
        &self,
        changes: TableChanges<'_>,
        (encoding, shape): (TextEncoding, ExportShape),
        sink: &mut dyn PlanSink,
    ) -> Result<(), SaveFailure> {
        let format = self.format;
        let decode = |bytes: &[u8]| {
            let mut text = String::new();
            decode_into(format.encoding(), bytes, &mut text);
            text
        };
        if !writes_every_character(encoding) {
            let mut lookup = ScanLookup::new(&self.source, &self.index, format);
            let found = count_cells(
                changes,
                (&mut lookup, format.dialect(), decode),
                (
                    |text: &str| encoded(text, encoding).is_none(),
                    || sink.stopped(),
                ),
            )
            .map_err(|failure| unplannable(&failure))?;
            if let Some((row, column)) = found.first() {
                return Err(SaveFailure::UnrepresentableCells {
                    count: found.count(),
                    row,
                    column,
                });
            }
        }
        if !encoding.bom().is_empty() {
            sink.text(BYTE_ORDER_MARK);
        }
        let mut lookup = ScanLookup::new(&self.source, &self.index, format);
        write_export(
            changes,
            (&mut lookup, format.dialect(), decode),
            shape,
            sink,
        )
        .map_err(|failure| unplannable(&failure))
    }
}

/// 書き出しの中身を作れなかった理由を、保存の失敗にする。
fn unplannable(failure: &SavePlanFailure<ScanFailure>) -> SaveFailure {
    SaveFailure::Unplannable(format!("{failure:?}"))
}

/// どの字も表せる文字コードか（UTF-8 と UTF-16。書く前の検査が要らない）。
const fn writes_every_character(encoding: TextEncoding) -> bool {
    match encoding {
        TextEncoding::Utf8
        | TextEncoding::Utf8WithBom
        | TextEncoding::Utf16LeWithBom
        | TextEncoding::Utf16BeWithBom
        | TextEncoding::Utf16Le
        | TextEncoding::Utf16Be => true,
        TextEncoding::ShiftJis | TextEncoding::EucJp | TextEncoding::Windows1252 => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nene_ledger_application::added_row_shape::AddedRowShape;
    use nene_ledger_application::wakeup::Wakeup;
    use nene_ledger_core::cell_edits::CellEdits;
    use nene_ledger_core::cell_value::CellValue;
    use nene_ledger_core::column_id::ColumnId;
    use nene_ledger_core::column_order::ColumnOrder;
    use nene_ledger_core::delimiter::Delimiter;
    use nene_ledger_core::dialect::Dialect;
    use nene_ledger_core::line_ending::LineEnding;
    use nene_ledger_core::quote_policy::QuotePolicy;
    use nene_ledger_core::quoting::Quoting;
    use nene_ledger_core::row_id::RowId;
    use nene_ledger_core::row_order::RowOrder;
    use nene_ledger_store::stamping::stamp_of;
    use std::path::PathBuf;
    use std::sync::Arc;

    /// 起こさない口。
    struct Quiet;

    impl Wakeup for Quiet {
        fn wake(&self) {}
    }

    /// テストごとのフォルダ `name` に、`contents` を書いた `source.csv`。
    fn fixture(name: &str, contents: &[u8]) -> PathBuf {
        let folder = std::env::temp_dir().join(format!(
            "nene-ledger-app-exporting-{}-{name}",
            std::process::id()
        ));
        std::fs::create_dir_all(&folder).unwrap();
        let path = folder.join("source.csv");
        std::fs::write(&path, contents).unwrap();
        path
    }

    fn reader_of(path: &Path) -> Reader {
        let stamp = stamp_of(path).unwrap();
        let source = Source::open(path).unwrap();
        let format = crate::opening::guess_format(&source).unwrap().format();
        Reader::new(
            (source, Some(path.to_path_buf()), Some(stamp)),
            format,
            Arc::new(Quiet),
        )
    }

    fn shape(delimiter: u8, policy: QuotePolicy, ending: LineEnding) -> ExportShape {
        let dialect =
            Dialect::new(Delimiter::new(delimiter).unwrap(), Quoting::Doubled, b'"').unwrap();
        ExportShape::new(dialect, policy, ending)
    }

    /// `path` の表に書き換え `edits` を重ねて、`target` へ書き出した結果。
    fn exported(
        path: &Path,
        edits: &CellEdits,
        target: &Path,
        format: (TextEncoding, ExportShape),
    ) -> Result<(), SavingFailure> {
        let reader = reader_of(path);
        let (rows, columns) = (RowOrder::whole(), ColumnOrder::whole());
        let changes = TableChanges::new(
            (&rows, &columns),
            edits,
            AddedRowShape::new(3, LineEnding::CrLf),
        );
        start_exporting(changes, &reader, target, format)?
            .wait()
            .map_err(SavingFailure::Save)
    }

    /// フォルダに残ったファイルの名前（昇順）。
    fn names(folder: &Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(folder)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    /// `Shift_JIS`・カンマ・CRLF の表を、BOM 付きの UTF-8・タブ・LF・すべて引用で書き出す。書き換えた値が載り、
    /// 元のファイルは 1 バイトも変わらない（ADR 0036 の決定 3・4）。
    #[test]
    fn a_shift_jis_table_is_exported_as_quoted_utf8_tsv_and_the_source_is_untouched() {
        // 「番号,名前,メモ」「0012,"東,京",」「3,大阪」（Shift_JIS）。
        let original: &[u8] = b"\x94\xd4\x8d\x86,\x96\xbc\x91\x4f,\x83\x81\x83\x82\r\n0012,\"\x93\x8c,\x8b\x9e\",\r\n3,\x91\xe5\x8d\xe3\r\n";
        let path = fixture("sjis", original);
        let target = path.with_file_name("out.tsv");
        let mut edits = CellEdits::empty();
        edits.set(
            RowId::Source(2),
            ColumnId::Source(1),
            CellValue::Text(String::from("名古屋")),
        );
        let format = (
            TextEncoding::Utf8WithBom,
            shape(b'\t', QuotePolicy::All, LineEnding::Lf),
        );
        exported(&path, &edits, &target, format).unwrap();
        assert_eq!(
            String::from_utf8(std::fs::read(&target).unwrap()).unwrap(),
            "\u{FEFF}\"番号\"\t\"名前\"\t\"メモ\"\n\"0012\"\t\"東,京\"\t\"\"\n\"3\"\t\"名古屋\"\n"
        );
        assert_eq!(std::fs::read(&path).unwrap(), original);
        assert_eq!(names(path.parent().unwrap()), ["out.tsv", "source.csv"]);
    }

    /// UTF-8 の表を `Shift_JIS` と UTF-16LE（BOM 付き）で書き出す。既にある書き出し先は置き換える。
    #[test]
    fn a_utf8_table_is_exported_as_shift_jis_and_utf16() {
        let path = fixture("utf8", "id,名前\n1,東京\n2,\"a;b\"".as_bytes());
        let target = path.with_file_name("out.csv");
        std::fs::write(&target, b"old").unwrap();
        let semicolons = shape(b';', QuotePolicy::AsNeeded, LineEnding::CrLf);
        let none = CellEdits::empty();
        exported(&path, &none, &target, (TextEncoding::ShiftJis, semicolons)).unwrap();
        assert_eq!(
            std::fs::read(&target).unwrap(),
            b"id;\x96\xbc\x91\x4f\r\n1;\x93\x8c\x8b\x9e\r\n2;\"a;b\""
        );
        let wide = (
            TextEncoding::Utf16LeWithBom,
            shape(b',', QuotePolicy::AsNeeded, LineEnding::Lf),
        );
        exported(&path, &none, &target, wide).unwrap();
        let units: Vec<u16> = std::fs::read(&target)
            .unwrap()
            .chunks(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect();
        assert_eq!(
            String::from_utf16(&units).unwrap(),
            "\u{FEFF}id,名前\n1,東京\n2,a;b"
        );
    }

    /// 表せない字のあるセルがあれば、数えて止め、書き出す先を作らない（ADR 0036 の決定 5・強制）。同じファイルへの
    /// 書き出しは断る（決定 2）。
    #[test]
    fn unrepresentable_cells_and_the_same_file_stop_the_export_before_writing() {
        let path = fixture("emoji", "id,memo\n1,😀\n2,ok\n3,𠮷野家\n".as_bytes());
        let target = path.with_file_name("out.csv");
        let plain = shape(b',', QuotePolicy::AsNeeded, LineEnding::Lf);
        let none = CellEdits::empty();
        let stopped = exported(&path, &none, &target, (TextEncoding::ShiftJis, plain));
        assert!(matches!(
            stopped,
            Err(SavingFailure::Save(SaveFailure::UnrepresentableCells {
                count: 2,
                row: 1,
                column: 1
            }))
        ));
        assert_eq!(names(path.parent().unwrap()), ["source.csv"]);
        let same = exported(&path, &none, &path, (TextEncoding::Utf8, plain));
        assert!(matches!(
            same,
            Err(SavingFailure::Save(SaveFailure::SameAsSource))
        ));
        // 書き換えて表せるようにすれば、書き出せる。
        let mut edits = CellEdits::empty();
        for (row, fixed) in [(1, "笑"), (3, "吉野家")] {
            edits.set(
                RowId::Source(row),
                ColumnId::Source(1),
                CellValue::Text(String::from(fixed)),
            );
        }
        exported(&path, &edits, &target, (TextEncoding::ShiftJis, plain)).unwrap();
        assert_eq!(
            std::fs::read(&target).unwrap(),
            b"id,memo\n1,\x8f\xce\n2,ok\n3,\x8b\x67\x96\xec\x89\xc6\n"
        );
    }
}
