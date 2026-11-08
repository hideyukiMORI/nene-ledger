//! パレットから実行できる操作。

use crate::column_place::ColumnPlace;
use crate::column_shift::ColumnShift;
use crate::cursor_step::CursorStep;
use crate::detail_pane::DetailPane;
use crate::intent::Intent;
use crate::reading_aspect::ReadingAspect;
use crate::row_place::RowPlace;
use crate::selection_mode::SelectionMode;
use crate::zoom_step::ZoomStep;

/// パレットから実行できる操作。閉じた集合で、言語に依らない識別子（表示名は言語ごとの表から引く・ADR 0011）。
/// 並びは分類（移動・表示・コピー）の固定の順で、何も打たないときの候補の順（Issue #41）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandId {
    /// 表の先頭に移る。
    Top,
    /// 表の末尾に移る。
    Bottom,
    /// 行の番号を打って飛ぶ（2 段目で打つ・Issue #54）。
    GoTo,
    /// すべてのセルを選ぶ。
    SelectAll,
    /// 選択が掛かる行の全体を選ぶ（Issue #62）。
    SelectRows,
    /// 選択が掛かる列の全体を選ぶ。
    SelectColumns,
    /// 詳細ペインを開く / 閉じる。
    ToggleDetail,
    /// 読み取り専用にする / 外す（EDIT-13・GitHub Issue #238）。
    ToggleReadOnly,
    /// 文字を大きくする。
    ZoomIn,
    /// 文字を小さくする。
    ZoomOut,
    /// 文字の大きさを戻す。
    ZoomReset,
    /// 配色を選ぶ（2 段目で試して選ぶ）。
    ChooseTheme,
    /// 判別の候補から選んで開き直す（2 段目に点数の順で並べ、試して選ぶ・Issue #3）。
    ChooseDialect,
    /// 文字コードを選んで開き直す（2 段目で試して選ぶ・Issue #52）。
    ChooseEncoding,
    /// 区切りを選んで開き直す。
    ChooseDelimiter,
    /// 引用符の扱いを選んで開き直す。
    ChooseQuoting,
    /// 見出しの行を選ぶ。
    ChooseHeader,
    /// 選んだ列を左へ移す（表示だけ・Issue #62）。
    MoveColumnsLeft,
    /// 選んだ列を右へ移す。
    MoveColumnsRight,
    /// 列の並びを元に戻す。
    ResetColumns,
    /// 列の幅を元に戻す（Issue #64）。
    ResetWidths,
    /// カーソルの列まで左に固定する（表示だけ・Issue #4）。
    FreezeColumns,
    /// 列の固定を外す。
    UnfreezeColumns,
    /// 選んだ範囲をコピーする。
    Copy,
    /// 選んだ範囲を切り取る（Ctrl+X・ADR 0029 の決定 5）。
    Cut,
    /// 探す（Ctrl+F・ADR 0030 の決定 1）。
    Find,
    /// 次を探す（F3）。
    FindNext,
    /// 前を探す（Shift+F3）。
    FindPrevious,
    /// 置き換える（Ctrl+H・ADR 0030 の決定 6）。
    Replace,
    /// 書き換えを保存する（ADR 0026 の決定 6）。
    Save,
    /// 名前を付けて保存する。
    SaveAs,
    /// 新しい表を新しい窓で作る（ADR 0028 の決定 1）。
    NewTable,
    /// 1 手元に戻す（UNDO-01）。
    Undo,
    /// 戻した 1 手をやり直す。
    Redo,
    /// カーソルの行（選んだ行）の上に空の行を足す（ADR 0027）。
    InsertRowsAbove,
    /// 選んだ行の下に空の行を足す。
    InsertRowsBelow,
    /// 表の末尾に空の行を足す。
    AppendRow,
    /// 選んだ範囲にかかる行を消す。
    RemoveRows,
    /// カーソルの列（選んだ列）の左に空の列を足す（ADR 0027）。
    InsertColumnsLeft,
    /// 選んだ列の右に空の列を足す。
    InsertColumnsRight,
    /// 選んだ範囲にかかる列を消す。
    RemoveColumns,
    /// キーの一覧を見る（2 段目・Issue #63）。
    ShowKeys,
    /// このアプリについて（版・著作権・ライセンス・2 段目・ADR 0032）。
    ShowAbout,
}

