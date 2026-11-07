//! 未保存のまま閉じるときに尋ねる画面の配置・当たり判定・描画（ADR 0026 の決定 7・ARC-011）。
//!
//! 窓の真ん中に、問いの 1 行と 3 つのボタンを並べた箱を置く。UI は描くだけで、押した位置は意図（`Intent::Point`）で届く。

use alloc::vec::Vec;

use nene_ledger_core::display_width::display_columns;
use nene_ledger_core::draw_command::DrawCommand;
use nene_ledger_core::palette::Palette;
use nene_ledger_core::pixel_point::PixelPoint;
use nene_ledger_core::pixel_rect::PixelRect;
use nene_ledger_core::pixel_size::PixelSize;
use nene_ledger_core::text_role::TextRole;

use crate::close_choice::CloseChoice;
use crate::window_view::WindowView;

/// 問いの文言。
const QUESTION: &str = "変えた所が保存されていません。閉じる前に保存しますか？";
/// 箱の内側の余白（論理の画素）。
const PADDING: u32 = 16;
/// ボタンの間（論理の画素）。
const GAP: u32 = 12;
/// ボタンの文字の左右の余白（論理の画素）。
const BUTTON_PADDING: u32 = 12;
/// 箱の縁の太さ（論理の画素）。
const EDGE: u32 = 2;

/// 窓 `view` での、尋ねる画面の配置。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ClosePrompt {
    view: WindowView,
}

impl ClosePrompt {
    /// 窓 `view` に置く。
    pub(crate) const fn new(view: WindowView) -> Self {
        Self { view }
    }

    /// 文字列 `text` の幅（画素・半角 1 字の幅で数える近似）。
    fn width_of(&self, text: &str) -> u32 {
        display_columns(text).saturating_mul(self.view.metrics().half_width())
    }

    /// ボタンの高さ。
    fn button_height(&self) -> u32 {
        let metrics = self.view.metrics();
        metrics.line_height().saturating_add(metrics.px(PADDING))
    }

    /// ボタンの幅。
    fn button_width(&self, choice: CloseChoice) -> u32 {
        self.width_of(choice.label())
            .saturating_add(self.view.metrics().px(BUTTON_PADDING).saturating_mul(2))
    }

    /// ボタンの幅の合計（間を含む）。
    fn buttons_width(&self) -> u32 {
        let gaps = self.view.metrics().px(GAP).saturating_mul(2);
        CloseChoice::ALL
            .iter()
            .map(|&choice| self.button_width(choice))
            .fold(gaps, u32::saturating_add)
    }

    /// 箱の矩形（窓の真ん中・窓に収める）。
    pub(crate) fn frame(&self) -> PixelRect {
        let metrics = self.view.metrics();
        let padding = metrics.px(PADDING);
        let inner = self.width_of(QUESTION).max(self.buttons_width());
        let width = inner
            .saturating_add(padding.saturating_mul(2))
            .min(self.view.size().width());
        let height = metrics
            .line_height()
            .saturating_add(self.button_height())
            .saturating_add(padding.saturating_mul(3))
            .min(self.view.size().height());
        let left = self.view.size().width().saturating_sub(width) / 2;
        let top = self.view.size().height().saturating_sub(height) / 2;
        PixelRect::new(PixelPoint::new(left, top), PixelSize::new(width, height))
    }

    /// ボタンの矩形（左から `CloseChoice::ALL` の順）。
    pub(crate) fn buttons(&self) -> [(CloseChoice, PixelRect); 3] {
        let metrics = self.view.metrics();
        let frame = self.frame();
        let padding = metrics.px(PADDING);
        let top = frame
            .origin()
            .y()
            .saturating_add(padding.saturating_mul(2))
            .saturating_add(metrics.line_height());
        let mut left = frame.origin().x().saturating_add(padding);
        CloseChoice::ALL.map(|choice| {
            let width = self.button_width(choice);
            let rect = PixelRect::new(
                PixelPoint::new(left, top),
                PixelSize::new(width, self.button_height()),
            );
            left = left.saturating_add(width).saturating_add(metrics.px(GAP));
            (choice, rect)
        })
    }

    /// 窓の中の位置 `at` のボタン。ボタンの外なら `None`。
    pub(crate) fn choice_at(&self, at: PixelPoint) -> Option<CloseChoice> {
        self.buttons()
            .into_iter()
            .find(|&(_, rect)| contains(rect, at))
            .map(|(choice, _)| choice)
    }

