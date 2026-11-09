//! 利用者の意図。

use nene_ledger_core::pixel_point::PixelPoint;
use nene_ledger_core::pointer_point::PointerPoint;

use crate::column_place::ColumnPlace;
use crate::column_shift::ColumnShift;
use crate::cursor_step::CursorStep;
use crate::find_toggle::FindToggle;
use crate::go_to_target::GoToTarget;
use crate::row_place::RowPlace;
use crate::scroll::Scroll;
use crate::selection_mode::SelectionMode;
use crate::theme_step::ThemeStep;
use crate::zoom_step::ZoomStep;

/// 利用者の操作を UI が変えた意図。閉じた集合（ARC-011）。どう動くかは application が決める。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent {
    /// 表示だけを動かす（ホイール）。カーソルは動かさない。
    Scroll(Scroll),
    /// カーソルを動かす（キー）。カーソルが見えるように表示も動かす。
    Move {
        /// 動かし方。
        step: CursorStep,
        /// 選択を畳むか広げるか。
        mode: SelectionMode,
    },
    /// 窓の中の位置 `at`（物理画素）のセルを選ぶ（クリック）。
    Point {
        /// 窓の左上を原点とする位置。
        at: PixelPoint,
        /// 選択を畳むか広げるか。
        mode: SelectionMode,
    },
    /// 左のボタンを 2 度続けて押した（2 度目・`at` は窓の左上を原点とする位置）。本体のセルなら、押してから
    /// そのセルを編集の状態で直し始める（F2 と同じ・ADR 0026 の決定 1）。セルの外なら 1 度押したのと同じ。
    /// 2 度目かを決めるのは UI（OS のダブルクリックの間と距離）。
    DoubleClick {
        /// 窓の左上を原点とする位置。
        at: PixelPoint,
    },
    /// 左のボタンを押したまま指を `at` へ動かした（窓の外へ出ると負にもなる）。
    Drag {
        /// 窓の左上を原点とする位置。
        at: PointerPoint,
    },
    /// 左のボタンを離した。
    Release,
    /// 行（と列）へ飛んでそのセルを選ぶ（Issue #54）。
    GoTo(GoToTarget),
    /// 飛ぶ先を打つ入力欄を開く（Ctrl+G）。
    OpenGoTo,
    /// すべてのセルを選ぶ（Ctrl+A・左上の角）。
    SelectAll,
    /// 今の選択が掛かる行の全体を選ぶ（Shift+Space・Issue #62）。
    SelectRows,
    /// 今の選択が掛かる列の全体を選ぶ（Ctrl+Space）。
    SelectColumns,
    /// 今の選択が掛かる列を、表示の上で 1 つ移す（Ctrl+Shift+PageUp / PageDown・表示だけ）。
    MoveColumns(ColumnShift),
    /// 列の並びを元に戻す（入れ替えは表示だけ・Issue #49）。
    ResetColumns,
    /// 列の幅を、開いたときの自動の幅に戻す（Issue #64）。
    ResetWidths,
    /// カーソルの列まで（表示の位置で）左に固定する（表示だけ・VIEW-01・Issue #4）。
    FreezeColumns,
    /// 列の固定を外す。
    UnfreezeColumns,
    /// 選択した矩形をコピーする（Ctrl+C）。
    Copy,
    /// 選択した矩形を切り取る（Ctrl+X）。コピーと同じ文字列を置き、この窓で貼ったら元を空にする（ADR 0029 の決定 5）。
    Cut,
    /// 検索の欄を開く（Ctrl+F・ADR 0030 の決定 1）。`replace` なら置換の欄（Ctrl+H・決定 6）。
    OpenFind {
        /// 置換の欄を開くか。
        replace: bool,
    },
    /// 前に探した問いで、次（`forward`）か前を探す（F3・Shift+F3）。
    FindAgain {
        /// 次へ探すか。
        forward: bool,
    },
    /// 検索の欄で条件を切り替える（Alt+C・Alt+W）。
    FindToggle(FindToggle),
    /// 置換の欄で、当たりを全部置き換える（Alt+A・件数を見せてから）。
    ReplaceAll,
    /// 詳細ペインを開閉する（Ctrl+Shift+U）。
    ToggleDetail,
    /// 読み取り専用にする / 外す（パレット・EDIT-13・GitHub Issue #238）。
    ToggleReadOnly,
    /// 配色を表の中で替える（試しに・Ctrl+Shift+T と Ctrl+Shift+Alt+T）。
    Theme(ThemeStep),
    /// 表と詳細ペインの文字の大きさを変える（Ctrl+ホイール・Ctrl+0・パレット。Ctrl++ / Ctrl+- は行と列の足し消し・ADR 0027）。
    Zoom(ZoomStep),
    /// コマンドパレットを開く（Ctrl+Shift+P・F1）。
    OpenPalette,
    /// 字を打った（半角の英数字と記号。日本語の入力は FR-020）。
    Type(char),
    /// 1 字消す（Backspace）。
    Erase,
    /// 決める（Enter）。
    Confirm,
    /// やめる（Esc）。
    Cancel,
    /// カーソルのセルを、見えていた値の続きから直す（F2）。編集中なら入力と編集の状態を替える（ADR 0026）。
    Edit,
    /// 右の 1 字を消す（Delete）。編集していなければ、カーソルのセルを空にする。
    Delete,
    /// セルの中の改行（Alt+Enter・EDIT-10）。
    BreakLine,
    /// 編集中の欄の文字を、選んだ範囲のすべてのセルに書く（Ctrl+Enter・EDIT-02・GitHub Issue #203）。合成ルートが
    /// `Document::fill_selection` に渡す（貼り付けと同じく、書く行を読んでから書く）。
    FillSelection,
    /// 確定してその向きへ動く（Tab・Shift+Tab・Shift+Enter）。編集していなければ動くだけ。
    Advance(CursorStep),
    /// 書き換えを保存する（Ctrl+S・ADR 0026 の決定 6）。
    Save,
    /// 別の名前・場所に保存する（Ctrl+Shift+S・ADR 0028 の決定 4）。
    SaveAs,
    /// 新しい表を新しい窓で開く（Ctrl+N・ADR 0028 の決定 1）。合成ルートが受ける（表の状態は変えない）。
    NewTable,
    /// ファイルを選んで新しい窓で開く（Ctrl+O・GitHub Issue #344）。合成ルートが受ける（表の状態は変えない）。
    OpenFile,
    /// クリップボードの文字列を貼る（Ctrl+V・Shift+Insert・CLIP-01）。UI が OS のクリップボードを読み、
    /// [`crate::session::Session::paste`] で渡す（ここまで来たら何もしない）。
    Paste,
    /// 1 手元に戻す（Ctrl+Z・UNDO-01）。編集中なら編集をやめる。
    Undo,
    /// 戻した 1 手をやり直す（Ctrl+Y・Ctrl+Shift+Z）。
    Redo,
    /// 空の行を足す（ADR 0027 の決定 2・EDIT-03）。編集中なら確定してから。
    InsertRows(RowPlace),
    /// 選んだ範囲にかかる行を消す（尋ねずに消し、Ctrl+Z で戻す・ADR 0027 の決定 4）。
    RemoveRows,
    /// 空の列を足す（ADR 0027 の決定 2）。編集中なら確定してから。
    InsertColumns(ColumnPlace),
    /// 選んだ範囲にかかる列を消す（尋ねずに消し、Ctrl+Z で戻す）。
    RemoveColumns,
    /// Ctrl++（ADR 0027 の決定 2）。列全体を選んでいれば左に列を、そうでなければ上に行を足す。
    InsertHere,
    /// Ctrl+-。列全体を選んでいれば列を、そうでなければ選んだ範囲にかかる行を消す。
    RemoveHere,
}
