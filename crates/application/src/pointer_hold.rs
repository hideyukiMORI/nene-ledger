//! 左のボタンを押している間の状態。

use nene_ledger_core::pointer_point::PointerPoint;
use nene_ledger_core::scroll_axis::ScrollAxis;

use crate::cell_restore::CellRestore;
use crate::press_kind::PressKind;

use crate::scroll_orientation::ScrollOrientation;
use crate::track_side::TrackSide;

/// 左のボタンを押している間の状態（離すまで）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerHold {
    /// つまみをドラッグしている。`grab` はつまみの始まりから指まで。尺度 `axis` は押したときのもの
    /// （索引を作っている間に総数が変わっても、指の下でつまみが逃げない）。
    Thumb {
        /// 向き。
        orientation: ScrollOrientation,
        /// つまみの始まりから指まで。
        grab: i64,
        /// 押したときの尺度。
        axis: ScrollAxis,
    },
    /// つまみの外の溝を押している（押し続けると 1 画面ずつ繰り返す）。
    Track {
        /// 向き。
        orientation: ScrollOrientation,
        /// つまみのどちら側を押したか。
        side: TrackSide,
        /// 指の位置（溝の始まりから）。
        along: i64,
        /// 押してから数えた時計の刻み。
        ticks: u64,
    },
    /// 見出しの列の境目を押して、列の幅を変えている（Issue #64）。
    ColumnWidth {
        /// 幅を変える元の列。
        column: usize,
        /// 押した所。
        press: PointerPoint,
        /// 押したときの幅（半角の字数）。
        start: u32,
        /// 押す前に利用者が変えていた幅（Esc で戻す先・無ければ自動の幅）。
        original: Option<u32>,
    },
    /// 表と詳細ペインの境の線を押して、ペインの高さを変えている（GitHub Issue #452）。
    PaneHeight {
        /// 押した所の縦の位置（窓の中）。
        press_y: u32,
        /// 押したときのペインの本文の行の数。
        start: u32,
    },
    /// セル・行番号・見出しの上で押した（しきい値を超えて動かせば範囲を広げるドラッグ）。
    Cells {
        /// 何を広げるか。
        kind: PressKind,
        /// 押した所。
        press: PointerPoint,
        /// 今の指の位置。
        at: PointerPoint,
        /// しきい値を超えて動かしたか。
        dragging: bool,
        /// 押す前の選択と表示（Esc で戻す）。
        before: CellRestore,
        /// ドラッグしてから数えた時計の刻み（横の自動スクロールの間隔）。
        ticks: u64,
    },
}

impl PointerHold {
    /// スクロールバーを押しているなら、その向き。
    #[must_use]
    pub fn orientation(self) -> Option<ScrollOrientation> {
        match self {
            Self::Thumb { orientation, .. } | Self::Track { orientation, .. } => Some(orientation),
            Self::Cells { .. } | Self::ColumnWidth { .. } | Self::PaneHeight { .. } => None,
        }
    }

    /// セルの上でドラッグしているか（しきい値を超えて動かした）。
    #[must_use]
    pub fn dragging_cells(self) -> bool {
        match self {
            Self::Cells { dragging, .. } => dragging,
            Self::Thumb { .. }
            | Self::Track { .. }
            | Self::ColumnWidth { .. }
            | Self::PaneHeight { .. } => false,
        }
    }
}
