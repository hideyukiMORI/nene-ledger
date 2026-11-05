//! Windows の仮想キーとマウスを意図にする（X11 版の `key_intent` と同じ割り当て・Issue #55）。
//!
//! Win32 を呼ばない純関数なので、どの OS でもテストする。仮想キーの値は winuser.h のもの。

use nene_ledger_application::column_shift::ColumnShift;
use nene_ledger_application::cursor_step::CursorStep;
use nene_ledger_application::intent::Intent;
use nene_ledger_application::scroll::Scroll;
use nene_ledger_application::selection_mode::SelectionMode;
use nene_ledger_application::theme_step::ThemeStep;
use nene_ledger_application::zoom_step::ZoomStep;
use nene_ledger_core::pixel_point::PixelPoint;

/// ホイールの 1 刻み（`WHEEL_DELTA`）で動かす行の数と列の数（X11 版と同じ）。
const WHEEL_ROWS: u64 = 3;
const WHEEL_COLUMNS: usize = 1;

const VK_BACK: u16 = 0x08;
const VK_TAB: u16 = 0x09;
const VK_DELETE: u16 = 0x2E;
const VK_F2: u16 = 0x71;
const VK_RETURN: u16 = 0x0D;
const VK_ESCAPE: u16 = 0x1B;
const VK_SPACE: u16 = 0x20;
const VK_PRIOR: u16 = 0x21;
const VK_NEXT: u16 = 0x22;
const VK_END: u16 = 0x23;
const VK_HOME: u16 = 0x24;
const VK_LEFT: u16 = 0x25;
const VK_UP: u16 = 0x26;
const VK_RIGHT: u16 = 0x27;
const VK_DOWN: u16 = 0x28;
const VK_0: u16 = 0x30;
const VK_A: u16 = 0x41;
const VK_C: u16 = 0x43;
const VK_G: u16 = 0x47;
const VK_P: u16 = 0x50;
const VK_S: u16 = 0x53;
const VK_T: u16 = 0x54;
const VK_U: u16 = 0x55;
const VK_NUMPAD0: u16 = 0x60;
const VK_ADD: u16 = 0x6B;
const VK_SUBTRACT: u16 = 0x6D;
const VK_F1: u16 = 0x70;
/// `;` と `+` のキー（日本語の配列）・`=` と `+` のキー（英語の配列）。
const VK_OEM_PLUS: u16 = 0xBB;
const VK_OEM_1: u16 = 0xBA;
const VK_OEM_MINUS: u16 = 0xBD;

/// 押されている修飾キー。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Modifiers {
    shift: bool,
    control: bool,
    alt: bool,
}

impl Modifiers {
    /// Shift・Ctrl・Alt の押下。
    pub(crate) fn new(shift: bool, control: bool, alt: bool) -> Self {
        Self {
            shift,
            control,
            alt,
        }
    }
}

/// `WM_KEYDOWN` の仮想キー `key` の意図。字を打つのは `WM_CHAR`（`typed_intent`）。
/// 矢印・PageUp・PageDown・Home・End でカーソルを動かす。Shift で選択を広げ、Ctrl+Home / Ctrl+End は表の先頭 / 末尾。
pub(crate) fn key_intent(key: u16, modifiers: Modifiers) -> Option<Intent> {
    let Modifiers {
        shift,
        control,
        alt,
    } = modifiers;
    match key {
        VK_F1 => return Some(Intent::OpenPalette),
        VK_F2 => return Some(Intent::Edit),
        VK_ESCAPE => return Some(Intent::Cancel),
        // Alt+Enter は `WM_SYSKEYDOWN` で届く（ui の側で同じ関数に渡す）。
        VK_RETURN if alt => return Some(Intent::BreakLine),
        VK_RETURN if shift => return Some(Intent::Advance(CursorStep::Up)),
        VK_RETURN => return Some(Intent::Confirm),
        VK_TAB if shift => return Some(Intent::Advance(CursorStep::Left)),
        VK_TAB => return Some(Intent::Advance(CursorStep::Right)),
        VK_BACK => return Some(Intent::Erase),
        VK_DELETE => return Some(Intent::Delete),
        // 行全体（Issue #62）。続く `WM_CHAR` の空白は表では何もしない。
        VK_SPACE if shift && !control => return Some(Intent::SelectRows),
        _ => {}
    }
    if control && let Some(intent) = control_intent(key, shift, alt) {
        return Some(intent);
    }
    let step = match key {
        VK_UP => CursorStep::Up,
        VK_DOWN => CursorStep::Down,
        VK_LEFT => CursorStep::Left,
        VK_RIGHT => CursorStep::Right,
        VK_PRIOR => CursorStep::PageUp,
        VK_NEXT => CursorStep::PageDown,
        VK_HOME if control => CursorStep::Top,
        VK_HOME => CursorStep::RowStart,
        VK_END if control => CursorStep::Bottom,
        VK_END => CursorStep::RowEnd,
        _ => return None,
    };
    let mode = if shift {
        SelectionMode::Extend
    } else {
        SelectionMode::Collapse
    };
    Some(Intent::Move { step, mode })
}

