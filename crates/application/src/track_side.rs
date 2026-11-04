//! 溝のどちら側か。

/// 溝の、つまみのどちら側か。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrackSide {
    /// つまみより前（上・左）。
    Before,
    /// つまみより後ろ（下・右）。
    After,
}
