//! 原本 ＋ セルの書き換えを保存する（ADR 0024・0025）。scan（原本を読む）と store（書いて置き換える）を、
//! application のポートで結ぶ。🔴 判断は持たない（計画は application、手順は store）。

use std::ops::ControlFlow;
use std::path::Path;

use nene_ledger_application::added_row_shape::AddedRowShape;
use nene_ledger_application::file_stamp::FileStamp;
use nene_ledger_application::replace_failure::ReplaceFailure;
use nene_ledger_application::save_plan_failure::SavePlanFailure;
use nene_ledger_application::save_planning::save_plan;
use nene_ledger_application::save_problem::SaveProblem;
use nene_ledger_application::table_changes::TableChanges;
use nene_ledger_core::cell_edits::CellEdits;
use nene_ledger_core::column_order::ColumnOrder;
use nene_ledger_core::line_ending::LineEnding;
use nene_ledger_core::row_order::RowOrder;
use nene_ledger_core::text_format::TextFormat;
use nene_ledger_scan::block_size::BlockSize;
use nene_ledger_scan::index_outcome::IndexOutcome;
use nene_ledger_scan::indexing::build_index;
use nene_ledger_scan::scan_failure::ScanFailure;
use nene_ledger_scan::source::Source;
use nene_ledger_scan::sparse_index::SparseIndex;
use nene_ledger_store::save_failure::SaveFailure;
use nene_ledger_store::save_job::SaveJob;
use nene_ledger_store::saving::{discard, save};
use nene_ledger_store::stamping::stamp_of;

use crate::open_failure::OpenFailure;
use crate::opening::guess_format;
use crate::reader::Reader;
use crate::scan_bytes::ScanBytes;
use crate::scan_lookup::ScanLookup;

/// 保存できなかった理由。
#[derive(Debug)]
pub(crate) enum SavingFailure {
    /// 原本を開けない・読めない。
    Open(OpenFailure),
    /// 保存の計画を作れない（書き換えた行が無い・長すぎる・その方言で書けない値）。
    Plan(SavePlanFailure<ScanFailure>),
    /// 書けない・置き換えられない。
    Save(SaveFailure),
    /// 原本のファイルが無い新しい表（ADR 0028）。
    Untitled,
}

impl From<OpenFailure> for SavingFailure {
    fn from(failure: OpenFailure) -> Self {
        Self::Open(failure)
    }
}

/// `path` を判別した読み方で開き、セルの書き換え `edits` を重ねて保存する。
///
/// 書き換えが無ければ、原本の全体を写して置き換える（`--resave`・KEEP-02 の実測）。置き換えに失敗したら、
/// 途中で止まった（`Interrupted`）とき以外は一時ファイルを消す（再試行を尋ねる画面はまだ無い）。
pub(crate) fn save_edited(path: &Path, edits: &CellEdits) -> Result<(), SavingFailure> {
    let rows = RowOrder::whole();
    // 🔴 印は開く前に取る。開いた後で変わっていれば、保存の前に違いが分かる（FILE-01）。
    let opened = stamp_of(path).map_err(|kind| SavingFailure::Save(SaveFailure::Target(kind)))?;
    let source = Source::open(path).map_err(OpenFailure::from)?;
    let format = guess_format(&source)?.format();
    let index = if edits.is_empty() {
        SparseIndex::empty()
    } else {
        finished_index(&source, format)?
    };
    let mut lookup = ScanLookup::new(&source, &index, format);
    // 端末からの保存は行を足し消ししないので、足した行の書き方は使わない。
    let columns = ColumnOrder::whole();
    let changes = TableChanges::new(
        (&rows, &columns),
        edits,
        AddedRowShape::new(1, LineEnding::CrLf),
    );
    let plan = save_plan(changes, &mut lookup, source.len(), format.dialect())
        .map_err(SavingFailure::Plan)?;
    let mut bytes = ScanBytes::new(source.try_clone().map_err(OpenFailure::from)?);
    let saved = save_with_os(path, (&plan, format, opened), &mut bytes);
    if let Err(SaveFailure::Replace { failure, temporary }) = &saved
        && *failure != ReplaceFailure::Interrupted
    {
        discard(temporary);
    }
    saved.map_err(SavingFailure::Save)
}

