//! スクロールバーの上の、押した所。

use crate::track_side::TrackSide;

/// スクロールバーの上の、押した所。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarHit {
    /// つまみ。`grab` はつまみの始まりから指までの長さ（ドラッグの間、指とつまみの間を保つ）。
    Thumb {
        /// つまみの始まりから指まで。
        grab: i64,
    },
    /// つまみの外の溝。
    Track(TrackSide),
}
