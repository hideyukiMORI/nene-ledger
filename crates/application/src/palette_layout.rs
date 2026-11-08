//! コマンドパレットの配置（物理画素）。描画とクリックの当たり判定が同じ配置を使う。

use nene_ledger_core::pixel_point::PixelPoint;
use nene_ledger_core::pixel_rect::PixelRect;
use nene_ledger_core::pixel_size::PixelSize;

use crate::command_palette::CommandPalette;
use crate::palette_mode::PaletteMode;
use crate::window_view::WindowView;

/// 幅の下限と上限（窓の幅の半分をこの間に収める）。
const MIN_WIDTH: u32 = 480;
const MAX_WIDTH: u32 = 640;
/// タイトルバーとの空き・窓の端との空き。
const MARGIN: u32 = 8;
/// 内側の余白。
const PADDING: u32 = 12;
/// 一度に見せる候補の数の上限。
const MAX_ROWS: u32 = 10;

/// コマンドパレットの配置。窓の上の中央に重ね、上に検索欄、配色の段なら試しの案内、その下に候補の行。
/// 字は窓の部品の大きさで、表の拡大には従わない（ADR 0020）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaletteLayout {
    panel: PixelRect,
    field: PixelRect,
    second: Option<PixelRect>,
    hint: Option<PixelRect>,
    list_top: u32,
    row_height: u32,
    first: usize,
    visible: usize,
}

impl PaletteLayout {
    /// 窓 `view` での、パレット `palette` の配置。選んでいる候補が見えるように、先頭の候補をずらす。
    #[must_use]
    pub fn new(view: WindowView, palette: &CommandPalette) -> Self {
        let metrics = view.metrics();
        let top = view
            .chrome()
            .title_bar()
            .size()
            .height()
            .saturating_add(metrics.px(MARGIN));
        let layout = Self::at(view, palette, top);
        if !palette.is_low() {
            return layout;
        }
        // 窓の下へ寄せる（下の端との空きは上と同じ・GitHub Issue #286）。窓より高ければ上のまま。
        let height = layout.panel.size().height();
        let low = view
            .size()
            .height()
            .saturating_sub(metrics.px(MARGIN))
            .saturating_sub(height);
        if low > top {
            layout.shifted(low.saturating_sub(top))
        } else {
            layout
        }
    }

    /// 縦に `down` 画素ずらした配置（候補の数は上に置いたときと同じ）。
    fn shifted(self, down: u32) -> Self {
        let move_rect = |area: PixelRect| {
            PixelRect::new(
                PixelPoint::new(area.origin().x(), area.origin().y().saturating_add(down)),
                area.size(),
            )
        };
        Self {
            panel: move_rect(self.panel),
            field: move_rect(self.field),
            second: self.second.map(move_rect),
            hint: self.hint.map(move_rect),
            list_top: self.list_top.saturating_add(down),
            ..self
        }
    }

    /// 上の端を `top` に置いた配置。
    fn at(view: WindowView, palette: &CommandPalette, top: u32) -> Self {
        let size = view.size();
        let metrics = view.metrics();
        let line = metrics.line_height();
        let (field_height, row_height) = (
            line.saturating_add(metrics.px(16)),
            line.saturating_add(metrics.px(8)),
        );
        let (margin, padding) = (metrics.px(MARGIN), metrics.px(PADDING));
        let width = (size.width() / 2)
            .clamp(metrics.px(MIN_WIDTH), metrics.px(MAX_WIDTH))
            .min(size.width().saturating_sub(margin.saturating_mul(2)));
        let left = size.width().saturating_sub(width) / 2;
        let inner_left = left.saturating_add(padding);
        let inner_width = width.saturating_sub(padding.saturating_mul(2));
        let field = rect(
            inner_left,
            top.saturating_add(padding),
            inner_width,
            field_height,
        );
        let (second, hint) = below_field(palette.mode(), field, row_height, padding);
        let list_top = second
            .or(hint)
            .map_or(bottom(field), bottom)
            .saturating_add(padding / 2);
        let room = size
            .height()
            .saturating_sub(list_top)
            .saturating_sub(margin.saturating_mul(6));
        let fit = (room / row_height.max(1)).clamp(1, MAX_ROWS);
        let count = u32::try_from(palette.items().len())
            .unwrap_or(u32::MAX)
            .max(least_rows(palette.mode()));
        let visible = usize::try_from(count.min(fit)).unwrap_or(1);
        let first = palette.selected().saturating_add(1).saturating_sub(visible);
        let list_height = row_height.saturating_mul(u32::try_from(visible).unwrap_or(1));
        let height = list_top
            .saturating_add(list_height)
            .saturating_add(padding)
            .saturating_sub(top);
        Self {
            panel: rect(left, top, width, height),
            field,
            second,
            hint,
            list_top,
            row_height,
            first,
            visible,
        }
    }