    /// 描画命令（色は `colors`）。
    pub(crate) fn paint(&self, colors: Palette) -> Vec<DrawCommand> {
        let metrics = self.view.metrics();
        let frame = self.frame();
        let edge = metrics.px(EDGE);
        let mut commands = Vec::new();
        commands.push(DrawCommand::Fill {
            rect: frame,
            color: colors.cursor(),
        });
        commands.push(DrawCommand::Fill {
            rect: inset(frame, edge),
            color: colors.background(),
        });
        let padding = metrics.px(PADDING);
        let question = PixelPoint::new(
            frame.origin().x().saturating_add(padding),
            frame
                .origin()
                .y()
                .saturating_add(padding)
                .saturating_add(metrics.ascent()),
        );
        commands.push(DrawCommand::Text {
            baseline: question,
            text: alloc::string::String::from(QUESTION),
            color: colors.text(),
            clip: frame,
            role: TextRole::Chrome,
        });
        for (choice, rect) in self.buttons() {
            let fill = match choice {
                CloseChoice::Save => colors.selection(),
                CloseChoice::Discard | CloseChoice::Cancel => colors.gutter(),
            };
            commands.push(DrawCommand::Fill { rect, color: fill });
            let spare = rect.size().height().saturating_sub(metrics.line_height());
            let baseline = PixelPoint::new(
                rect.origin().x().saturating_add(metrics.px(BUTTON_PADDING)),
                rect.origin()
                    .y()
                    .saturating_add(spare / 2)
                    .saturating_add(metrics.ascent()),
            );
            commands.push(DrawCommand::Text {
                baseline,
                text: alloc::string::String::from(choice.label()),
                color: colors.text(),
                clip: rect,
                role: TextRole::Chrome,
            });
        }
        commands
    }
}

/// `rect` が `at` を含むか。
fn contains(rect: PixelRect, at: PixelPoint) -> bool {
    let (left, top) = (rect.origin().x(), rect.origin().y());
    at.x() >= left
        && at.y() >= top
        && at.x() < left.saturating_add(rect.size().width())
        && at.y() < top.saturating_add(rect.size().height())
}

/// `rect` を四方から `by` だけ縮めた矩形。
fn inset(rect: PixelRect, by: u32) -> PixelRect {
    PixelRect::new(
        PixelPoint::new(
            rect.origin().x().saturating_add(by),
            rect.origin().y().saturating_add(by),
        ),
        PixelSize::new(
            rect.size().width().saturating_sub(by.saturating_mul(2)),
            rect.size().height().saturating_sub(by.saturating_mul(2)),
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use nene_ledger_core::glyph_metrics::GlyphMetrics;
    use nene_ledger_core::window_placement::WindowPlacement;

    fn view() -> WindowView {
        WindowView::new(
            PixelSize::new(1000, 600),
            GlyphMetrics::new(8, 16, 12),
            WindowPlacement::Normal,
            None,
        )
    }

    #[test]
    fn the_box_is_centred_and_the_buttons_sit_inside_it_from_left_to_right() {
        let prompt = ClosePrompt::new(view());
        let frame = prompt.frame();
        assert_eq!(
            frame.origin().x() * 2 + frame.size().width(),
            1000,
            "{frame:?}"
        );
        let buttons = prompt.buttons();
        assert_eq!(buttons.map(|(choice, _)| choice), CloseChoice::ALL);
        for pair in buttons.windows(2) {
            assert!(pair[0].1.origin().x() < pair[1].1.origin().x());
        }
        let last = buttons[2].1;
        assert!(
            last.origin().x() + last.size().width() <= frame.origin().x() + frame.size().width()
        );
    }

    #[test]
    fn pressing_a_button_chooses_it_and_elsewhere_chooses_nothing() {
        let prompt = ClosePrompt::new(view());
        for (choice, rect) in prompt.buttons() {
            let middle = PixelPoint::new(
                rect.origin().x() + rect.size().width() / 2,
                rect.origin().y() + rect.size().height() / 2,
            );
            assert_eq!(prompt.choice_at(middle), Some(choice));
        }
        assert_eq!(prompt.choice_at(PixelPoint::new(0, 0)), None);
        let frame = prompt.frame();
        assert_eq!(prompt.choice_at(frame.origin()), None);
    }

    #[test]
    fn the_box_draws_the_question_and_every_label() {
        let commands =
            ClosePrompt::new(view()).paint(crate::theme_state::ThemeState::at(1).palette());
        let texts: Vec<&str> = commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Text { text, .. } => Some(text.as_str()),
                DrawCommand::Fill { .. }
                | DrawCommand::Line { .. }
                | DrawCommand::TextSpan { .. } => None,
            })
            .collect();
        assert!(texts.contains(&QUESTION));
        for choice in CloseChoice::ALL {
            assert!(texts.contains(&choice.label()));
        }
    }
}