impl CommandId {
    /// すべての操作（何も打たないときの順）。
    pub const ALL: [Self; 43] = [
        Self::Top,
        Self::Bottom,
        Self::GoTo,
        Self::SelectAll,
        Self::SelectRows,
        Self::SelectColumns,
        Self::ToggleDetail,
        Self::ZoomIn,
        Self::ZoomOut,
        Self::ZoomReset,
        Self::ChooseTheme,
        Self::ChooseDialect,
        Self::ChooseEncoding,
        Self::ChooseDelimiter,
        Self::ChooseQuoting,
        Self::ChooseHeader,
        Self::MoveColumnsLeft,
        Self::MoveColumnsRight,
        Self::ResetColumns,
        Self::ResetWidths,
        Self::FreezeColumns,
        Self::UnfreezeColumns,
        Self::Copy,
        Self::Cut,
        Self::Find,
        Self::FindNext,
        Self::FindPrevious,
        Self::Replace,
        Self::Save,
        Self::SaveAs,
        Self::NewTable,
        Self::Undo,
        Self::Redo,
        Self::InsertRowsAbove,
        Self::InsertRowsBelow,
        Self::AppendRow,
        Self::RemoveRows,
        Self::InsertColumnsLeft,
        Self::InsertColumnsRight,
        Self::RemoveColumns,
        Self::ToggleReadOnly,
        Self::ShowKeys,
        Self::ShowAbout,
    ];

