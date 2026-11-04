//! 窓の飾り（タイトルバー・ボタン・縁）の配置と当たり判定（物理画素）。

use crate::chrome_button::ChromeButton;
use crate::chrome_hit::ChromeHit;
use crate::display_scale::DisplayScale;
use crate::edge_band::{Band, band};
use crate::glyph_metrics::GlyphMetrics;
use crate::pixel_point::PixelPoint;
use crate::pixel_rect::PixelRect;
use crate::pixel_size::PixelSize;
use crate::resize_edge::ResizeEdge;
use crate::window_placement::WindowPlacement;

/// 見えない縁の幅（物理画素）。ここを押すとリサイズする。🔴 画面ごとの倍率（WIN-09）を入れるまでの仮。
pub const RESIZE_EDGE: u32 = 6;
/// 角として当たる長さ（物理画素）。辺の端からこの長さまでは角のリサイズにする。
pub const RESIZE_CORNER: u32 = 16;
/// 窓の縁の線の太さ（物理画素）。最大化している間は引かない。
pub const BORDER_LINE: u32 = 1;

/// 窓の飾りの配置。上にタイトルバー（右端に最小化・最大化・閉じる）、その下が中身。縁は中身の内側に重なる見えない帯（ADR 0013）。
///
/// 移動・リサイズ・最大化そのものは OS が行う。ここは「どこが何に当たるか」だけを決める（ARC-011）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowChrome {
    size: PixelSize,
    title_height: u32,
    button_width: u32,
    placement: WindowPlacement,
    scale: DisplayScale,
}

impl WindowChrome {
    /// 窓の大きさ `size`・字の寸法・置かれ方から配置を決める。タイトルバーの高さは 1 行の 1.75 倍、ボタンの幅はその 1.5 倍。
    #[must_use]
    pub fn new(size: PixelSize, metrics: GlyphMetrics, placement: WindowPlacement) -> Self {
        let title_height = metrics
            .line_height()
            .saturating_mul(7)
            .div_ceil(4)
            .min(size.height());
        Self {
            size,
            title_height,
            button_width: title_height.saturating_mul(3).div_ceil(2),
            placement,
            scale: metrics.scale(),
        }
    }

    /// 窓の大きさ。
    #[must_use]
    pub fn size(self) -> PixelSize {
        self.size
    }

    /// 置かれ方。
    #[must_use]
    pub fn placement(self) -> WindowPlacement {
        self.placement
    }

    /// 縁の線の太さ。最大化している間は 0。
    #[must_use]
    pub fn border(self) -> u32 {
        match self.placement {
            WindowPlacement::Normal => self.scale.px(BORDER_LINE),
            WindowPlacement::Maximized => 0,
        }
    }

    /// タイトルバー（窓の幅いっぱい）。
    #[must_use]
    pub fn title_bar(self) -> PixelRect {
        PixelRect::new(
            PixelPoint::new(0, 0),
            PixelSize::new(self.size.width(), self.title_height),
        )
    }

    /// 題名を書く所（タイトルバーの、ボタンより左）。
    #[must_use]
    pub fn caption(self) -> PixelRect {
        let left_of_buttons = self.button(ChromeButton::Minimize).origin().x();
        PixelRect::new(
            PixelPoint::new(0, 0),
            PixelSize::new(left_of_buttons, self.title_height),
        )
    }

    /// ボタンの矩形。右の縁の線の内側から、閉じる・最大化・最小化の順に左へ並ぶ。窓が狭ければ左端で重なる。
    #[must_use]
    pub fn button(self, button: ChromeButton) -> PixelRect {
        let from_right = match button {
            ChromeButton::Minimize => 3_u32,
            ChromeButton::Maximize => 2_u32,
            ChromeButton::Close => 1_u32,
        };
        let right = self.size.width().saturating_sub(self.border());
        let left = right.saturating_sub(self.button_width.saturating_mul(from_right));
        PixelRect::new(
            PixelPoint::new(left, 0),
            PixelSize::new(self.button_width.min(right), self.title_height),
        )
    }