    /// パレットの面。
    #[must_use]
    pub fn panel(&self) -> PixelRect {
        self.panel
    }

    /// 検索欄。
    #[must_use]
    pub fn field(&self) -> PixelRect {
        self.field
    }

    /// 置換の段の 2 つ目の欄（置き換える文字）。ほかの段は `None`。
    #[must_use]
    pub fn second(&self) -> Option<PixelRect> {
        self.second
    }

    /// 窓の中の位置 `at` が、置換の段のどちらの欄か（`Some(true)` は置き換える欄）。欄の外なら `None`。
    #[must_use]
    pub fn field_at(&self, at: PixelPoint) -> Option<bool> {
        if contains(self.field, at) {
            Some(false)
        } else {
            self.second
                .filter(|&second| contains(second, at))
                .map(|_| true)
        }
    }

    /// 配色の段の、試しの案内の行。
    #[must_use]
    pub fn hint(&self) -> Option<PixelRect> {
        self.hint
    }

    /// 見せている最初の候補の番目。
    #[must_use]
    pub fn first(&self) -> usize {
        self.first
    }

    /// 見せている候補の数（候補が無ければ、その知らせの 1 行）。
    #[must_use]
    pub fn visible(&self) -> usize {
        self.visible
    }

    /// 見せている `slot` 行目（0 から）の矩形。
    #[must_use]
    pub fn row(&self, slot: usize) -> PixelRect {
        let offset = self
            .row_height
            .saturating_mul(u32::try_from(slot).unwrap_or(u32::MAX));
        rect(
            self.field.origin().x(),
            self.list_top.saturating_add(offset),
            self.field.size().width(),
            self.row_height,
        )
    }

    /// 窓の中の位置 `at` の候補の番目。候補の行の外なら `None`。
    #[must_use]
    pub fn item_at(&self, at: PixelPoint) -> Option<usize> {
        (0..self.visible)
            .find(|&slot| contains(self.row(slot), at))
            .map(|slot| self.first.saturating_add(slot))
    }

    /// 位置 `at` がパレットの面の中か。
    #[must_use]
    pub fn contains(&self, at: PixelPoint) -> bool {
        contains(self.panel, at)
    }
}

/// 欄の下に置く物：置換の段の 2 つ目の欄（置き換える文字・探す欄の下に同じ大きさで・GitHub Issue #287）と、試しの
/// 案内の行（配色・読み方・判別の候補）。
fn below_field(
    mode: PaletteMode,
    field: PixelRect,
    row_height: u32,
    padding: u32,
) -> (Option<PixelRect>, Option<PixelRect>) {
    let (left, width) = (field.origin().x(), field.size().width());
    let second = matches!(mode, PaletteMode::Replace { .. }).then(|| {
        rect(
            left,
            bottom(field).saturating_add(padding / 2),
            width,
            field.size().height(),
        )
    });
    let hint = mode
        .tries_the_selection()
        .then(|| rect(left, bottom(field), width, row_height));
    (second, hint)
}

fn rect(left: u32, top: u32, width: u32, height: u32) -> PixelRect {
    PixelRect::new(PixelPoint::new(left, top), PixelSize::new(width, height))
}

/// 候補が無くても取る行の数。知らせの 1 行、探す段は条件の 2 行、置換の段は置き換え方のキーと条件の 3 行（ADR 0030 の
/// 決定 3・6・GitHub Issue #199）。
fn least_rows(mode: PaletteMode) -> u32 {
    match mode {
        PaletteMode::Replace { .. } => 3,
        PaletteMode::Find => 2,
        PaletteMode::Commands
        | PaletteMode::GoTo
        | PaletteMode::Keys
        | PaletteMode::About
        | PaletteMode::Themes { .. }
        | PaletteMode::Reading { .. }
        | PaletteMode::Candidates { .. }
        | PaletteMode::SaveAs { .. } => 1,
    }
}