    /// 表示名（「対象を＋動詞」の言い切り。次に選ぶものがあれば「…」）。開閉は今の状態で名前が変わる。
    /// 🔴 表示言語の文言の表（ADR 0011）を入れるまでの仮の日本語。
    #[must_use]
    pub fn name(self, detail: DetailPane) -> &'static str {
        match self {
            Self::Top => "表の先頭に移る",
            Self::Bottom => "表の末尾に移る",
            Self::GoTo => "行へ飛ぶ…",
            Self::SelectAll => "すべてのセルを選ぶ",
            Self::SelectRows => "行全体を選ぶ",
            Self::SelectColumns => "列全体を選ぶ",
            Self::ToggleDetail => match detail {
                DetailPane::Open => "詳細ペインを閉じる",
                DetailPane::Closed => "詳細ペインを開く",
            },
            Self::ToggleReadOnly => "読み取り専用を切り替える",
            Self::ZoomIn => "文字を大きくする",
            Self::ZoomOut => "文字を小さくする",
            Self::ZoomReset => "文字の大きさを戻す",
            Self::ChooseTheme => "配色を選ぶ…",
            Self::ChooseDialect => "判別の候補から選んで開き直す…",
            Self::ChooseEncoding => "文字コードを選んで開き直す…",
            Self::ChooseDelimiter => "区切りを選んで開き直す…",
            Self::ChooseQuoting => "引用符の扱いを選んで開き直す…",
            Self::ChooseHeader => "見出しの行を選ぶ…",
            Self::MoveColumnsLeft => "選んだ列を左へ移す",
            Self::MoveColumnsRight => "選んだ列を右へ移す",
            Self::ResetColumns => "列の並びを元に戻す",
            Self::ResetWidths => "列の幅を元に戻す",
            Self::FreezeColumns => "カーソルの列まで左に固定する",
            Self::UnfreezeColumns => "列の固定を外す",
            Self::Copy => "選んだ範囲をコピーする",
            Self::Cut => "選んだ範囲を切り取る（貼るまで元は消さない）",
            Self::Find => "探す（見えている値で）",
            Self::FindNext => "次を探す",
            Self::FindPrevious => "前を探す",
            Self::Replace => "置き換える（探して）",
            Self::Save => "変えた所を保存する",
            Self::SaveAs => "名前を付けて保存する",
            Self::NewTable => "新しい表を作る",
            Self::Undo => "元に戻す",
            Self::Redo => "やり直す",
            Self::InsertRowsAbove => "上に行を足す",
            Self::InsertRowsBelow => "下に行を足す",
            Self::AppendRow => "末尾に行を足す",
            Self::RemoveRows => "行を消す",
            Self::InsertColumnsLeft => "左に列を足す",
            Self::InsertColumnsRight => "右に列を足す",
            Self::RemoveColumns => "列を消す",
            Self::ShowKeys => "キーの一覧を見る…",
            Self::ShowAbout => "このアプリについて…",
        }
    }

    /// 名前の読み（ひらがな）。ローマ字はここから機械で作る。開閉はどちらの語でも当たるよう両方を持つ。
    #[must_use]
    pub fn reading(self) -> &'static str {
        match self {
            Self::Top => "ひょうのせんとうにうつる",
            Self::Bottom => "ひょうのまつびにうつる",
            Self::GoTo => "ぎょうへとぶ",
            Self::SelectAll => "すべてのせるをえらぶ",
            Self::SelectRows => "ぎょうぜんたいをえらぶ",
            Self::SelectColumns => "れつぜんたいをえらぶ",
            Self::ToggleDetail => "しょうさいぺいんをひらくとじる",
            Self::ToggleReadOnly => "よみとりせんようをきりかえる",
            Self::ZoomIn => "もじをおおきくする",
            Self::ZoomOut => "もじをちいさくする",
            Self::ZoomReset => "もじのおおきさをもどす",
            Self::ChooseTheme => "はいしょくをえらぶ",
            Self::ChooseDialect => "はんべつのこうほからえらんでひらきなおす",
            Self::ChooseEncoding => "もじこーどをえらんでひらきなおす",
            Self::ChooseDelimiter => "くぎりをえらんでひらきなおす",
            Self::ChooseQuoting => "いんようふのあつかいをえらんでひらきなおす",
            Self::ChooseHeader => "みだしのぎょうをえらぶ",
            Self::MoveColumnsLeft => "えらんだれつをひだりへうつす",
            Self::MoveColumnsRight => "えらんだれつをみぎへうつす",
            Self::ResetColumns => "れつのならびをもとにもどす",
            Self::ResetWidths => "れつのはばをもとにもどす",
            Self::FreezeColumns => "かーそるのれつまでひだりにこていする",
            Self::UnfreezeColumns => "れつのこていをはずす",
            Self::Copy => "えらんだはんいをこぴーする",
            Self::Cut => "えらんだはんいをきりとる",
            Self::Find => "さがす",
            Self::FindNext => "つぎをさがす",
            Self::FindPrevious => "まえをさがす",
            Self::Replace => "おきかえる",
            Self::Save => "かえたところをほぞんする",
            Self::SaveAs => "なまえをつけてほぞんする",
            Self::NewTable => "あたらしいひょうをつくる",
            Self::Undo => "もとにもどす",
            Self::Redo => "やりなおす",
            Self::InsertRowsAbove => "うえにぎょうをたす",
            Self::InsertRowsBelow => "したにぎょうをたす",
            Self::AppendRow => "まつびにぎょうをたす",
            Self::RemoveRows => "ぎょうをけす",
            Self::InsertColumnsLeft => "ひだりにれつをたす",
            Self::InsertColumnsRight => "みぎにれつをたす",
            Self::RemoveColumns => "れつをけす",
            Self::ShowKeys => "きーのいちらんをみる",
            Self::ShowAbout => "このあぷりについて",
        }
    }

    /// 英語の検索語（どの表示言語でも効く）。
    #[must_use]
    pub fn keywords(self) -> &'static [&'static str] {
        match self {
            Self::Top => &["top", "first", "start", "home"],
            Self::Bottom => &["bottom", "last", "end"],
            Self::GoTo => &["go to", "goto", "jump", "row", "line"],
            Self::SelectAll => &["select all", "all"],
            Self::SelectRows => &["select row", "whole row", "entire row"],
            Self::SelectColumns => &["select column", "whole column", "entire column"],
            Self::ToggleDetail => &["detail", "pane", "inspector", "toggle"],
            Self::ToggleReadOnly => &["read only", "readonly", "lock", "protect", "view only"],
            Self::ZoomIn => &["zoom in", "bigger", "larger", "font"],
            Self::ZoomOut => &["zoom out", "smaller", "font"],
            Self::ZoomReset => &["zoom reset", "actual size", "font"],
            Self::ChooseTheme => &["theme", "color", "colour", "scheme"],
            Self::ChooseDialect => &[
                "candidates",
                "dialect",
                "detect",
                "guess",
                "confidence",
                "reopen",
            ],
            Self::ChooseEncoding => &["encoding", "charset", "reopen", "utf", "sjis"],
            Self::ChooseDelimiter => &["delimiter", "separator", "reopen", "tab", "comma"],
            Self::ChooseQuoting => &["quote", "quoting", "escape", "reopen"],
            Self::ChooseHeader => &["header", "heading", "first row"],
            Self::MoveColumnsLeft => &["move column left", "column order"],
            Self::MoveColumnsRight => &["move column right", "column order"],
            Self::ResetColumns => &["reset columns", "column order", "original order"],
            Self::ResetWidths => &["reset widths", "column width", "autofit"],
            Self::FreezeColumns => &["freeze", "pin", "lock columns", "fixed column"],
            Self::UnfreezeColumns => &["unfreeze", "unpin", "unlock columns"],
            Self::Copy => &["copy", "clipboard"],
            Self::Cut => &["cut", "move", "clipboard"],
            Self::Find => &["find", "search", "kensaku"],
            Self::FindNext => &["find", "next", "search"],
            Self::FindPrevious => &["find", "previous", "back", "search"],
            Self::Replace => &["replace", "find", "okikae", "chikan"],
            Self::Save => &["save", "write"],
            Self::SaveAs => &["saveas", "save as", "export"],
            Self::NewTable => &["new", "create", "blank"],
            Self::Undo => &["undo"],
            Self::Redo => &["redo"],
            Self::InsertRowsAbove => &["insert row above", "add row", "new row"],
            Self::InsertRowsBelow => &["insert row below", "add row", "new row"],
            Self::AppendRow => &["append row", "add row at end", "new row"],
            Self::RemoveRows => &["delete row", "remove row"],
            Self::InsertColumnsLeft => &["insert column left", "add column", "new column"],
            Self::InsertColumnsRight => &["insert column right", "add column", "new column"],
            Self::RemoveColumns => &["delete column", "remove column"],
            Self::ShowKeys => &["keys", "shortcuts", "keyboard", "help"],
            Self::ShowAbout => &["about", "version", "license", "kono", "apuri"],
        }
    }

    /// キーの案内（半角・キーの名前は訳さない・`+` が続くと読み違えるので `Plus` と書く）。
    #[must_use]
    pub fn keys(self) -> Option<&'static str> {
        match self {
            Self::Top => Some("Ctrl+Home"),
            Self::Bottom => Some("Ctrl+End"),
            Self::GoTo => Some("Ctrl+G"),
            Self::SelectAll => Some("Ctrl+A"),
            Self::SelectRows => Some("Shift+Space"),
            Self::SelectColumns => Some("Ctrl+Space"),
            Self::ToggleDetail => Some("Ctrl+Shift+U"),
            Self::ZoomReset => Some("Ctrl+0"),
            Self::MoveColumnsLeft => Some("Ctrl+Shift+PageUp"),
            Self::MoveColumnsRight => Some("Ctrl+Shift+PageDown"),
            // 文字の大きさのキーは Ctrl+ホイール（キーの一覧の WheelZoom）と Ctrl+0（ADR 0020 の施主の判断）。
            Self::ToggleReadOnly
            | Self::ZoomIn
            | Self::ZoomOut
            | Self::ChooseTheme
            | Self::ChooseDialect
            | Self::ChooseEncoding
            | Self::ChooseDelimiter
            | Self::ChooseQuoting
            | Self::ChooseHeader
            | Self::ResetColumns
            | Self::ResetWidths
            | Self::FreezeColumns
            | Self::UnfreezeColumns
            | Self::ShowKeys
            | Self::ShowAbout
            | Self::InsertRowsBelow
            | Self::AppendRow
            | Self::InsertColumnsRight => None,
            // Ctrl++ と Ctrl+- は、列全体を選んでいれば列に、そうでなければ行に効く（ADR 0027 の決定 2）。
            Self::InsertRowsAbove | Self::InsertColumnsLeft => Some("Ctrl+Plus"),
            Self::RemoveRows | Self::RemoveColumns => Some("Ctrl+Minus"),
            Self::Copy => Some("Ctrl+C"),
            Self::Cut => Some("Ctrl+X"),
            Self::Find => Some("Ctrl+F"),
            Self::FindNext => Some("F3"),
            Self::FindPrevious => Some("Shift+F3"),
            Self::Replace => Some("Ctrl+H"),
            Self::Save => Some("Ctrl+S"),
            Self::SaveAs => Some("Ctrl+Shift+S"),
            Self::NewTable => Some("Ctrl+N"),
            Self::Undo => Some("Ctrl+Z"),
            Self::Redo => Some("Ctrl+Y / Ctrl+Shift+Z"),
        }
    }

    /// 2 段目で選ぶ読み方の面（読み方を選ぶ操作だけ）。
    #[must_use]
    pub fn reading_aspect(self) -> Option<ReadingAspect> {
        match self {
            Self::ChooseEncoding => Some(ReadingAspect::Encoding),
            Self::ChooseDelimiter => Some(ReadingAspect::Delimiter),
            Self::ChooseQuoting => Some(ReadingAspect::Quoting),
            Self::ChooseHeader => Some(ReadingAspect::Header),
            Self::Top
            | Self::Bottom
            | Self::GoTo
            | Self::SelectAll
            | Self::SelectRows
            | Self::SelectColumns
            | Self::ToggleDetail
            | Self::ToggleReadOnly
            | Self::ZoomIn
            | Self::ZoomOut
            | Self::ZoomReset
            | Self::ChooseTheme
            | Self::ChooseDialect
            | Self::MoveColumnsLeft
            | Self::MoveColumnsRight
            | Self::ResetColumns
            | Self::ResetWidths
            | Self::FreezeColumns
            | Self::UnfreezeColumns
            | Self::Copy
            | Self::Cut
            | Self::Find
            | Self::FindNext
            | Self::FindPrevious
            | Self::Replace
            | Self::Save
            | Self::SaveAs
            | Self::NewTable
            | Self::Undo
            | Self::Redo
            | Self::InsertRowsAbove
            | Self::InsertRowsBelow
            | Self::AppendRow
            | Self::RemoveRows
            | Self::InsertColumnsLeft
            | Self::InsertColumnsRight
            | Self::RemoveColumns
            | Self::ShowKeys
            | Self::ShowAbout => None,
        }
    }

    /// 実行する意図。配色と読み方を選ぶ操作は 2 段目を開くので `None`。
    #[must_use]
    pub fn intent(self) -> Option<Intent> {
        let moving = |step| Intent::Move {
            step,
            mode: SelectionMode::Collapse,
        };
        match self {
            Self::Top => Some(moving(CursorStep::Top)),
            Self::Bottom => Some(moving(CursorStep::Bottom)),
            Self::GoTo => Some(Intent::OpenGoTo),
            Self::SelectAll => Some(Intent::SelectAll),
            Self::SelectRows => Some(Intent::SelectRows),
            Self::SelectColumns => Some(Intent::SelectColumns),
            Self::ToggleDetail => Some(Intent::ToggleDetail),
            Self::ToggleReadOnly => Some(Intent::ToggleReadOnly),
            Self::ZoomIn => Some(Intent::Zoom(ZoomStep::In)),
            Self::ZoomOut => Some(Intent::Zoom(ZoomStep::Out)),
            Self::ZoomReset => Some(Intent::Zoom(ZoomStep::Reset)),
            Self::ChooseTheme
            | Self::ChooseDialect
            | Self::ChooseEncoding
            | Self::ChooseDelimiter
            | Self::ChooseQuoting
            | Self::ChooseHeader
            | Self::ShowKeys
            | Self::ShowAbout => None,
            Self::MoveColumnsLeft => Some(Intent::MoveColumns(ColumnShift::Left)),
            Self::MoveColumnsRight => Some(Intent::MoveColumns(ColumnShift::Right)),
            Self::ResetColumns => Some(Intent::ResetColumns),
            Self::ResetWidths => Some(Intent::ResetWidths),
            Self::FreezeColumns => Some(Intent::FreezeColumns),
            Self::UnfreezeColumns => Some(Intent::UnfreezeColumns),
            Self::Copy => Some(Intent::Copy),
            Self::Cut => Some(Intent::Cut),
            Self::Find => Some(Intent::OpenFind { replace: false }),
            Self::FindNext => Some(Intent::FindAgain { forward: true }),
            Self::FindPrevious => Some(Intent::FindAgain { forward: false }),
            Self::Replace => Some(Intent::OpenFind { replace: true }),
            Self::Save => Some(Intent::Save),
            Self::SaveAs => Some(Intent::SaveAs),
            Self::NewTable => Some(Intent::NewTable),
            Self::Undo => Some(Intent::Undo),
            Self::Redo => Some(Intent::Redo),
            Self::InsertRowsAbove => Some(Intent::InsertRows(RowPlace::Above)),
            Self::InsertRowsBelow => Some(Intent::InsertRows(RowPlace::Below)),
            Self::AppendRow => Some(Intent::InsertRows(RowPlace::End)),
            Self::RemoveRows => Some(Intent::RemoveRows),
            Self::InsertColumnsLeft => Some(Intent::InsertColumns(ColumnPlace::Left)),
            Self::InsertColumnsRight => Some(Intent::InsertColumns(ColumnPlace::Right)),
            Self::RemoveColumns => Some(Intent::RemoveColumns),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_command_has_a_name_a_reading_keywords_and_an_action() {
        for command in CommandId::ALL {
            assert!(!command.name(DetailPane::Open).is_empty());
            assert!(!command.reading().is_empty());
            assert!(!command.keywords().is_empty());
            // 2 段目を開く操作（「…」）は意図もキーの案内も持たない（キーの無い操作もある）。
            let opens_a_list = command == CommandId::ChooseTheme
                || command == CommandId::ChooseDialect
                || command == CommandId::ShowKeys
                || command == CommandId::ShowAbout
                || command.reading_aspect().is_some();
            assert_eq!(command.intent().is_none(), opens_a_list);
            assert!(!opens_a_list || command.keys().is_none());
            // 「…」は次に選ぶか打つものがある操作。行へ飛ぶは入力欄を開く意図を持つ。
            let asks_more = opens_a_list || command == CommandId::GoTo;
            assert_eq!(command.name(DetailPane::Open).ends_with('…'), asks_more);
        }
    }

    #[test]
    fn the_toggle_is_named_by_what_it_will_do() {
        assert_eq!(
            CommandId::ToggleDetail.name(DetailPane::Open),
            "詳細ペインを閉じる"
        );
        assert_eq!(
            CommandId::ToggleDetail.name(DetailPane::Closed),
            "詳細ペインを開く"
        );
        assert_eq!(
            CommandId::Top.intent(),
            Some(Intent::Move {
                step: CursorStep::Top,
                mode: SelectionMode::Collapse
            })
        );
        assert_eq!(
            CommandId::ZoomReset.intent(),
            Some(Intent::Zoom(ZoomStep::Reset))
        );
        assert_eq!(CommandId::Copy.keys(), Some("Ctrl+C"));
    }
}
