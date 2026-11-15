//! 選んだ範囲の枠のドラッグの段（EDIT-17・GitHub Issue #498）。

use alloc::string::String;

use crate::cell_position::CellPosition;
use crate::selection::Selection;

/// 選んだ範囲の枠を掴んで値を動かす（写す）間の段。押してから、落とした値を貼り終えるまで順に進む。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FrameDrag {
    /// 何も掴んでいない。
    Idle,
    /// 枠を掴んでいる。掴んだときの指の下のセル（範囲の中に収めたもの。落とす先はここからの差で決める）。
    Held(CellPosition),
    /// 落とした。落とした先（範囲の左上）を、合成ルートが範囲の行を読んで `copy_text` に渡すまで持つ。
    Dropping(CellPosition),
    /// 範囲の値を読んだ。落とした先と、動かす（写す）値のタブ区切り（合成ルートが `take_drop` で取って貼る）。
    Dropped(CellPosition, String),
    /// 貼っている。落とす前の範囲と履歴の版（貼れなかったら範囲を戻す・`end_drop`）。
    Pasting(Selection, u64),
}

impl FrameDrag {
    /// 掴んだセル（掴んでいなければ `None`）。
    pub(crate) const fn grab(&self) -> Option<CellPosition> {
        match self {
            Self::Held(cell) => Some(*cell),
            Self::Idle | Self::Dropping(_) | Self::Dropped(..) | Self::Pasting(..) => None,
        }
    }

    /// 落とした先を取り出して、何も掴んでいない段へ戻す（落とした段でなければ `None` で、段は変えない）。
    pub(crate) fn take_dropping(&mut self) -> Option<CellPosition> {
        let Self::Dropping(target) = *self else {
            return None;
        };
        *self = Self::Idle;
        Some(target)
    }

    /// 読んだ値を取り出して、何も掴んでいない段へ戻す（値を読んだ段でなければ `None` で、段は変えない）。
    pub(crate) fn take_dropped(&mut self) -> Option<(CellPosition, String)> {
        if !matches!(self, Self::Dropped(..)) {
            return None;
        }
        match core::mem::replace(self, Self::Idle) {
            Self::Dropped(target, text) => Some((target, text)),
            Self::Idle | Self::Held(_) | Self::Dropping(_) | Self::Pasting(..) => None,
        }
    }

    /// 落とす前の範囲と版を取り出して、何も掴んでいない段へ戻す（貼っている段でなければ `None` で、段は変えない）。
    pub(crate) fn take_pasting(&mut self) -> Option<(Selection, u64)> {
        let Self::Pasting(range, revision) = *self else {
            return None;
        };
        *self = Self::Idle;
        Some((range, revision))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_held_frame_keeps_its_stage_when_another_stage_is_asked() {
        let cell = CellPosition::new(1, 2);
        let mut drag = FrameDrag::Held(cell);
        assert_eq!(drag.grab(), Some(cell));
        assert_eq!(drag.take_dropping(), None);
        assert_eq!(drag.take_dropped(), None);
        assert_eq!(drag.take_pasting(), None);
        assert_eq!(drag, FrameDrag::Held(cell));
    }

    #[test]
    fn each_stage_is_taken_once_and_returns_to_idle() {
        let cell = CellPosition::new(1, 2);
        let mut drag = FrameDrag::Dropping(cell);
        assert_eq!(drag.take_dropping(), Some(cell));
        assert_eq!(drag, FrameDrag::Idle);
        drag = FrameDrag::Dropped(cell, String::from("a"));
        assert_eq!(drag.take_dropped(), Some((cell, String::from("a"))));
        drag = FrameDrag::Pasting(Selection::at(cell), 3);
        assert_eq!(drag.take_pasting(), Some((Selection::at(cell), 3)));
        assert_eq!(drag.grab(), None);
    }
}
