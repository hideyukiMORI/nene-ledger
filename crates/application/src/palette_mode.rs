//! パレットの段。

use crate::reading_aspect::ReadingAspect;
use crate::reading_choice::ReadingChoice;

/// パレットの段。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteMode {
    /// 操作の一覧。
    Commands,
    /// 飛ぶ先の行（と列）の番号を打つ段（Issue #54）。
    GoTo,
    /// 読み方の面 `aspect` を選ぶ段（Issue #52）。`current` は今の読み方（印を付け、Esc で戻す先）。
    Reading {
        /// 選ぶ面。
        aspect: ReadingAspect,
        /// 今の読み方。
        current: ReadingChoice,
    },
    /// キーの一覧（Issue #63）。
    Keys,
    /// 配色の一覧。`original` は開いたときの配色の、表の中の位置（Esc で戻す先）。
    Themes {
        /// 開いたときの配色の位置。
        original: usize,
    },
}