fn bottom(rect: PixelRect) -> u32 {
    rect.origin().y().saturating_add(rect.size().height())
}

fn contains(rect: PixelRect, at: PixelPoint) -> bool {
    let (left, top) = (rect.origin().x(), rect.origin().y());
    (left..left.saturating_add(rect.size().width())).contains(&at.x())
        && (top..top.saturating_add(rect.size().height())).contains(&at.y())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intent::Intent;
    use nene_ledger_core::find_options::FindOptions;
    use nene_ledger_core::glyph_metrics::GlyphMetrics;
    use nene_ledger_core::window_placement::WindowPlacement;

    fn view(width: u32, height: u32) -> WindowView {
        WindowView::new(
            PixelSize::new(width, height),
            GlyphMetrics::new(8, 20, 15),
            WindowPlacement::Normal,
            None,
        )
    }

    #[test]
    fn the_panel_sits_at_the_top_center_below_the_title_bar() {
        let layout = PaletteLayout::new(view(1024, 640), &CommandPalette::open());
        // 幅は窓の半分 512（480〜640）。タイトルバー 35 の下 8。検索欄 36・行 28。
        assert_eq!(layout.panel().origin(), PixelPoint::new(256, 43));
        assert_eq!(layout.panel().size().width(), 512);
        assert_eq!(layout.field(), rect(268, 55, 488, 36));
        assert_eq!(layout.hint(), None);
        assert_eq!(layout.visible(), 10);
        assert_eq!(layout.row(0), rect(268, 97, 488, 28));
    }

    #[test]
    fn clicks_find_the_row_or_the_panel() {
        let layout = PaletteLayout::new(view(1024, 640), &CommandPalette::open());
        assert_eq!(layout.item_at(PixelPoint::new(300, 97 + 28 + 3)), Some(1));
        assert_eq!(layout.item_at(PixelPoint::new(300, 60)), None);
        assert!(layout.contains(PixelPoint::new(300, 60)));
        assert!(!layout.contains(PixelPoint::new(10, 60)));
    }

    #[test]
    fn the_width_stays_between_its_bounds_and_inside_the_window() {
        // 狭い窓・広い窓では 480〜640 に収め、窓より広くしない。
        assert_eq!(
            PaletteLayout::new(view(1600, 640), &CommandPalette::open())
                .panel()
                .size()
                .width(),
            640
        );
        assert_eq!(
            PaletteLayout::new(view(400, 640), &CommandPalette::open())
                .panel()
                .size()
                .width(),
            384
        );
    }

    #[test]
    fn the_theme_list_scrolls_to_keep_the_selection_visible() {
        let mut palette = CommandPalette::open();
        for character in "theme".chars() {
            palette.handle(Intent::Type(character), 0);
        }
        palette.handle(Intent::Confirm, 50);
        let layout = PaletteLayout::new(view(1024, 640), &palette);
        assert!(layout.hint().is_some());
        assert_eq!(layout.visible(), 10);
        // 50 番目が最後の行に見える。
        assert_eq!(layout.first(), 41);
        assert_eq!(layout.item_at(layout.row(9).origin()), Some(50));
        // 低い窓では候補の数を減らす（少なくとも 1）。
        assert_eq!(PaletteLayout::new(view(1024, 200), &palette).visible(), 1);
    }

    #[test]
    fn a_lowered_panel_sits_at_the_bottom_with_the_same_rows() {
        let mut palette = CommandPalette::open_find("x", FindOptions::standard());
        let high = PaletteLayout::new(view(1024, 640), &palette);
        palette.set_low(true);
        let low = PaletteLayout::new(view(1024, 640), &palette);
        // 下の端との空きは上と同じ 8。大きさと候補の数は変えない。
        assert_eq!(bottom(low.panel()), 640 - 8);
        assert_eq!(low.panel().size(), high.panel().size());
        assert_eq!(low.visible(), high.visible());
        let down = low.panel().origin().y() - high.panel().origin().y();
        assert_eq!(low.field().origin().y(), high.field().origin().y() + down);
        assert_eq!(low.row(0).origin().y(), high.row(0).origin().y() + down);
        // 窓より高い箱は上のまま。
        let short = PaletteLayout::new(view(1024, 120), &palette);
        assert_eq!(short.panel().origin().y(), 43);
    }
}