/// Ctrl と組んだキー。コピー・すべて・飛ぶ・拡大、Ctrl+Shift でパレット・詳細ペイン・配色。
fn control_intent(key: u16, shift: bool, alt: bool) -> Option<Intent> {
    match (key, shift) {
        (VK_C, _) => Some(Intent::Copy),
        (VK_SPACE, false) => Some(Intent::SelectColumns),
        (VK_PRIOR, true) => Some(Intent::MoveColumns(ColumnShift::Left)),
        (VK_NEXT, true) => Some(Intent::MoveColumns(ColumnShift::Right)),
        (VK_A, false) => Some(Intent::SelectAll),
        (VK_S, false) => Some(Intent::Save),
        (VK_G, false) => Some(Intent::OpenGoTo),
        (VK_P, true) => Some(Intent::OpenPalette),
        (VK_U, true) => Some(Intent::ToggleDetail),
        (VK_T, true) => Some(Intent::Theme(if alt {
            ThemeStep::Previous
        } else {
            ThemeStep::Next
        })),
        (VK_OEM_PLUS | VK_OEM_1 | VK_ADD, _) => Some(Intent::Zoom(ZoomStep::In)),
        (VK_OEM_MINUS | VK_SUBTRACT, _) => Some(Intent::Zoom(ZoomStep::Out)),
        (VK_0 | VK_NUMPAD0, _) => Some(Intent::Zoom(ZoomStep::Reset)),
        _ => None,
    }
}

/// `WM_CHAR` の字 `character` の意図。制御文字でない字を打つ。Ctrl と組めば打たない。
/// IME の確定も `WM_CHAR` で 1 字ずつ届くので、日本語もここで打てる（確定前の文字列を見せるのは IMM32 の Issue・ADR 0026）。
/// 🔴 BMP の外の字（絵文字など）はサロゲートの対で届き、1 つずつでは `char` にならないので、今は捨てる。
pub(crate) fn typed_intent(character: char, control: bool) -> Option<Intent> {
    (!character.is_control() && !control).then_some(Intent::Type(character))
}

/// 左のボタンを窓の中の `(x, y)` で押した意図。Shift で広げ、Ctrl で範囲を足す（両方なら広げる）。
pub(crate) fn press_intent(x: i32, y: i32, modifiers: Modifiers) -> Option<Intent> {
    let at = PixelPoint::new(u32::try_from(x).ok()?, u32::try_from(y).ok()?);
    let mode = if modifiers.shift {
        SelectionMode::Extend
    } else if modifiers.control {
        SelectionMode::Add
    } else {
        SelectionMode::Collapse
    };
    Some(Intent::Point { at, mode })
}

/// 縦のホイール（`WM_MOUSEWHEEL`）の刻み `delta` の意図。上が正。Ctrl で拡大、Shift で横へ。
pub(crate) fn wheel_intent(delta: i16, modifiers: Modifiers) -> Option<Intent> {
    if delta == 0 {
        return None;
    }
    let up = delta > 0;
    if modifiers.control {
        return Some(Intent::Zoom(if up { ZoomStep::In } else { ZoomStep::Out }));
    }
    let scroll = match (up, modifiers.shift) {
        (true, true) => Scroll::Left {
            columns: WHEEL_COLUMNS,
        },
        (false, true) => Scroll::Right {
            columns: WHEEL_COLUMNS,
        },
        (true, false) => Scroll::Up { rows: WHEEL_ROWS },
        (false, false) => Scroll::Down { rows: WHEEL_ROWS },
    };
    Some(Intent::Scroll(scroll))
}

