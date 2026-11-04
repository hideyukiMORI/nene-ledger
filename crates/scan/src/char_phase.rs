//! 文字の途中かどうか。

/// 文字の単位で進む切り出しの、文字の途中かどうか（ADR 0019）。塊の境目を越えて持ち越す。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CharPhase {
    /// 次のバイトは文字の頭。
    Head,
    /// `Shift_JIS` の 1 バイト目を見た。次のバイトが 2 バイト目として正しければ、同じ文字の続き。
    Trail,
    /// UTF-16 の単位の 1 バイト目（中身）を受け取った。次のバイトと組んで単位の値になる。
    HalfUnit(u8),
}
