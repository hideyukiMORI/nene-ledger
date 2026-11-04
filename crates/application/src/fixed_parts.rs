//! 表の固定の部分の大きさ。

use nene_ledger_core::display_scale::DisplayScale;

/// 表の固定の部分の大きさ（画素）。見出しの行の高さ（無ければ 0）・本体の 1 行の高さ・行番号の列の幅。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedParts {
    header: u32,
    row: u32,
    gutter: u32,
    scale: DisplayScale,
}

impl FixedParts {
    /// 見出しの行の高さ `header`・1 行の高さ `row`・行番号の列の幅 `gutter`。
    #[must_use]
    pub fn new(header: u32, row: u32, gutter: u32) -> Self {
        Self {
            header,
            row,
            gutter,
            scale: DisplayScale::hundred(),
        }
    }

    /// 画面の倍率 `scale` で（スクロールバーの幅などの固定の画素に掛ける）。
    #[must_use]
    pub fn with_scale(self, scale: DisplayScale) -> Self {
        Self { scale, ..self }
    }

    /// 画面の倍率。
    #[must_use]
    pub fn scale(self) -> DisplayScale {
        self.scale
    }

    /// 見出しの行の高さ（無ければ 0）。
    #[must_use]
    pub fn header(self) -> u32 {
        self.header
    }

    /// 本体の 1 行の高さ。
    #[must_use]
    pub fn row(self) -> u32 {
        self.row
    }

    /// 行番号の列の幅。
    #[must_use]
    pub fn gutter(self) -> u32 {
        self.gutter
    }

    /// 高さ `height` の表に全部入る本体の行の数。
    #[must_use]
    pub fn rows_in(self, height: u32) -> u64 {
        u64::from(height.saturating_sub(self.header) / self.row.max(1))
    }

    /// 幅 `width` の表で、セルの列に使える幅。
    #[must_use]
    pub fn columns_in(self, width: u32) -> u64 {
        u64::from(width.saturating_sub(self.gutter))
    }
}
