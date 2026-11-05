//! パレットから実行できる操作。

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
    /// 書き換えを保存する（ADR 0026 の決定 6）。
    Save,
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
    /// キーの一覧を見る（2 段目・Issue #63）。
    ShowKeys,
}

impl CommandId {
    /// すべての操作（何も打たないときの順）。
    pub const ALL: [Self; 31] = [
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
        Self::Save,
        Self::Undo,
        Self::Redo,
        Self::InsertRowsAbove,
        Self::InsertRowsBelow,
        Self::AppendRow,
        Self::RemoveRows,
        Self::ShowKeys,
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
            Self::Save => "直したセルを保存する",
            Self::Undo => "元に戻す",
            Self::Redo => "やり直す",
            Self::InsertRowsAbove => "上に行を足す",
            Self::InsertRowsBelow => "下に行を足す",
            Self::AppendRow => "末尾に行を足す",
            Self::RemoveRows => "行を消す",
            Self::ShowKeys => "キーの一覧を見る…",
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
            Self::Save => "なおしたせるをほぞんする",
            Self::Undo => "もとにもどす",
            Self::Redo => "やりなおす",
            Self::InsertRowsAbove => "うえにぎょうをたす",
            Self::InsertRowsBelow => "したにぎょうをたす",
            Self::AppendRow => "まつびにぎょうをたす",
            Self::RemoveRows => "ぎょうをけす",
            Self::ShowKeys => "きーのいちらんをみる",
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
            Self::Save => &["save", "write"],
            Self::Undo => &["undo"],
            Self::Redo => &["redo"],
            Self::InsertRowsAbove => &["insert row above", "add row", "new row"],
            Self::InsertRowsBelow => &["insert row below", "add row", "new row"],
            Self::AppendRow => &["append row", "add row at end", "new row"],
            Self::RemoveRows => &["delete row", "remove row"],
            Self::ShowKeys => &["keys", "shortcuts", "keyboard", "help"],
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
            Self::ZoomIn => Some("Ctrl+Plus"),
            Self::ZoomOut => Some("Ctrl+Minus"),
            Self::ZoomReset => Some("Ctrl+0"),
            Self::MoveColumnsLeft => Some("Ctrl+Shift+PageUp"),
            Self::MoveColumnsRight => Some("Ctrl+Shift+PageDown"),
            Self::ChooseTheme
            | Self::ChooseDialect
            | Self::ChooseEncoding
            | Self::ChooseDelimiter
            | Self::ChooseQuoting
            | Self::ChooseHeader
            | Self::ResetColumns
            | Self::ResetWidths
            | Self::FreezeColumns
            | Self::UnfreezeColumns
            | Self::ShowKeys => None,
            // 🔴 ADR 0027 の Ctrl++ と Ctrl+- は文字の大きさ（ADR 0020）とぶつかるので、キーは施主に諮ってから。
            Self::InsertRowsAbove | Self::InsertRowsBelow | Self::AppendRow | Self::RemoveRows => {
                None
            }
            Self::Copy => Some("Ctrl+C"),
            Self::Save => Some("Ctrl+S"),
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
            | Self::Save
            | Self::Undo
            | Self::Redo
            | Self::InsertRowsAbove
            | Self::InsertRowsBelow
            | Self::AppendRow
            | Self::RemoveRows
            | Self::ShowKeys => None,
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
            Self::ZoomIn => Some(Intent::Zoom(ZoomStep::In)),
            Self::ZoomOut => Some(Intent::Zoom(ZoomStep::Out)),
            Self::ZoomReset => Some(Intent::Zoom(ZoomStep::Reset)),
            Self::ChooseTheme
            | Self::ChooseDialect
            | Self::ChooseEncoding
            | Self::ChooseDelimiter
            | Self::ChooseQuoting
            | Self::ChooseHeader
            | Self::ShowKeys => None,
            Self::MoveColumnsLeft => Some(Intent::MoveColumns(ColumnShift::Left)),
            Self::MoveColumnsRight => Some(Intent::MoveColumns(ColumnShift::Right)),
            Self::ResetColumns => Some(Intent::ResetColumns),
            Self::ResetWidths => Some(Intent::ResetWidths),
            Self::FreezeColumns => Some(Intent::FreezeColumns),
            Self::UnfreezeColumns => Some(Intent::UnfreezeColumns),
            Self::Copy => Some(Intent::Copy),
            Self::Save => Some(Intent::Save),
            Self::Undo => Some(Intent::Undo),
            Self::Redo => Some(Intent::Redo),
            Self::InsertRowsAbove => Some(Intent::InsertRows(RowPlace::Above)),
            Self::InsertRowsBelow => Some(Intent::InsertRows(RowPlace::Below)),
            Self::AppendRow => Some(Intent::InsertRows(RowPlace::End)),
            Self::RemoveRows => Some(Intent::RemoveRows),
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
