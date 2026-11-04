//! タイトルバーとボタンの上のマウスの状態。

use nene_ledger_core::chrome_button::ChromeButton;
use nene_ledger_core::chrome_hit::ChromeHit;

use crate::chrome_action::ChromeAction;
use crate::pointer_at::PointerAt;
use crate::press::Press;

/// 2 回の押しをダブルクリックとみなす間隔（ミリ秒）。
const DOUBLE_CLICK_MS: u32 = 500;
/// タイトルバーを押してからこれより大きく動いたら、窓の移動を頼む（物理画素）。
const DRAG_THRESHOLD: i32 = 4;

/// タイトルバーとボタンの上のマウスの状態。当たり判定（core）の結果と X の時刻から、頼むことを決める。
///
/// 押しただけでは動かさず、少し動いてから移動を頼む（ダブルクリックをウィンドウマネージャーに奪われない）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Pointer {
    press: Press,
    last_caption_press: Option<u32>,
    hovered: Option<ChromeButton>,
}

impl Pointer {
    pub(crate) fn new() -> Self {
        Self {
            press: Press::Idle,
            last_caption_press: None,
            hovered: None,
        }
    }

    /// 指の乗ったボタン。
    pub(crate) fn hovered(self) -> Option<ChromeButton> {
        self.hovered
    }

    /// 左のボタンを `hit` の所で押した。
    pub(crate) fn press(&mut self, hit: ChromeHit, at: PointerAt) -> ChromeAction {
        self.press = Press::Idle;
        match hit {
            ChromeHit::Client => ChromeAction::Nothing,
            ChromeHit::Edge(edge) => ChromeAction::Resize(edge),
            ChromeHit::Button(button) => {
                self.press = Press::Button(button);
                ChromeAction::Nothing
            }
            ChromeHit::Caption => {
                let double = self
                    .last_caption_press
                    .is_some_and(|last| at.time().wrapping_sub(last) <= DOUBLE_CLICK_MS);
                if double {
                    self.last_caption_press = None;
                    return ChromeAction::ToggleMaximize;
                }
                self.last_caption_press = Some(at.time());
                self.press = Press::Caption(at);
                ChromeAction::Nothing
            }
        }
    }

    /// マウスが `hit` の所へ動いた。
    pub(crate) fn motion(&mut self, hit: ChromeHit, at: PointerAt) -> ChromeAction {
        if let Press::Caption(from) = self.press
            && at.is_far_from(from, DRAG_THRESHOLD)
        {
            self.press = Press::Idle;
            self.last_caption_press = None;
            return ChromeAction::Move;
        }
        let hovered = match hit {
            ChromeHit::Button(button) => Some(button),
            ChromeHit::Client | ChromeHit::Caption | ChromeHit::Edge(_) => None,
        };
        self.hover(hovered)
    }

    /// 左のボタンを `hit` の所で離した。押したボタンの上で離したときだけ、そのボタンが働く。
    pub(crate) fn release(&mut self, hit: ChromeHit) -> ChromeAction {
        let press = self.press;
        self.press = Press::Idle;
        match (press, hit) {
            (Press::Button(pressed), ChromeHit::Button(released)) if pressed == released => {
                match pressed {
                    ChromeButton::Minimize => ChromeAction::Minimize,
                    ChromeButton::Maximize => ChromeAction::ToggleMaximize,
                    ChromeButton::Close => ChromeAction::Close,
                }
            }
            (Press::Idle | Press::Caption(_) | Press::Button(_), _) => ChromeAction::Nothing,
        }
    }

    /// マウスが窓の外へ出た。
    pub(crate) fn leave(&mut self) -> ChromeAction {
        self.hover(None)
    }

