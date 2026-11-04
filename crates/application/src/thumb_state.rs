//! つまみの見え方の状態。

/// つまみの見え方の状態。色と太さの両方で示す（色だけに頼らない・Issue #44）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThumbState {
    /// 普段（つまみの色・細い）。
    Normal,
    /// 指が乗っている（補助の字の色・太い）。
    Hovered,
    /// ドラッグ中・溝を押している（補助の字の色・太い）。
    Held,
}
