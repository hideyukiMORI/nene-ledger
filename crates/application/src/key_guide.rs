//! パレットに無いキーの操作（キーの一覧に出すだけ・Issue #63）。

/// パレットの操作に無い、キーとマウスの操作。閉じた集合で、キーの一覧の後半に固定の順で出す。
/// キーのあるパレットの操作は `CommandId::keys` が正本で、ここには書かない。
/// 🔴 各 OS の `key_intent` のテストが、行ごとに割り当てと食い違わないことを確かめる。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyGuide {
    /// 矢印でカーソルを動かす。
    MoveCursor,
    /// Shift+矢印で選択を広げる。
    ExtendSelection,
    /// `PageUp` / `PageDown` で 1 画面ぶん動かす。
    Page,
    /// Home / End で行の端へ。
    RowEnds,
    /// Shift+クリックで選択を広げる。
    ExtendClick,
    /// Ctrl+クリックで離れた範囲を足す。
    AddRange,
    /// Shift+ホイールで横に動かす。
    Sideways,
    /// Ctrl+ホイールで文字の大きさを変える。
    WheelZoom,
    /// 次の配色を試す。
    NextTheme,
    /// 前の配色を試す。
    PreviousTheme,
    /// コマンドパレットを開く。
    Palette,
    /// やめる・戻す。
    Cancel,
    /// カーソルのセルを続きから直す（ADR 0026）。
    EditCell,
    /// カーソルのセルを空にする。
    ClearCell,
    /// セルの中の改行（EDIT-10）。
    BreakLine,
    /// 確定して右・左・上へ動く。
    Advance,
}

impl KeyGuide {
    /// すべて（一覧の順）。
    pub const ALL: [Self; 16] = [
        Self::MoveCursor,
        Self::ExtendSelection,
        Self::Page,
        Self::RowEnds,
        Self::ExtendClick,
        Self::AddRange,
        Self::Sideways,
        Self::WheelZoom,
        Self::NextTheme,
        Self::PreviousTheme,
        Self::Palette,
        Self::Cancel,
        Self::EditCell,
        Self::ClearCell,
        Self::BreakLine,
        Self::Advance,
    ];

    /// 表示名。🔴 表示言語の文言の表（ADR 0011）を入れるまでの仮の日本語。
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::MoveCursor => "カーソルを動かす",
            Self::ExtendSelection => "選択を広げる",
            Self::Page => "1 画面ぶん動かす",
            Self::RowEnds => "行の端へ移る",
            Self::ExtendClick => "押したセルまで選択を広げる",
            Self::AddRange => "離れた範囲を足す",
            Self::Sideways => "横に動かす",
            Self::WheelZoom => "文字の大きさを変える",
            Self::NextTheme => "次の配色を試す",
            Self::PreviousTheme => "前の配色を試す",
            Self::Palette => "コマンドパレットを開く",
            Self::Cancel => "やめる・元に戻す",
            Self::EditCell => "セルを続きから直す（入力と編集を替える）",
            Self::ClearCell => "セルを空にする",
            Self::BreakLine => "セルの中で改行する",
            Self::Advance => "確定して右・左・上へ動く",
        }
    }

    /// 名前の読み（ひらがな）。ローマ字はここから機械で作る。
    #[must_use]
    pub fn reading(self) -> &'static str {
        match self {
            Self::MoveCursor => "かーそるをうごかす",
            Self::ExtendSelection => "せんたくをひろげる",
            Self::Page => "いちがめんぶんうごかす",
            Self::RowEnds => "ぎょうのはしへうつる",
            Self::ExtendClick => "おしたせるまでせんたくをひろげる",
            Self::AddRange => "はなれたはんいをたす",
            Self::Sideways => "よこにうごかす",
            Self::WheelZoom => "もじのおおきさをかえる",
            Self::NextTheme => "つぎのはいしょくをためす",
            Self::PreviousTheme => "まえのはいしょくをためす",
            Self::Palette => "こまんどぱれっとをひらく",
            Self::Cancel => "やめるもとにもどす",
            Self::EditCell => "せるをつづきからなおす",
            Self::ClearCell => "せるをからにする",
            Self::BreakLine => "せるのなかでかいぎょうする",
            Self::Advance => "かくていしてうごく",
        }
    }

    /// キーの案内（`CommandId::keys` と同じ書き方・半角・キーの名前は訳さない）。
    #[must_use]
    pub fn keys(self) -> &'static str {
        match self {
            Self::MoveCursor => "Arrow keys",
            Self::ExtendSelection => "Shift+Arrow keys",
            Self::Page => "PageUp / PageDown",
            Self::RowEnds => "Home / End",
            Self::ExtendClick => "Shift+Click",
            Self::AddRange => "Ctrl+Click",
            Self::Sideways => "Shift+Wheel",
            Self::WheelZoom => "Ctrl+Wheel",
            Self::NextTheme => "Ctrl+Shift+T",
            Self::PreviousTheme => "Ctrl+Shift+Alt+T",
            Self::Palette => "Ctrl+Shift+P / F1",
            Self::Cancel => "Esc",
            Self::EditCell => "F2",
            Self::ClearCell => "Delete",
            Self::BreakLine => "Alt+Enter",
            Self::Advance => "Tab / Shift+Tab / Shift+Enter",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command_id::CommandId;

    #[test]
    fn every_guide_has_a_name_a_reading_and_keys_unlike_any_command() {
        for guide in KeyGuide::ALL {
            assert!(!guide.name().is_empty());
            assert!(!guide.reading().is_empty());
            assert!(!guide.keys().is_empty());
            // パレットの操作のキーと重ねて書かない。
            assert!(
                CommandId::ALL
                    .iter()
                    .all(|command| command.keys() != Some(guide.keys()))
            );
        }
    }
}