/// 窓の保存（Ctrl+S・ADR 0026 の決定 6）を始める。計画はここ（画面のスレッド）で作り、書いて置き換えるのは裏の仕事。
/// 比べる印は、読み手が開く前に取ったもの（開いた後の外からの変更を見つける・FILE-01）。
pub(crate) fn start_saving(
    changes: TableChanges<'_>,
    reader: &Reader,
) -> Result<SaveJob, SavingFailure> {
    let Some(path) = reader.path() else {
        return Err(SavingFailure::Untitled);
    };
    let opened = reader
        .opened()
        .ok_or(SavingFailure::Save(SaveFailure::Target(
            std::io::ErrorKind::NotFound,
        )))?;
    // 計画は原本の今の中身を読むので、外から変わっていれば計画より先に止める（行が無いと言うより正しい理由）。
    // 書く前と置き換えの直前にも store が確かめる（ADR 0025）。
    if stamp_of(path).ok() != Some(opened) {
        return Err(SavingFailure::Save(SaveFailure::ChangedOutside));
    }
    let format = reader.format();
    let plan = {
        let mut lookup = reader.lookup();
        save_plan(
            changes,
            &mut lookup,
            reader.source_length(),
            format.dialect(),
        )
        .map_err(SavingFailure::Plan)?
    };
    let bytes = ScanBytes::new(reader.source_copy().map_err(OpenFailure::from)?);
    #[cfg(windows)]
    let replacing = nene_ledger_adapters_windows::file_replacing::FileReplacing;
    #[cfg(not(windows))]
    let replacing = nene_ledger_store::rename_replacing::RenameReplacing;
    SaveJob::spawn(
        path.to_path_buf(),
        (plan, format.encoding(), opened),
        (bytes, replacing),
        reader.wakeup(),
    )
    .map_err(SavingFailure::Save)
}

/// 保存できなかった理由のうち、状態の行で示すもの。
pub(crate) fn problem_of(failure: &SavingFailure) -> SaveProblem {
    match failure {
        SavingFailure::Open(_) | SavingFailure::Plan(_) => SaveProblem::Unplannable,
        SavingFailure::Untitled => SaveProblem::Untitled,
        SavingFailure::Save(SaveFailure::ChangedOutside | SaveFailure::SourceShrank) => {
            SaveProblem::ChangedOutside
        }
        SavingFailure::Save(
            SaveFailure::Blocked(ReplaceFailure::Interrupted)
            | SaveFailure::Replace {
                failure: ReplaceFailure::Interrupted,
                ..
            },
        ) => SaveProblem::Interrupted,
        SavingFailure::Save(SaveFailure::Blocked(_) | SaveFailure::Replace { .. }) => {
            SaveProblem::Locked
        }
        SavingFailure::Save(SaveFailure::Unrepresentable(text)) => text
            .chars()
            .next()
            .map_or(SaveProblem::Other, SaveProblem::Unrepresentable),
        SavingFailure::Save(SaveFailure::HardLinked(_)) => SaveProblem::HardLinked,
        SavingFailure::Save(
            SaveFailure::Target(_)
            | SaveFailure::Write(_)
            | SaveFailure::Read(_)
            | SaveFailure::Spawn(_)
            | SaveFailure::Panicked,
        ) => SaveProblem::Other,
    }
}

/// 保存できなかった理由の文（端末とメッセージの窓に出す）。
pub(crate) fn describe(failure: &SavingFailure) -> String {
    match failure {
        SavingFailure::Open(OpenFailure::Scan(scan)) => format!("原本を読めなかった: {scan:?}"),
        SavingFailure::Open(OpenFailure::Write(error)) => format!("書き出せなかった: {error}"),
        SavingFailure::Plan(plan) => format!("保存の中身を作れなかった: {plan:?}"),
        SavingFailure::Untitled => String::from("新しい表にはまだファイルが無い"),
        SavingFailure::Save(SaveFailure::Unrepresentable(text)) => {
            format!("ファイルの文字コードで表せない字がある: {text}")
        }
        SavingFailure::Save(SaveFailure::Replace {
            failure: ReplaceFailure::Interrupted,
            temporary,
        }) => format!(
            "置き換えが途中で止まった。新しい内容は {} にある（消していない）",
            temporary.display()
        ),
        SavingFailure::Save(SaveFailure::Replace { failure, .. }) => {
            format!("置き換えられなかった（{failure:?}）。原本は元のまま")
        }
        SavingFailure::Save(SaveFailure::ChangedOutside) => String::from(
            "開いた後にほかのアプリがファイルを変えたので、保存しなかった（原本は元のまま）",
        ),
        SavingFailure::Save(SaveFailure::Blocked(_)) => String::from(
            "ほかのアプリがファイルを開いているので、保存を始めなかった（原本は元のまま）",
        ),
        SavingFailure::Save(save) => format!("保存できなかった: {save:?}"),
    }
}

/// 索引を作り終える（中止を求めない）。
fn finished_index(source: &Source, format: TextFormat) -> Result<SparseIndex, OpenFailure> {
    let outcome = build_index(source, format, BlockSize::MEBIBYTE, |_| {
        ControlFlow::Continue(())
    })?;
    Ok(match outcome {
        IndexOutcome::Finished(index) => index,
        // 中止を求めないので来ないが、来ても目印の無い索引で先頭から切り進めれば同じ結果になる。
        IndexOutcome::Cancelled => SparseIndex::empty(),
    })
}

