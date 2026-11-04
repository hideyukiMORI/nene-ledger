//! core の当たり判定を Windows の領域（`WM_NCHITTEST` の値）に写す（ADR 0013 の 1）。Win32 を呼ばない。

use nene_ledger_core::chrome_button::ChromeButton;
use nene_ledger_core::chrome_hit::ChromeHit;
use nene_ledger_core::resize_edge::ResizeEdge;

/// winuser.h の当たり判定の値。
pub(crate) const HTCLIENT: u32 = 1;
pub(crate) const HTCAPTION: u32 = 2;
pub(crate) const HTMINBUTTON: u32 = 8;
pub(crate) const HTMAXBUTTON: u32 = 9;
pub(crate) const HTLEFT: u32 = 10;
pub(crate) const HTRIGHT: u32 = 11;
pub(crate) const HTTOP: u32 = 12;
pub(crate) const HTTOPLEFT: u32 = 13;
pub(crate) const HTTOPRIGHT: u32 = 14;
pub(crate) const HTBOTTOM: u32 = 15;
pub(crate) const HTBOTTOMLEFT: u32 = 16;
pub(crate) const HTBOTTOMRIGHT: u32 = 17;
pub(crate) const HTCLOSE: u32 = 20;

/// core の当たり判定 `hit` の、`WM_NCHITTEST` の値。ボタンは OS の値にする（最大化の上で Windows 11 の Snap Layouts が出る）。
pub(crate) fn hit_code(hit: ChromeHit) -> u32 {
    match hit {
        ChromeHit::Client => HTCLIENT,
        ChromeHit::Caption => HTCAPTION,
        ChromeHit::Button(ChromeButton::Minimize) => HTMINBUTTON,
        ChromeHit::Button(ChromeButton::Maximize) => HTMAXBUTTON,
        ChromeHit::Button(ChromeButton::Close) => HTCLOSE,
        ChromeHit::Edge(edge) => match edge {
            ResizeEdge::Left => HTLEFT,
            ResizeEdge::Right => HTRIGHT,
            ResizeEdge::Top => HTTOP,
            ResizeEdge::Bottom => HTBOTTOM,
            ResizeEdge::TopLeft => HTTOPLEFT,
            ResizeEdge::TopRight => HTTOPRIGHT,
            ResizeEdge::BottomLeft => HTBOTTOMLEFT,
            ResizeEdge::BottomRight => HTBOTTOMRIGHT,
        },
    }
}

/// `WM_NCHITTEST` の値 `code` の、窓のボタン（ボタンでなければ `None`）。
pub(crate) fn button_of(code: u32) -> Option<ChromeButton> {
    match code {
        HTMINBUTTON => Some(ChromeButton::Minimize),
        HTMAXBUTTON => Some(ChromeButton::Maximize),
        HTCLOSE => Some(ChromeButton::Close),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buttons_and_edges_become_windows_codes_and_back() {
        for button in [
            ChromeButton::Minimize,
            ChromeButton::Maximize,
            ChromeButton::Close,
        ] {
            assert_eq!(button_of(hit_code(ChromeHit::Button(button))), Some(button));
        }
        assert_eq!(hit_code(ChromeHit::Caption), HTCAPTION);
        assert_eq!(
            hit_code(ChromeHit::Edge(ResizeEdge::BottomRight)),
            HTBOTTOMRIGHT
        );
        assert_eq!(button_of(HTCLIENT), None);
    }
}