    fn hover(&mut self, hovered: Option<ChromeButton>) -> ChromeAction {
        if self.hovered == hovered {
            return ChromeAction::Nothing;
        }
        self.hovered = hovered;
        ChromeAction::Redraw
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nene_ledger_core::resize_edge::ResizeEdge;

    const CLOSE: ChromeHit = ChromeHit::Button(ChromeButton::Close);

    fn at(root_x: i16, root_y: i16, time: u32) -> PointerAt {
        PointerAt::new(root_x, root_y, time)
    }

    #[test]
    fn a_button_works_only_when_released_over_it() {
        let mut pointer = Pointer::new();
        assert_eq!(pointer.press(CLOSE, at(0, 0, 10)), ChromeAction::Nothing);
        assert_eq!(pointer.release(CLOSE), ChromeAction::Close);
        pointer.press(CLOSE, at(0, 0, 20));
        assert_eq!(pointer.release(ChromeHit::Client), ChromeAction::Nothing);
        assert_eq!(pointer.release(CLOSE), ChromeAction::Nothing);
        let minimize = ChromeHit::Button(ChromeButton::Minimize);
        pointer.press(minimize, at(0, 0, 30));
        assert_eq!(pointer.release(minimize), ChromeAction::Minimize);
        let maximize = ChromeHit::Button(ChromeButton::Maximize);
        pointer.press(maximize, at(0, 0, 40));
        assert_eq!(pointer.release(CLOSE), ChromeAction::Nothing);
        pointer.press(maximize, at(0, 0, 50));
        assert_eq!(pointer.release(maximize), ChromeAction::ToggleMaximize);
    }

    #[test]
    fn dragging_the_caption_moves_after_the_threshold() {
        let mut pointer = Pointer::new();
        assert_eq!(
            pointer.press(ChromeHit::Caption, at(100, 100, 0)),
            ChromeAction::Nothing
        );
        assert_eq!(
            pointer.motion(ChromeHit::Caption, at(104, 96, 0)),
            ChromeAction::Nothing
        );
        assert_eq!(
            pointer.motion(ChromeHit::Caption, at(105, 100, 0)),
            ChromeAction::Move
        );
        // 動かした後は押していない扱い。
        assert_eq!(
            pointer.motion(ChromeHit::Caption, at(200, 200, 0)),
            ChromeAction::Nothing
        );
        assert_eq!(pointer.release(ChromeHit::Caption), ChromeAction::Nothing);
    }

    #[test]
    fn a_double_click_on_the_caption_toggles_maximize() {
        let mut pointer = Pointer::new();
        pointer.press(ChromeHit::Caption, at(0, 0, 1_000));
        pointer.release(ChromeHit::Caption);
        assert_eq!(
            pointer.press(ChromeHit::Caption, at(0, 0, 1_500)),
            ChromeAction::ToggleMaximize
        );
        // 3 回目は新しい 1 回目。
        assert_eq!(
            pointer.press(ChromeHit::Caption, at(0, 0, 1_600)),
            ChromeAction::Nothing
        );
        assert_eq!(
            pointer.press(ChromeHit::Caption, at(0, 0, 2_101)),
            ChromeAction::Nothing
        );
        // X の時刻が一巡しても間隔で見る。
        let mut wrapped = Pointer::new();
        wrapped.press(ChromeHit::Caption, at(0, 0, u32::MAX - 10));
        assert_eq!(
            wrapped.press(ChromeHit::Caption, at(0, 0, 10)),
            ChromeAction::ToggleMaximize
        );
    }

    #[test]
    fn edges_resize_and_the_client_does_nothing() {
        let mut pointer = Pointer::new();
        let edge = ChromeHit::Edge(ResizeEdge::BottomRight);
        assert_eq!(
            pointer.press(edge, at(0, 0, 0)),
            ChromeAction::Resize(ResizeEdge::BottomRight)
        );
        assert_eq!(
            pointer.press(ChromeHit::Client, at(0, 0, 0)),
            ChromeAction::Nothing
        );
    }

    #[test]
    fn hovering_redraws_only_on_change() {
        let mut pointer = Pointer::new();
        assert_eq!(pointer.motion(CLOSE, at(0, 0, 0)), ChromeAction::Redraw);
        assert_eq!(pointer.hovered(), Some(ChromeButton::Close));
        assert_eq!(pointer.motion(CLOSE, at(1, 1, 0)), ChromeAction::Nothing);
        assert_eq!(
            pointer.motion(ChromeHit::Caption, at(0, 0, 0)),
            ChromeAction::Redraw
        );
        assert_eq!(pointer.hovered(), None);
        pointer.motion(CLOSE, at(0, 0, 0));
        assert_eq!(pointer.leave(), ChromeAction::Redraw);
        assert_eq!(pointer.leave(), ChromeAction::Nothing);
    }
}
