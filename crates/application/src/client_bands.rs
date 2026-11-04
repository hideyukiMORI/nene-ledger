//! 中身の縦の配分。

use nene_ledger_core::glyph_metrics::GlyphMetrics;
use nene_ledger_core::table_layout::TableLayout;

use crate::detail_pane::DetailPane;
use crate::window_view::WindowView;

/// 詳細ペインの本文の行の数（多いとき）。
pub(crate) const DETAIL_LINES: u32 = 5;
/// 詳細ペインの本文の行の数の下限。1 行では値が 2 行以上あると「あと N 行」だけになり、値が見えない（Issue #39）。
pub(crate) const MIN_DETAIL_LINES: u32 = 2;
/// 詳細ペインの本文の上下の余白（画素・100% のとき）。
pub(crate) const DETAIL_PADDING: u32 = 6;
/// 詳細ペインと表の境の線の太さ（画素・100% のとき）。
pub(crate) const DETAIL_RULE: u32 = 1;

/// 中身の縦の配分。上から表・詳細ペイン（開いていれば）・状態の行。状態の行を先に取り、残りをペインと表で分ける。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ClientBands {
    table: u32,
    detail: u32,
    status: u32,
}

impl ClientBands {
    /// 中身の高さ `height` を分ける。状態の行は窓の部品の字、詳細ペインは中身の字の寸法で決める（ADR 0020）。
    pub(crate) fn split(height: u32, view: WindowView, pane: DetailPane) -> Self {
        let status = TableLayout::row_height_for(view.metrics()).min(height);
        let rest = height.saturating_sub(status);
        let detail = match pane {
            DetailPane::Open => Self::detail_height(view).min(rest),
            DetailPane::Closed => 0,
        };
        Self {
            table: rest.saturating_sub(detail),
            detail,
            status,
        }
    }

    /// 開いた詳細ペインの高さ。100% のとき（窓の部品の字で本文 `DETAIL_LINES` 行）の高さを目安にし、中身の字で
    /// その高さに入るだけの本文の行を置く（拡大するほど行の数が減る・Issue #38）。ただし本文は少なくとも
    /// `MIN_DETAIL_LINES` 行で、とても大きく拡大すると目安より少し高くなる（Issue #39）。
    pub(crate) fn detail_height(view: WindowView) -> u32 {
        let budget = Self::pane_height(view.metrics(), DETAIL_LINES);
        let content = view.content_metrics();
        Self::pane_height(content, Self::body_lines(budget, content))
    }

    /// 字の寸法 `metrics` で本文 `lines` 行の詳細ペインの高さ。表との境の線・見出しの行（表の行と同じ高さ・下の罫線を
    /// 含む）・本文の上下の余白・本文。
    fn pane_height(metrics: GlyphMetrics, lines: u32) -> u32 {
        Self::body_line(metrics)
            .saturating_mul(lines)
            .saturating_add(metrics.px(DETAIL_PADDING).saturating_mul(2))
            .saturating_add(TableLayout::row_height_for(metrics))
            .saturating_add(metrics.px(DETAIL_RULE))
    }

    /// 高さ `height` の詳細ペインに入る本文の行の数（少なくとも `MIN_DETAIL_LINES`・多くて `DETAIL_LINES`）。描画もこれで数える。
    pub(crate) fn body_lines(height: u32, metrics: GlyphMetrics) -> u32 {
        let fixed = metrics
            .px(DETAIL_PADDING)
            .saturating_mul(2)
            .saturating_add(TableLayout::row_height_for(metrics))
            .saturating_add(metrics.px(DETAIL_RULE));
        (height.saturating_sub(fixed) / Self::body_line(metrics).max(1))
            .clamp(MIN_DETAIL_LINES, DETAIL_LINES)
    }

    /// 詳細ペインの本文の 1 行の高さ（1.5 行。和文を折り返して読む行の高さ）。
    pub(crate) fn body_line(metrics: GlyphMetrics) -> u32 {
        metrics.line_height().saturating_mul(3).div_ceil(2)
    }

    /// 表の高さ。
    pub(crate) fn table(self) -> u32 {
        self.table
    }

    /// 詳細ペインの高さ（閉じていれば 0）。
    pub(crate) fn detail(self) -> u32 {
        self.detail
    }

    /// 状態の行の高さ。
    pub(crate) fn status(self) -> u32 {
        self.status
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metrics() -> GlyphMetrics {
        GlyphMetrics::new(8, 16, 12)
    }

    fn view() -> WindowView {
        use crate::window_view::WindowView;
        use nene_ledger_core::pixel_size::PixelSize;
        use nene_ledger_core::window_placement::WindowPlacement;
        WindowView::new(
            PixelSize::new(400, 400),
            metrics(),
            WindowPlacement::Normal,
            None,
        )
    }

    #[test]
    fn the_status_line_comes_first_then_the_pane_then_the_table() {
        // 本文の 1 行は 24。ペインは 24 × 5 ＋ 余白 6 × 2 ＋ 見出しの行 25 ＋ 線 1 = 158。状態の行は 25。
        let closed = ClientBands::split(400, view(), DetailPane::Closed);
        assert_eq!(
            (closed.table(), closed.detail(), closed.status()),
            (375, 0, 25)
        );
        let open = ClientBands::split(400, view(), DetailPane::Open);
        assert_eq!((open.table(), open.detail(), open.status()), (217, 158, 25));
        let tiny = ClientBands::split(100, view(), DetailPane::Open);
        assert_eq!((tiny.table(), tiny.detail(), tiny.status()), (0, 75, 25));
    }

    #[test]
    fn zooming_in_takes_lines_away_from_the_pane_but_never_adds() {
        // 100% のペインは 158（本文 5 行）。中身の字を大きくすると、158 に入るだけの本文の行に減る。
        let at = |line_height: u32| {
            let content = GlyphMetrics::new(line_height / 2, line_height, line_height * 3 / 4);
            let zoomed = view().with_content_metrics(content);
            let height = ClientBands::detail_height(zoomed);
            (ClientBands::body_lines(height, content), height)
        };
        assert_eq!(at(16), (5, 158));
        // 150%（行の高さ 24）: 見出しの行 37・本文の 1 行 36 で 3 行。175%（28）: 2 行。
        assert_eq!(at(24).0, 3);
        assert_eq!(at(28).0, 2);
        for line_height in [20, 24, 28, 32] {
            assert!(at(line_height).1 <= 158, "{line_height}");
        }
        // 300%（48）: 158 には 1 行しか入らないが、下限の 2 行にする（ペインは少し高くなる・Issue #39）。
        assert_eq!(at(48).0, 2);
        assert!(at(48).1 > 158);
        // 縮小しても 5 行より増やさない。
        assert_eq!(at(12).0, 5);
        assert!(at(12).1 < 158);
    }
}