/// 横のホイール（`WM_MOUSEHWHEEL`）の刻み `delta` の意図。右が正。
pub(crate) fn sideways_wheel_intent(delta: i16) -> Option<Intent> {
    match delta {
        0 => None,
        right if right > 0 => Some(Intent::Scroll(Scroll::Right {
            columns: WHEEL_COLUMNS,
        })),
        _ => Some(Intent::Scroll(Scroll::Left {
            columns: WHEEL_COLUMNS,
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain() -> Modifiers {
        Modifiers::new(false, false, false)
    }

    fn control() -> Modifiers {
        Modifiers::new(false, true, false)
    }

    fn control_shift() -> Modifiers {
        Modifiers::new(true, true, false)
    }

    #[test]
    fn space_selects_whole_lines_and_page_keys_move_columns() {
        let shift = Modifiers::new(true, false, false);
        assert_eq!(key_intent(VK_SPACE, shift), Some(Intent::SelectRows));
        assert_eq!(key_intent(VK_SPACE, control()), Some(Intent::SelectColumns));
        // Space だけは `WM_CHAR` で字になる。
        assert_eq!(key_intent(VK_SPACE, plain()), None);
        assert_eq!(typed_intent(' ', false), Some(Intent::Type(' ')));
        assert_eq!(
            key_intent(VK_PRIOR, control_shift()),
            Some(Intent::MoveColumns(ColumnShift::Left))
        );
        assert_eq!(
            key_intent(VK_NEXT, control_shift()),
            Some(Intent::MoveColumns(ColumnShift::Right))
        );
        assert_eq!(
            key_intent(VK_NEXT, shift),
            Some(Intent::Move {
                step: CursorStep::PageDown,
                mode: SelectionMode::Extend
            })
        );
    }

    /// 編集のキーの一覧の行（ADR 0026）が、書いたとおりに割り当てられている。
    fn editing_bound(guide: nene_ledger_application::key_guide::KeyGuide) -> bool {
        use nene_ledger_application::key_guide::KeyGuide;
        let shift = Modifiers::new(true, false, false);
        match guide {
            KeyGuide::EditCell => key_intent(VK_F2, plain()) == Some(Intent::Edit),
            KeyGuide::ClearCell => key_intent(VK_DELETE, plain()) == Some(Intent::Delete),
            KeyGuide::BreakLine => {
                key_intent(VK_RETURN, Modifiers::new(false, false, true)) == Some(Intent::BreakLine)
            }
            KeyGuide::Advance => {
                key_intent(VK_TAB, plain()) == Some(Intent::Advance(CursorStep::Right))
                    && key_intent(VK_TAB, shift) == Some(Intent::Advance(CursorStep::Left))
                    && key_intent(VK_RETURN, shift) == Some(Intent::Advance(CursorStep::Up))
            }
            KeyGuide::MoveCursor
            | KeyGuide::ExtendSelection
            | KeyGuide::Page
            | KeyGuide::RowEnds
            | KeyGuide::ExtendClick
            | KeyGuide::AddRange
            | KeyGuide::Sideways
            | KeyGuide::WheelZoom
            | KeyGuide::NextTheme
            | KeyGuide::PreviousTheme
            | KeyGuide::Palette
            | KeyGuide::Cancel => false,
        }
    }

    /// キーの一覧（Issue #63）の各行が、書いたとおりに割り当てられている。行を足せばここの `match` が落ちる。
    #[test]
    fn every_key_in_the_list_is_bound_as_written() {
        use nene_ledger_application::key_guide::KeyGuide;
        let shift = Modifiers::new(true, false, false);
        let with_alt = Modifiers::new(true, true, true);
        let moving = |step, mode| Some(Intent::Move { step, mode });
        let point = |mode| {
            Some(Intent::Point {
                at: PixelPoint::new(5, 5),
                mode,
            })
        };
        for guide in KeyGuide::ALL {
            let bound = match guide {
                KeyGuide::MoveCursor => {
                    key_intent(VK_LEFT, plain())
                        == moving(CursorStep::Left, SelectionMode::Collapse)
                }
                KeyGuide::ExtendSelection => {
                    key_intent(VK_DOWN, shift) == moving(CursorStep::Down, SelectionMode::Extend)
                }
                KeyGuide::Page => {
                    key_intent(VK_NEXT, plain())
                        == moving(CursorStep::PageDown, SelectionMode::Collapse)
                }
                KeyGuide::RowEnds => {
                    key_intent(VK_END, plain())
                        == moving(CursorStep::RowEnd, SelectionMode::Collapse)
                }
                KeyGuide::ExtendClick => press_intent(5, 5, shift) == point(SelectionMode::Extend),
                KeyGuide::AddRange => press_intent(5, 5, control()) == point(SelectionMode::Add),
                KeyGuide::Sideways => {
                    wheel_intent(-120, shift)
                        == Some(Intent::Scroll(Scroll::Right {
                            columns: WHEEL_COLUMNS,
                        }))
                }
                KeyGuide::WheelZoom => {
                    wheel_intent(120, control()) == Some(Intent::Zoom(ZoomStep::In))
                }
                KeyGuide::NextTheme => {
                    key_intent(VK_T, control_shift()) == Some(Intent::Theme(ThemeStep::Next))
                }
                KeyGuide::PreviousTheme => {
                    key_intent(VK_T, with_alt) == Some(Intent::Theme(ThemeStep::Previous))
                }
                KeyGuide::Palette => {
                    key_intent(VK_P, control_shift()) == Some(Intent::OpenPalette)
                        && key_intent(VK_F1, plain()) == Some(Intent::OpenPalette)
                }
                KeyGuide::Cancel => key_intent(VK_ESCAPE, plain()) == Some(Intent::Cancel),
                KeyGuide::EditCell
                | KeyGuide::ClearCell
                | KeyGuide::BreakLine
                | KeyGuide::Advance => editing_bound(guide),
            };
            assert!(bound, "{guide:?}");
        }
    }

    #[test]
    fn keys_map_to_the_same_intents_as_on_x11() {
        assert_eq!(
            key_intent(VK_DOWN, Modifiers::new(true, false, false)),
            Some(Intent::Move {
                step: CursorStep::Down,
                mode: SelectionMode::Extend
            })
        );
        assert_eq!(
            key_intent(VK_END, control()),
            Some(Intent::Move {
                step: CursorStep::Bottom,
                mode: SelectionMode::Collapse
            })
        );
    }

    #[test]
    fn control_keys_copy_select_jump_and_open() {
        assert_eq!(key_intent(VK_C, control()), Some(Intent::Copy));
        assert_eq!(key_intent(VK_A, control()), Some(Intent::SelectAll));
        assert_eq!(key_intent(VK_G, control()), Some(Intent::OpenGoTo));
        assert_eq!(key_intent(VK_P, control_shift()), Some(Intent::OpenPalette));
        assert_eq!(
            key_intent(VK_U, control_shift()),
            Some(Intent::ToggleDetail)
        );
        assert_eq!(
            key_intent(VK_T, Modifiers::new(true, true, true)),
            Some(Intent::Theme(ThemeStep::Previous))
        );
    }

    #[test]
    fn control_keys_zoom_and_plain_keys_edit() {
        assert_eq!(
            key_intent(VK_OEM_1, control()),
            Some(Intent::Zoom(ZoomStep::In))
        );
        assert_eq!(
            key_intent(VK_NUMPAD0, control()),
            Some(Intent::Zoom(ZoomStep::Reset))
        );
        assert_eq!(key_intent(VK_ESCAPE, plain()), Some(Intent::Cancel));
        assert_eq!(key_intent(VK_RETURN, plain()), Some(Intent::Confirm));
        assert_eq!(key_intent(VK_BACK, plain()), Some(Intent::Erase));
        assert_eq!(key_intent(VK_A, plain()), None);
    }

    #[test]
    fn printable_characters_are_typed_including_japanese() {
        assert_eq!(typed_intent('a', false), Some(Intent::Type('a')));
        assert_eq!(typed_intent('a', true), None);
        assert_eq!(typed_intent('\r', false), None);
        assert_eq!(typed_intent('\t', false), None);
        assert_eq!(typed_intent('あ', false), Some(Intent::Type('あ')));
    }

    #[test]
    fn the_mouse_selects_scrolls_and_zooms() {
        assert_eq!(
            press_intent(10, 20, control()),
            Some(Intent::Point {
                at: PixelPoint::new(10, 20),
                mode: SelectionMode::Add
            })
        );
        assert_eq!(press_intent(-1, 20, plain()), None);
        assert_eq!(
            wheel_intent(120, plain()),
            Some(Intent::Scroll(Scroll::Up { rows: WHEEL_ROWS }))
        );
        assert_eq!(
            wheel_intent(-120, Modifiers::new(true, false, false)),
            Some(Intent::Scroll(Scroll::Right {
                columns: WHEEL_COLUMNS
            }))
        );
        assert_eq!(
            wheel_intent(120, control()),
            Some(Intent::Zoom(ZoomStep::In))
        );
        assert_eq!(
            sideways_wheel_intent(-120),
            Some(Intent::Scroll(Scroll::Left {
                columns: WHEEL_COLUMNS
            }))
        );
        assert_eq!(wheel_intent(0, plain()), None);
    }
}
