//! 状態の行に一度だけ出す知らせ。

use crate::columns_notice::ColumnsNotice;
use crate::find_notice::FindNotice;
use crate::paste_notice::PasteNotice;
use crate::read_only_notice::ReadOnlyNotice;
use crate::replace_notice::ReplaceNotice;
use crate::rows_notice::RowsNotice;
use crate::save_problem::SaveProblem;
use crate::zoom_level::ZoomLevel;

/// 状態の行に一度だけ出す知らせ。次の操作で消える。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Notice {
    /// `rows` 行 × `columns` 列をコピーした。
    Copied {
        /// 行の数。
        rows: u64,
        /// 列の数。
        columns: usize,
        /// そのうち、上限で切り詰めた値の行の数（GitHub Issue #36）。
        cut: u64,
    },
    /// 開いたファイルが文字のファイルでないようだ（NUL のバイトがある・Issue #57）。
    LooksBinary,
    /// 選択が大きすぎてコピーしなかった。
    TooLargeToCopy,
    /// 離れた範囲の行も列も揃っていないのでコピーしなかった（Issue #46）。
    RangesNotAligned,
    /// ファイルの終わりまでの選択を、索引を作り終える前なのでコピーしなかった（行の数がまだ決まらない・GitHub Issue #39）。
    CopyAwaitsIndex,
    /// コピーする行が無いのでコピーしなかった（クリップボードを替えない・GitHub Issue #39）。
    NothingToCopy,
    /// 設定を書けなかった（変えた見た目はこの起動の間だけ）。
    SettingsUnsaved,
    /// 文字の大きさを変えた。
    Zoom(ZoomLevel),
    /// 配色を替えた。
    Theme {
        /// 配色の名前。
        name: &'static str,
        /// 表の中の位置（0 始まり）。
        position: usize,
    },
    /// 左の `columns` 列を固定した（0 なら外した・VIEW-01・Issue #4）。
    Frozen {
        /// 固定した列の数。
        columns: usize,
    },
    /// 保存した。
    Saved,
    /// 1 手元に戻した（書き換えたセルの数）。
    Undone(usize),
    /// 1 手やり直した（書き換えたセルの数）。
    Redone(usize),
    /// 戻すものが無い。
    NothingToUndo,
    /// やり直すものが無い。
    NothingToRedo,
    /// 書き換えが無いので保存しなかった。
    NothingToSave,
    /// 保存の途中なので、直すことと保存を受けない。
    StillSaving,
    /// 保存できなかった（書き換えは残っている）。
    SaveFailed(SaveProblem),
    /// 保存したが、置き換えた後のファイルを開き直せなかった（表示は保存の前のまま）。
    ReopenFailed,
    /// 表への貼り付け（ADR 0029）。
    Paste(PasteNotice),
    /// 検索（ADR 0030）。
    Find(FindNotice),
    /// 置換（ADR 0030 の決定 6）。
    Replace(ReplaceNotice),
    /// 新しい表の窓を開けなかった（自分の exe を起こせない・ADR 0028 の決定 1）。
    NewWindowFailed,
    /// ファイルを選ぶ画面が無い（X11）。開くファイルは引数で渡すか、窓に落とす（GitHub Issue #344）。
    NoFileChooser,
    /// 読み取り専用（EDIT-13・GitHub Issue #238）。
    ReadOnly(ReadOnlyNotice),
    /// 新しい表は、読み方を変えられない（名前を付けて保存してから・ADR 0028 の決定 3）。
    ReadingNeedsFile,
    /// 書き換えがある間は、読み方を変えられない（先に保存する）。
    ReadingNeedsSave,
    /// 1 MiB で切り詰めたレコードのセルは直せない（保存の計画と同じ・ADR 0026 の決定 1）。
    EditCut,
    /// 行の足し消し（ADR 0027）。
    Rows(RowsNotice),
    /// 列の足し消し（ADR 0027）。
    Columns(ColumnsNotice),
    /// このセルは差分で消した（`Absent`・KEEP-05）ので直せない。列の数が足りない行のセルは足して直せる（GitHub Issue #83）。
    EditMissingCell,
    /// 方言の判別に確信が無い（FR-023・Issue #3）。黙って 1 位で開いたことを知らせ、候補から選ぶ入口を示す。
    DialectUnsure {
        /// 1 位の点数（千分率）。
        top: u32,
        /// 2 位の点数（千分率）。
        second: u32,
    },
}
