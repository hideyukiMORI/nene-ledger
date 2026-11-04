//! マウスのイベントの、画面の位置と時刻。

/// マウスのイベントの、画面（ルートの窓）の位置と X の時刻。ウィンドウマネージャーへの頼みとダブルクリックの判定に使う。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PointerAt {
    root_x: i16,
    root_y: i16,
    time: u32,
}

impl PointerAt {
    pub(crate) fn new(root_x: i16, root_y: i16, time: u32) -> Self {
        Self {
            root_x,
            root_y,
            time,
        }
    }

    pub(crate) fn root_x(self) -> i16 {
        self.root_x
    }

    pub(crate) fn root_y(self) -> i16 {
        self.root_y
    }

    pub(crate) fn time(self) -> u32 {
        self.time
    }

    /// `other` から縦か横に `threshold` より大きく離れているか。
    pub(crate) fn is_far_from(self, other: Self, threshold: i32) -> bool {
        (i32::from(self.root_x) - i32::from(other.root_x)).abs() > threshold
            || (i32::from(self.root_y) - i32::from(other.root_y)).abs() > threshold
    }
}