    /// 中身（表と状態の行）を置く所。タイトルバーの下で、左右と下の縁の線の内側。
    #[must_use]
    pub fn client(self) -> PixelRect {
        let border = self.border();
        PixelRect::new(
            PixelPoint::new(border, self.title_height),
            PixelSize::new(
                self.size.width().saturating_sub(border.saturating_mul(2)),
                self.size
                    .height()
                    .saturating_sub(self.title_height)
                    .saturating_sub(border),
            ),
        )
    }

    /// 縁の線（上・右・下・左の細い矩形）。最大化している間は無い。
    #[must_use]
    pub fn border_lines(self) -> [PixelRect; 4] {
        let (width, height, line) = (self.size.width(), self.size.height(), self.border());
        let bottom = height.saturating_sub(line);
        let right = width.saturating_sub(line);
        [
            PixelRect::new(PixelPoint::new(0, 0), PixelSize::new(width, line)),
            PixelRect::new(PixelPoint::new(right, 0), PixelSize::new(line, height)),
            PixelRect::new(PixelPoint::new(0, bottom), PixelSize::new(width, line)),
            PixelRect::new(PixelPoint::new(0, 0), PixelSize::new(line, height)),
        ]
    }

    /// `point` が当たる所。縁（通常のときだけ）・ボタン・タイトルバー・中身の順に見る。
    #[must_use]
    pub fn hit(self, point: PixelPoint) -> ChromeHit {
        if let Some(edge) = self.edge(point) {
            return ChromeHit::Edge(edge);
        }
        if point.y() >= self.title_height {
            return ChromeHit::Client;
        }
        ChromeButton::ALL
            .into_iter()
            .rev()
            .find(|&button| contains(self.button(button), point))
            .map_or(ChromeHit::Caption, ChromeHit::Button)
    }

    /// `point` が見えない縁に当たれば、その向き。辺の端から `RESIZE_CORNER` までは角にする。
    fn edge(self, point: PixelPoint) -> Option<ResizeEdge> {
        match self.placement {
            WindowPlacement::Normal => {}
            WindowPlacement::Maximized => return None,
        }
        let (width, height) = (self.size.width(), self.size.height());
        let (edge, corner) = (self.scale.px(RESIZE_EDGE), self.scale.px(RESIZE_CORNER));
        let on_row = band(point.y(), height, edge);
        let on_column = band(point.x(), width, edge);
        let row = on_row.or_else(|| on_column.and(band(point.y(), height, corner)));
        let column = on_column.or_else(|| on_row.and(band(point.x(), width, corner)));
        match (row, column) {
            (Some(Band::Start), Some(Band::Start)) => Some(ResizeEdge::TopLeft),
            (Some(Band::Start), None) => Some(ResizeEdge::Top),
            (Some(Band::Start), Some(Band::End)) => Some(ResizeEdge::TopRight),
            (None, Some(Band::End)) => Some(ResizeEdge::Right),
            (Some(Band::End), Some(Band::End)) => Some(ResizeEdge::BottomRight),
            (Some(Band::End), None) => Some(ResizeEdge::Bottom),
            (Some(Band::End), Some(Band::Start)) => Some(ResizeEdge::BottomLeft),
            (None, Some(Band::Start)) => Some(ResizeEdge::Left),
            (None, None) => None,
        }
    }
}