/// OS の置き換え（Linux は `rename`・Windows は `ReplaceFileW`・施主の判断 1）で保存する。
fn save_with_os(
    path: &Path,
    (plan, format, opened): (
        &nene_ledger_core::save_plan::SavePlan,
        TextFormat,
        FileStamp,
    ),
    bytes: &mut ScanBytes,
) -> Result<(), SaveFailure> {
    #[cfg(windows)]
    let replacing = nene_ledger_adapters_windows::file_replacing::FileReplacing;
    #[cfg(not(windows))]
    let replacing = nene_ledger_store::rename_replacing::RenameReplacing;
    save(path, (plan, format.encoding(), opened), bytes, &replacing)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nene_ledger_core::cell_value::CellValue;
    use nene_ledger_core::column_id::ColumnId;
    use nene_ledger_core::row_id::RowId;
    use std::path::PathBuf;

    fn fixture(name: &str, contents: &[u8]) -> PathBuf {
        let folder =
            std::env::temp_dir().join(format!("nene-ledger-app-saving-{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        let path = folder.join(name);
        std::fs::write(&path, contents).unwrap();
        path
    }

    fn text(value: &str) -> CellValue {
        CellValue::Text(String::from(value))
    }

    #[test]
    fn resaving_without_edits_keeps_every_byte() {
        let original = b"\xEF\xBB\xBFid,name\r\n0012,\"Sato, Taro\"\r\n0013,1-2\r\n".to_vec();
        let path = fixture("resave.csv", &original);
        save_edited(&path, &CellEdits::empty()).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), original);
    }

    #[test]
    fn one_rewritten_cell_changes_only_that_cell() {
        let original = b"\xEF\xBB\xBFid,name\r\n0012,\"Sato, Taro\"\r\n0013,1-2\r\n".to_vec();
        let path = fixture("edit.csv", &original);
        let mut edits = CellEdits::empty();
        edits.set(RowId::Source(2), ColumnId::Source(1), text("2026-11-05"));
        edits.set(RowId::Source(1), ColumnId::Source(1), text("Suzuki"));
        save_edited(&path, &edits).unwrap();
        assert_eq!(
            std::fs::read(&path).unwrap(),
            b"\xEF\xBB\xBFid,name\r\n0012,\"Suzuki\"\r\n0013,2026-11-05\r\n"
        );
    }

    #[test]
    fn a_shift_jis_file_is_saved_in_shift_jis() {
        // 「名前」「佐藤」（CP932）。
        let original = b"id,\x96\xbc\x91\x4f\r\n1,\x8d\xb2\x93\xa1\r\n".to_vec();
        let path = fixture("sjis.csv", &original);
        let mut edits = CellEdits::empty();
        edits.set(RowId::Source(1), ColumnId::Source(1), text("鈴木"));
        save_edited(&path, &edits).unwrap();
        assert_eq!(
            std::fs::read(&path).unwrap(),
            b"id,\x96\xbc\x91\x4f\r\n1,\x97\xe9\x96\xd8\r\n"
        );
        let mut emoji = CellEdits::empty();
        emoji.set(RowId::Source(1), ColumnId::Source(1), text("😀"));
        let refused = save_edited(&path, &emoji);
        assert!(matches!(
            refused,
            Err(SavingFailure::Save(SaveFailure::Unrepresentable(_)))
        ));
    }

    /// 新しい表（BOM だけの原本 ＋ 足した 1 行と 3 列）の保存の中身は、BOM 付きの UTF-8・CRLF・カンマで、
    /// 要るときだけ引用する（ADR 0028 の決定 3・強制の 1 つ目）。
    #[test]
    fn a_blank_table_is_planned_as_utf8_with_a_bom_crlf_and_commas() {
        use nene_ledger_core::save_step::SaveStep;
        let summary = crate::opening::blank_summary();
        let format = summary.format();
        let source = crate::opening::blank_source();
        let mut rows = RowOrder::whole();
        let added_rows = rows.insert(0, 2);
        let mut columns = ColumnOrder::whole();
        let added = columns.insert(0, 3);
        let mut edits = CellEdits::empty();
        edits.set(added_rows[0], added[0], text("名前"));
        edits.set(added_rows[0], added[2], text("a,b"));
        edits.set(added_rows[1], added[1], text("0012"));
        let ending = summary
            .line_endings()
            .dominant()
            .unwrap_or(LineEnding::CrLf);
        let changes = TableChanges::new((&rows, &columns), &edits, AddedRowShape::new(3, ending));
        let index = SparseIndex::empty();
        let mut lookup = ScanLookup::new(&source, &index, format);
        let plan = save_plan(changes, &mut lookup, source.len(), format.dialect()).unwrap();
        let mut written = Vec::new();
        for step in plan.steps() {
            match step {
                SaveStep::Copy { offset, length } => {
                    let size = BlockSize::new(usize::try_from(*length).unwrap()).unwrap();
                    written.extend(source.read_block(*offset, size).unwrap());
                }
                SaveStep::Text(text) => written.extend_from_slice(text.as_bytes()),
            }
        }
        assert_eq!(written, "\u{FEFF}名前,,\"a,b\"\r\n,0012,\r\n".as_bytes());
    }
}
