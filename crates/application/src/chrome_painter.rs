//! 窓の飾り（タイトルバー・ボタン・縁の線）を描画命令に変える。

use alloc::string::String;
use alloc::vec::Vec;

use nene_ledger_core::chrome_button::ChromeButton;
use nene_ledger_core::draw_command::DrawCommand;
use nene_ledger_core::palette::Palette;
use nene_ledger_core::pixel_point::PixelPoint;
use nene_ledger_core::pixel_rect::PixelRect;
use nene_ledger_core::pixel_size::PixelSize;
use nene_ledger_core::text_role::TextRole;
use nene_ledger_core::window_chrome::WindowChrome;
use nene_ledger_core::window_placement::WindowPlacement;

use crate::chrome_icon::ChromeIcon;

/// タイトルバーの下の区切りの線の太さ（画素・100% のとき）。
const TITLE_RULE: u32 = 1;
/// タイトルバーの左端の、操作の一覧を開くボタンの字（GitHub Issue #446）。
pub(crate) const MENU_LABEL: &str = "≡ 操作";
use crate::window_view::WindowView;

/// 窓の飾りの描画命令を、描く順に積む。
#[derive(Debug)]
pub(crate) struct ChromePainter {
    view: WindowView,
    chrome: WindowChrome,
    palette: Palette,
    commands: Vec<DrawCommand>,
}

impl ChromePainter {
    pub(crate) fn new(view: WindowView, palette: Palette) -> Self {
        Self {
            view,
            chrome: view.chrome(),
            palette,
            commands: Vec::new(),
        }
    }

    /// タイトルバー（背景・題名・3 つのボタン）の命令。
    pub(crate) fn title_bar(mut self, title: &str) -> Vec<DrawCommand> {
        let band = self.chrome.title_bar();
        self.commands.push(DrawCommand::Fill {
            rect: band,
            color: self.palette.title_bar(),
        });
        let metrics = self.view.metrics();
        let spare = band.size().height().saturating_sub(metrics.line_height());
        let baseline_y = (spare / 2).saturating_add(metrics.ascent());
        self.menu(baseline_y);
        let caption = self.chrome.caption();
        self.commands.push(DrawCommand::Text {
            baseline: PixelPoint::new(
                caption
                    .origin()
                    .x()
                    .saturating_add(metrics.half_width().saturating_mul(3) / 2),
                baseline_y,
            ),
            text: String::from(title),
            color: self.palette.title_text(),
            clip: self.chrome.caption(),
            role: TextRole::Chrome,
        });
        for button in ChromeButton::ALL {
            self.button(button);
        }
        // タイトルバーと中身（見出しの行の灰色）が 1 つの塊に見えないよう、下に区切りの線。
        let rule = metrics.px(TITLE_RULE).min(band.size().height());
        let rule_top = band.size().height().saturating_sub(rule);
        self.commands.push(DrawCommand::Fill {
            rect: PixelRect::new(
                PixelPoint::new(0, rule_top),
                PixelSize::new(band.size().width(), rule),
            ),
            color: self.palette.chrome_rule(),
        });
        self.commands
    }

    /// 左端の「≡ 操作」のボタン（押せば操作の一覧を開く・GitHub Issue #446）。右に題名との区切りの細い線。
    fn menu(&mut self, baseline_y: u32) {
        let metrics = self.view.metrics();
        let menu = self.chrome.menu();
        if menu.size().width() == 0 {
            return;
        }
        self.commands.push(DrawCommand::Text {
            baseline: PixelPoint::new(
                menu.origin()
                    .x()
                    .saturating_add(metrics.half_width().saturating_mul(3) / 2),
                baseline_y,
            ),
            text: String::from(MENU_LABEL),
            color: self.palette.title_text(),
            clip: menu,
            role: TextRole::Chrome,
        });
        let line = metrics.px(TITLE_RULE);
        let inset = menu.size().height() / 4;
        self.commands.push(DrawCommand::Fill {
            rect: PixelRect::new(
                PixelPoint::new(
                    menu.origin()
                        .x()
                        .saturating_add(menu.size().width())
                        .saturating_sub(line),
                    inset,
                ),
                PixelSize::new(
                    line,
                    menu.size().height().saturating_sub(inset.saturating_mul(2)),
                ),
            ),
            color: self.palette.chrome_rule(),
        });
    }

    /// 窓の縁の線の命令。最大化している間は無い。中身の後に重ねる。
    pub(crate) fn border(self) -> Vec<DrawCommand> {
        match self.chrome.placement() {
            WindowPlacement::Normal => self
                .chrome
                .border_lines()
                .into_iter()
                .map(|rect| DrawCommand::Fill {
                    rect,
                    color: self.palette.border(),
                })
                .collect(),
            WindowPlacement::Maximized => Vec::new(),
        }
    }

    /// 1 つのボタン。指が乗っていれば背景を塗り（閉じるは赤）、真ん中に印を描く。
    fn button(&mut self, button: ChromeButton) {
        let rect = self.chrome.button(button);
        let hovered = self.view.hovered() == Some(button);
        let (background, ink) = match (button, hovered) {
            (ChromeButton::Close, true) => (
                Some(self.palette.close_hover()),
                self.palette.close_hover_text(),
            ),
            (ChromeButton::Minimize | ChromeButton::Maximize, true) => {
                (Some(self.palette.button_hover()), self.palette.title_text())
            }
            (ChromeButton::Minimize | ChromeButton::Maximize | ChromeButton::Close, false) => {
                (None, self.palette.title_text())
            }
        };
        if let Some(color) = background {
            self.commands.push(DrawCommand::Fill { rect, color });
        }
        let icon = ChromeIcon::centered(rect, self.view.metrics(), ink);
        let lines = match button {
            ChromeButton::Minimize => icon.minimize(),
            ChromeButton::Maximize => match self.chrome.placement() {
                WindowPlacement::Normal => icon.maximize(),
                WindowPlacement::Maximized => icon.restore(),
            },
            ChromeButton::Close => icon.close(),
        };
        self.commands.extend(lines);
    }
}