/// `rect` が `point` を含むか（右と下の端は含まない）。
fn contains(rect: PixelRect, point: PixelPoint) -> bool {
    let (left, top) = (rect.origin().x(), rect.origin().y());
    (left..left.saturating_add(rect.size().width())).contains(&point.x())
        && (top..top.saturating_add(rect.size().height())).contains(&point.y())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metrics() -> GlyphMetrics {
        GlyphMetrics::new(8, 16, 12)
    }

    fn chrome(placement: WindowPlacement) -> WindowChrome {
        WindowChrome::new(PixelSize::new(800, 600), metrics(), placement)
    }

    fn hit(chrome: WindowChrome, x: u32, y: u32) -> ChromeHit {
        chrome.hit(PixelPoint::new(x, y))
    }

    #[test]
    fn the_title_bar_and_buttons_line_up_at_the_top_right() {
        let normal = chrome(WindowPlacement::Normal);
        // 高さは 1 行 16 の 1.75 倍で 28、ボタンの幅は 42。右の縁の線 1 の内側から並ぶ。
        assert_eq!(normal.title_bar().size(), PixelSize::new(800, 28));
        let close = normal.button(ChromeButton::Close);
        assert_eq!(close.origin(), PixelPoint::new(800 - 1 - 42, 0));
        assert_eq!(close.size(), PixelSize::new(42, 28));
        assert_eq!(
            normal.button(ChromeButton::Maximize).origin().x(),
            800 - 1 - 84
        );
        assert_eq!(
            normal.button(ChromeButton::Minimize).origin().x(),
            800 - 1 - 126
        );
        assert_eq!(normal.caption().size(), PixelSize::new(800 - 1 - 126, 28));
        let maximized = chrome(WindowPlacement::Maximized);
        assert_eq!(maximized.button(ChromeButton::Close).origin().x(), 800 - 42);
        assert_eq!(maximized.placement(), WindowPlacement::Maximized);
        assert_eq!(maximized.size(), PixelSize::new(800, 600));
    }

    #[test]
    fn the_client_sits_below_the_title_bar_inside_the_border() {
        let normal = chrome(WindowPlacement::Normal);
        assert_eq!(normal.border(), 1);
        assert_eq!(
            normal.client(),
            PixelRect::new(PixelPoint::new(1, 28), PixelSize::new(798, 571))
        );
        let lines = normal.border_lines();
        assert_eq!(
            lines[0],
            PixelRect::new(PixelPoint::new(0, 0), PixelSize::new(800, 1))
        );
        assert_eq!(
            lines[1],
            PixelRect::new(PixelPoint::new(799, 0), PixelSize::new(1, 600))
        );
        assert_eq!(
            lines[2],
            PixelRect::new(PixelPoint::new(0, 599), PixelSize::new(800, 1))
        );
        assert_eq!(
            lines[3],
            PixelRect::new(PixelPoint::new(0, 0), PixelSize::new(1, 600))
        );
        let maximized = chrome(WindowPlacement::Maximized);
        assert_eq!(maximized.border(), 0);
        assert_eq!(
            maximized.client(),
            PixelRect::new(PixelPoint::new(0, 28), PixelSize::new(800, 572))
        );
        assert!(
            maximized
                .border_lines()
                .iter()
                .all(|line| line.size().width() == 0 || line.size().height() == 0)
        );
    }

    #[test]
    fn buttons_caption_and_client_are_hit() {
        let normal = chrome(WindowPlacement::Normal);
        assert_eq!(hit(normal, 780, 14), ChromeHit::Button(ChromeButton::Close));
        assert_eq!(
            hit(normal, 740, 14),
            ChromeHit::Button(ChromeButton::Maximize)
        );
        assert_eq!(
            hit(normal, 700, 14),
            ChromeHit::Button(ChromeButton::Minimize)
        );
        assert_eq!(hit(normal, 300, 14), ChromeHit::Caption);
        assert_eq!(hit(normal, 300, 27), ChromeHit::Caption);
        assert_eq!(hit(normal, 300, 28), ChromeHit::Client);
        assert_eq!(hit(normal, 400, 300), ChromeHit::Client);
    }

    #[test]
    fn the_invisible_edges_resize_along_the_sides() {
        let normal = chrome(WindowPlacement::Normal);
        assert_eq!(hit(normal, 300, 5), ChromeHit::Edge(ResizeEdge::Top));
        assert_eq!(hit(normal, 300, 6), ChromeHit::Caption);
        assert_eq!(hit(normal, 799, 300), ChromeHit::Edge(ResizeEdge::Right));
        assert_eq!(hit(normal, 794, 300), ChromeHit::Edge(ResizeEdge::Right));
        assert_eq!(hit(normal, 793, 300), ChromeHit::Client);
        assert_eq!(hit(normal, 300, 599), ChromeHit::Edge(ResizeEdge::Bottom));
        assert_eq!(hit(normal, 2, 300), ChromeHit::Edge(ResizeEdge::Left));
    }

    #[test]
    fn the_invisible_edges_resize_at_the_corners() {
        let normal = chrome(WindowPlacement::Normal);
        assert_eq!(hit(normal, 0, 0), ChromeHit::Edge(ResizeEdge::TopLeft));
        // 角は辺の端から 16 まで。閉じるボタンの上の端でも縁が勝つ。
        assert_eq!(hit(normal, 785, 2), ChromeHit::Edge(ResizeEdge::TopRight));
        assert_eq!(hit(normal, 783, 2), ChromeHit::Edge(ResizeEdge::Top));
        assert_eq!(
            hit(normal, 799, 599),
            ChromeHit::Edge(ResizeEdge::BottomRight)
        );
        assert_eq!(
            hit(normal, 10, 597),
            ChromeHit::Edge(ResizeEdge::BottomLeft)
        );
        assert_eq!(hit(normal, 2, 590), ChromeHit::Edge(ResizeEdge::BottomLeft));
        assert_eq!(hit(normal, 2, 10), ChromeHit::Edge(ResizeEdge::TopLeft));
    }

    #[test]
    fn a_maximized_window_has_no_edges() {
        let maximized = chrome(WindowPlacement::Maximized);
        assert_eq!(hit(maximized, 0, 0), ChromeHit::Caption);
        assert_eq!(
            hit(maximized, 799, 2),
            ChromeHit::Button(ChromeButton::Close)
        );
        assert_eq!(hit(maximized, 0, 300), ChromeHit::Client);
        assert_eq!(hit(maximized, 400, 599), ChromeHit::Client);
    }

    #[test]
    fn a_tiny_window_stays_consistent() {
        let tiny = WindowChrome::new(PixelSize::new(50, 10), metrics(), WindowPlacement::Normal);
        // タイトルバーは窓の高さまで（10）。ボタンの幅はその 1.5 倍の 15。
        assert_eq!(tiny.title_bar().size(), PixelSize::new(50, 10));
        assert_eq!(tiny.button(ChromeButton::Minimize).origin().x(), 49 - 45);
        assert_eq!(tiny.client().size(), PixelSize::new(48, 0));
        assert_eq!(hit(tiny, 3, 3), ChromeHit::Edge(ResizeEdge::TopLeft));
        // 3 つのボタンが入らない幅では左端で重なり、右に並ぶボタンが勝つ。
        let narrow = WindowChrome::new(
            PixelSize::new(60, 300),
            metrics(),
            WindowPlacement::Maximized,
        );
        assert_eq!(narrow.button(ChromeButton::Minimize).origin().x(), 0);
        assert_eq!(
            hit(narrow, 5, 14),
            ChromeHit::Button(ChromeButton::Maximize)
        );
        assert_eq!(hit(narrow, 20, 14), ChromeHit::Button(ChromeButton::Close));
    }

    #[test]
    fn the_border_and_the_edges_grow_with_the_display_scale() {
        let scale = DisplayScale::of_percent(150);
        let metrics = GlyphMetrics::new(12, 24, 18).with_scale(scale);
        let normal = WindowChrome::new(PixelSize::new(800, 600), metrics, WindowPlacement::Normal);
        // 縁の線 1 は 2、リサイズの帯 6 は 9、角 16 は 24。タイトルバーは字の 1 行 24 の 1.75 倍で 42。
        assert_eq!(normal.border(), 2);
        assert_eq!(normal.title_bar().size(), PixelSize::new(800, 42));
        assert_eq!(hit(normal, 8, 300), ChromeHit::Edge(ResizeEdge::Left));
        assert_eq!(hit(normal, 10, 300), ChromeHit::Client);
        assert_eq!(hit(normal, 2, 20), ChromeHit::Edge(ResizeEdge::TopLeft));
        assert_eq!(
            hit(chrome(WindowPlacement::Normal), 2, 20),
            ChromeHit::Edge(ResizeEdge::Left)
        );
    }
}
