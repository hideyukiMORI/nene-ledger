//! キー・ホイール・クリックを意図に変える（UI は操作を意図にするだけ・ARC-011）。

use nene_ledger_application::column_shift::ColumnShift;
use nene_ledger_application::cursor_step::CursorStep;
use nene_ledger_application::intent::Intent;
use nene_ledger_application::scroll::Scroll;
use nene_ledger_application::selection_mode::SelectionMode;
use nene_ledger_application::theme_step::ThemeStep;
use nene_ledger_application::zoom_step::ZoomStep;
use nene_ledger_core::pixel_point::PixelPoint;
use x11rb::protocol::xproto::KeyButMask;

/// ホイールの 1 刻みで動かす行の数。
const WHEEL_ROWS: u64 = 3;
/// 横のホイールの 1 刻みで動かす列の数。
const WHEEL_COLUMNS: usize = 1;

/// keysym（X11 の `keysymdef.h`）。
const XK_HOME: u32 = 0xff50;
const XK_LEFT: u32 = 0xff51;
const XK_UP: u32 = 0xff52;
const XK_RIGHT: u32 = 0xff53;
const XK_DOWN: u32 = 0xff54;
const XK_PAGE_UP: u32 = 0xff55;
const XK_PAGE_DOWN: u32 = 0xff56;
const XK_END: u32 = 0xff57;
const XK_SPACE: u32 = 0x20;
const XK_A: u32 = 0x61;
const XK_C: u32 = 0x63;
const XK_S: u32 = 0x73;
const XK_Y: u32 = 0x79;
const XK_Z: u32 = 0x7a;
const XK_G: u32 = 0x67;
const XK_U: u32 = 0x75;
const XK_T: u32 = 0x74;
const XK_P: u32 = 0x70;
const XK_F1: u32 = 0xffbe;
const XK_ESCAPE: u32 = 0xff1b;
const XK_RETURN: u32 = 0xff0d;
const XK_KP_ENTER: u32 = 0xff8d;
const XK_BACKSPACE: u32 = 0xff08;
const XK_DELETE: u32 = 0xffff;
const XK_TAB: u32 = 0xff09;
/// Shift+Tab（多くの配列で Tab に Shift を足すとこの keysym になる）。
const XK_ISO_LEFT_TAB: u32 = 0xfe20;
const XK_F2: u32 = 0xffbf;
/// 拡大のキー（US 配列の `=`（Shift で `+`）・JIS 配列の `;`（Shift で `+`）・`-`・`0`・テンキー）。
const XK_EQUAL: u32 = 0x3d;
const XK_PLUS: u32 = 0x2b;
const XK_MINUS: u32 = 0x2d;
const XK_0: u32 = 0x30;
const XK_KP_ADD: u32 = 0xffab;
const XK_KP_SUBTRACT: u32 = 0xffad;
const XK_KP_0: u32 = 0xffb0;

/// マウスのボタン（X11 の約束で 1 が左、4・5 が縦のホイール、6・7 が横のホイール）。
const LEFT: u8 = 1;
const WHEEL_UP: u8 = 4;
const WHEEL_DOWN: u8 = 5;
const WHEEL_LEFT: u8 = 6;
const WHEEL_RIGHT: u8 = 7;

/// キーの意図。`typed` はこのキーで打たれる字（半角の英数字と記号・無ければ `None`）。
/// 矢印・PageUp・PageDown・Home・End でカーソルを動かす。Shift で選択を広げ、Ctrl+Home / Ctrl+End は表の先頭 / 末尾。
/// Ctrl+C はコピー、Ctrl+Shift+U は詳細ペインの開閉（Excel で数式バーを広げるキー）、Ctrl+Shift+T / Ctrl+Shift+Alt+T は
/// 配色を次 / 前へ（試しに替える。名前から選ぶのはコマンドパレット）。
pub(crate) fn key_intent(keysym: u32, typed: Option<char>, state: KeyButMask) -> Option<Intent> {
    let control = state.contains(KeyButMask::CONTROL);
    if let Some(intent) = line_intent(keysym, control, state.contains(KeyButMask::SHIFT)) {
        return Some(intent);
    }
    if let Some(intent) = editing_intent(keysym, typed, state) {
        return Some(intent);
    }
    if control && keysym == XK_C {
        return Some(Intent::Copy);
    }
    if control && let Some(intent) = save_intent(keysym, state.contains(KeyButMask::SHIFT)) {
        return Some(intent);
    }
    if control && let Some(intent) = undo_intent(keysym, state.contains(KeyButMask::SHIFT)) {
        return Some(intent);
    }
    if control && !state.contains(KeyButMask::SHIFT) && keysym == XK_A {
        return Some(Intent::SelectAll);
    }
    if control && !state.contains(KeyButMask::SHIFT) && keysym == XK_G {
        return Some(Intent::OpenGoTo);
    }
    if control && let Some(intent) = sign_intent(keysym) {
        return Some(intent);
    }
    let control_shift = control && state.contains(KeyButMask::SHIFT);
    if control_shift && keysym == XK_P {
        return Some(Intent::OpenPalette);
    }
    if control_shift && keysym == XK_U {
        return Some(Intent::ToggleDetail);
    }
    if control_shift && keysym == XK_T {
        let step = if state.contains(KeyButMask::MOD1) {
            ThemeStep::Previous
        } else {
            ThemeStep::Next
        };
        return Some(Intent::Theme(step));
    }
    let step = match keysym {
        XK_UP => CursorStep::Up,
        XK_DOWN => CursorStep::Down,
        XK_LEFT => CursorStep::Left,
        XK_RIGHT => CursorStep::Right,
        XK_PAGE_UP => CursorStep::PageUp,
        XK_PAGE_DOWN => CursorStep::PageDown,
        XK_HOME if control => CursorStep::Top,
        XK_HOME => CursorStep::RowStart,
        XK_END if control => CursorStep::Bottom,
        XK_END => CursorStep::RowEnd,
        _ => return None,
    };
    Some(Intent::Move {
        step,
        mode: mode(state),
    })
}

/// マウスのボタンの意図。左は窓の中の位置 `(x, y)` のセルを選ぶ（Shift で広げる）。縦のホイールは縦に、
/// Shift を押した縦のホイールと横のホイールは横に、表示だけを動かす。
pub(crate) fn button_intent(button: u8, state: KeyButMask, x: i16, y: i16) -> Option<Intent> {
    let sideways = state.contains(KeyButMask::SHIFT);
    let zooming = state.contains(KeyButMask::CONTROL);
    let scroll = match button {
        WHEEL_UP if zooming => return Some(Intent::Zoom(ZoomStep::In)),
        WHEEL_DOWN if zooming => return Some(Intent::Zoom(ZoomStep::Out)),
        LEFT => {
            let at = PixelPoint::new(u32::try_from(x).ok()?, u32::try_from(y).ok()?);
            return Some(Intent::Point {
                at,
                mode: pointer_mode(state),
            });
        }
        WHEEL_UP if sideways => Scroll::Left {
            columns: WHEEL_COLUMNS,
        },
        WHEEL_DOWN if sideways => Scroll::Right {
            columns: WHEEL_COLUMNS,
        },
        WHEEL_UP => Scroll::Up { rows: WHEEL_ROWS },
        WHEEL_DOWN => Scroll::Down { rows: WHEEL_ROWS },
        WHEEL_LEFT => Scroll::Left {
            columns: WHEEL_COLUMNS,
        },
        WHEEL_RIGHT => Scroll::Right {
            columns: WHEEL_COLUMNS,
        },
        _ => return None,
    };
    Some(Intent::Scroll(scroll))
}

/// パレットを開く・決める・やめる・消す・字を打つ・セルを直す（ADR 0026 の決定 1・2）。
/// Ctrl と組んだ字は字として打たない（Ctrl+C など）。Alt+Enter はセルの中の改行、Shift+Enter・Tab・Shift+Tab は確定して動く。
fn editing_intent(keysym: u32, typed: Option<char>, state: KeyButMask) -> Option<Intent> {
    let shift = state.contains(KeyButMask::SHIFT);
    match keysym {
        XK_F1 => Some(Intent::OpenPalette),
        XK_F2 => Some(Intent::Edit),
        XK_ESCAPE => Some(Intent::Cancel),
        XK_RETURN | XK_KP_ENTER if state.contains(KeyButMask::MOD1) => Some(Intent::BreakLine),
        XK_RETURN | XK_KP_ENTER if shift => Some(Intent::Advance(CursorStep::Up)),
        XK_RETURN | XK_KP_ENTER => Some(Intent::Confirm),
        XK_TAB if shift => Some(Intent::Advance(CursorStep::Left)),
        XK_ISO_LEFT_TAB => Some(Intent::Advance(CursorStep::Left)),
        XK_TAB => Some(Intent::Advance(CursorStep::Right)),
        XK_BACKSPACE => Some(Intent::Erase),
        XK_DELETE => Some(Intent::Delete),
        _ => typed
            .filter(|_| !state.contains(KeyButMask::CONTROL))
            .map(Intent::Type),
    }
}

/// 行全体・列全体を選ぶキーと、列を移すキー（Issue #62）。Shift+Space は字を打つより先に受ける。
fn line_intent(keysym: u32, control: bool, shift: bool) -> Option<Intent> {
    match (keysym, control, shift) {
        (XK_SPACE, false, true) => Some(Intent::SelectRows),
        (XK_SPACE, true, false) => Some(Intent::SelectColumns),
        (XK_PAGE_UP, true, true) => Some(Intent::MoveColumns(ColumnShift::Left)),
        (XK_PAGE_DOWN, true, true) => Some(Intent::MoveColumns(ColumnShift::Right)),
        _ => None,
    }
}

/// Ctrl と組んで元に戻す・やり直すキー（Ctrl+Z・Ctrl+Y・Ctrl+Shift+Z・UNDO-01）。Shift で `Z` が大文字の keysym になる配列もある。
/// Ctrl と組んで保存する（Ctrl+S）・名前を付けて保存する（Ctrl+Shift+S・ADR 0028 の決定 4）キー。
/// Shift を押した keysym は大文字（`S`・0x53）で届くこともある（Ctrl+Shift+Z と同じ）。
fn save_intent(keysym: u32, shift: bool) -> Option<Intent> {
    match (keysym, shift) {
        (XK_S, false) => Some(Intent::Save),
        (XK_S | 0x53, true) => Some(Intent::SaveAs),
        _ => None,
    }
}

fn undo_intent(keysym: u32, shift: bool) -> Option<Intent> {
    match (keysym, shift) {
        (XK_Z, false) => Some(Intent::Undo),
        (XK_Z | 0x5a, true) | (XK_Y, false) => Some(Intent::Redo),
        _ => None,
    }
}

/// Ctrl と組んで行と列を足し消しするキー（Ctrl++ と Ctrl+-・Excel と同じ・ADR 0027）と、文字の大きさを戻す Ctrl+0。文字の大きさは Ctrl+ホイールと
/// Ctrl+0（ADR 0020 の施主の判断・2026-11-06）。JIS の Ctrl++ は Shift+; で、keysym は `plus` になる。
fn sign_intent(keysym: u32) -> Option<Intent> {
    match keysym {
        XK_EQUAL | XK_PLUS | XK_KP_ADD => Some(Intent::InsertHere),
        XK_MINUS | XK_KP_SUBTRACT => Some(Intent::RemoveHere),
        XK_0 | XK_KP_0 => Some(Intent::Zoom(ZoomStep::Reset)),
        _ => None,
    }
}

/// クリックの選択の仕方。Shift で広げ、Ctrl で範囲を足す（Issue #46・両方なら広げる）。
fn pointer_mode(state: KeyButMask) -> SelectionMode {
    if state.contains(KeyButMask::SHIFT) {
        SelectionMode::Extend
    } else if state.contains(KeyButMask::CONTROL) {
        SelectionMode::Add
    } else {
        SelectionMode::Collapse
    }
}

/// Shift を押していれば選択を広げる。
fn mode(state: KeyButMask) -> SelectionMode {
    if state.contains(KeyButMask::SHIFT) {
        SelectionMode::Extend
    } else {
        SelectionMode::Collapse
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn moving(step: CursorStep, mode: SelectionMode) -> Intent {
        Intent::Move { step, mode }
    }

    /// 修飾キーを押していない。
    fn plain() -> KeyButMask {
        KeyButMask::from(0_u16)
    }

    #[test]
    fn keys_move_the_cursor() {
        let collapse = SelectionMode::Collapse;
        assert_eq!(
            key_intent(XK_DOWN, None, plain()),
            Some(moving(CursorStep::Down, collapse))
        );
        assert_eq!(
            key_intent(XK_UP, None, plain()),
            Some(moving(CursorStep::Up, collapse))
        );
        assert_eq!(
            key_intent(XK_LEFT, None, plain()),
            Some(moving(CursorStep::Left, collapse))
        );
        assert_eq!(
            key_intent(XK_RIGHT, None, plain()),
            Some(moving(CursorStep::Right, collapse))
        );
        assert_eq!(
            key_intent(XK_PAGE_DOWN, None, plain()),
            Some(moving(CursorStep::PageDown, collapse))
        );
        assert_eq!(
            key_intent(XK_PAGE_UP, None, plain()),
            Some(moving(CursorStep::PageUp, collapse))
        );
        assert_eq!(
            key_intent(XK_HOME, None, plain()),
            Some(moving(CursorStep::RowStart, collapse))
        );
        assert_eq!(
            key_intent(XK_END, None, plain()),
            Some(moving(CursorStep::RowEnd, collapse))
        );
        assert_eq!(key_intent(0x61, None, plain()), None);
    }

    #[test]
    fn control_jumps_to_the_ends_and_shift_extends() {
        let control = KeyButMask::CONTROL;
        assert_eq!(
            key_intent(XK_HOME, None, control),
            Some(moving(CursorStep::Top, SelectionMode::Collapse))
        );
        assert_eq!(
            key_intent(XK_END, None, control),
            Some(moving(CursorStep::Bottom, SelectionMode::Collapse))
        );
        assert_eq!(
            key_intent(XK_C, None, KeyButMask::CONTROL),
            Some(Intent::Copy)
        );
        assert_eq!(key_intent(XK_C, None, plain()), None);
        let control_shift =
            KeyButMask::from(u16::from(KeyButMask::CONTROL) | u16::from(KeyButMask::SHIFT));
        assert_eq!(
            key_intent(XK_U, None, control_shift),
            Some(Intent::ToggleDetail)
        );
        assert_eq!(key_intent(XK_U, None, KeyButMask::CONTROL), None);
        assert_eq!(
            key_intent(XK_T, None, control_shift),
            Some(Intent::Theme(ThemeStep::Next))
        );
        let with_alt = KeyButMask::from(u16::from(control_shift) | u16::from(KeyButMask::MOD1));
        assert_eq!(
            key_intent(XK_T, None, with_alt),
            Some(Intent::Theme(ThemeStep::Previous))
        );
        assert_eq!(
            key_intent(XK_DOWN, None, KeyButMask::SHIFT),
            Some(moving(CursorStep::Down, SelectionMode::Extend))
        );
    }

    #[test]
    fn space_selects_whole_lines_and_page_keys_move_columns() {
        let control_shift =
            KeyButMask::from(u16::from(KeyButMask::CONTROL) | u16::from(KeyButMask::SHIFT));
        // Shift+Space は字を打つより先に行全体。Space だけなら字。
        assert_eq!(
            key_intent(XK_SPACE, Some(' '), KeyButMask::SHIFT),
            Some(Intent::SelectRows)
        );
        assert_eq!(
            key_intent(XK_SPACE, Some(' '), KeyButMask::CONTROL),
            Some(Intent::SelectColumns)
        );
        assert_eq!(
            key_intent(XK_SPACE, Some(' '), plain()),
            Some(Intent::Type(' '))
        );
        assert_eq!(
            key_intent(XK_PAGE_UP, None, control_shift),
            Some(Intent::MoveColumns(ColumnShift::Left))
        );
        assert_eq!(
            key_intent(XK_PAGE_DOWN, None, control_shift),
            Some(Intent::MoveColumns(ColumnShift::Right))
        );
        // Shift だけなら 1 画面ぶん広げるまま。
        assert_eq!(
            key_intent(XK_PAGE_DOWN, None, KeyButMask::SHIFT),
            Some(moving(CursorStep::PageDown, SelectionMode::Extend))
        );
    }

    /// 編集のキーの一覧の行（ADR 0026）が、書いたとおりに割り当てられている。
    fn editing_bound(guide: nene_ledger_application::key_guide::KeyGuide) -> bool {
        use nene_ledger_application::key_guide::KeyGuide;
        match guide {
            KeyGuide::EditCell => key_intent(XK_F2, None, plain()) == Some(Intent::Edit),
            KeyGuide::ClearCell => key_intent(XK_DELETE, None, plain()) == Some(Intent::Delete),
            KeyGuide::BreakLine => {
                key_intent(XK_RETURN, None, KeyButMask::MOD1) == Some(Intent::BreakLine)
            }
            KeyGuide::Advance => {
                key_intent(XK_TAB, None, plain()) == Some(Intent::Advance(CursorStep::Right))
                    && key_intent(XK_ISO_LEFT_TAB, None, KeyButMask::SHIFT)
                        == Some(Intent::Advance(CursorStep::Left))
                    && key_intent(XK_TAB, None, KeyButMask::SHIFT)
                        == Some(Intent::Advance(CursorStep::Left))
                    && key_intent(XK_RETURN, None, KeyButMask::SHIFT)
                        == Some(Intent::Advance(CursorStep::Up))
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
        let shift = KeyButMask::SHIFT;
        let control = KeyButMask::CONTROL;
        let control_shift = KeyButMask::from(u16::from(control) | u16::from(shift));
        let with_alt = KeyButMask::from(u16::from(control_shift) | u16::from(KeyButMask::MOD1));
        let point = |mode| {
            Some(Intent::Point {
                at: PixelPoint::new(5, 5),
                mode,
            })
        };
        for guide in KeyGuide::ALL {
            let bound = match guide {
                KeyGuide::MoveCursor => {
                    key_intent(XK_LEFT, None, plain())
                        == Some(moving(CursorStep::Left, SelectionMode::Collapse))
                }
                KeyGuide::ExtendSelection => {
                    key_intent(XK_DOWN, None, shift)
                        == Some(moving(CursorStep::Down, SelectionMode::Extend))
                }
                KeyGuide::Page => {
                    key_intent(XK_PAGE_DOWN, None, plain())
                        == Some(moving(CursorStep::PageDown, SelectionMode::Collapse))
                }
                KeyGuide::RowEnds => {
                    key_intent(XK_END, None, plain())
                        == Some(moving(CursorStep::RowEnd, SelectionMode::Collapse))
                }
                KeyGuide::ExtendClick => {
                    button_intent(LEFT, shift, 5, 5) == point(SelectionMode::Extend)
                }
                KeyGuide::AddRange => {
                    button_intent(LEFT, control, 5, 5) == point(SelectionMode::Add)
                }
                KeyGuide::Sideways => {
                    button_intent(WHEEL_DOWN, shift, 0, 0)
                        == Some(Intent::Scroll(Scroll::Right { columns: 1 }))
                }
                KeyGuide::WheelZoom => {
                    button_intent(WHEEL_UP, control, 0, 0) == Some(Intent::Zoom(ZoomStep::In))
                }
                KeyGuide::NextTheme => {
                    key_intent(XK_T, None, control_shift) == Some(Intent::Theme(ThemeStep::Next))
                }
                KeyGuide::PreviousTheme => {
                    key_intent(XK_T, None, with_alt) == Some(Intent::Theme(ThemeStep::Previous))
                }
                KeyGuide::Palette => {
                    key_intent(XK_P, None, control_shift) == Some(Intent::OpenPalette)
                        && key_intent(XK_F1, None, plain()) == Some(Intent::OpenPalette)
                }
                KeyGuide::Cancel => key_intent(XK_ESCAPE, None, plain()) == Some(Intent::Cancel),
                KeyGuide::EditCell
                | KeyGuide::ClearCell
                | KeyGuide::BreakLine
                | KeyGuide::Advance => editing_bound(guide),
            };
            assert!(bound, "{guide:?}");
        }
    }

    #[test]
    fn the_wheel_scrolls_and_shift_turns_it_sideways() {
        assert_eq!(
            button_intent(4, plain(), 0, 0),
            Some(Intent::Scroll(Scroll::Up { rows: 3 }))
        );
        assert_eq!(
            button_intent(5, plain(), 0, 0),
            Some(Intent::Scroll(Scroll::Down { rows: 3 }))
        );
        assert_eq!(
            button_intent(5, KeyButMask::SHIFT, 0, 0),
            Some(Intent::Scroll(Scroll::Right { columns: 1 }))
        );
        assert_eq!(
            button_intent(4, KeyButMask::SHIFT, 0, 0),
            Some(Intent::Scroll(Scroll::Left { columns: 1 }))
        );
        assert_eq!(
            button_intent(6, plain(), 0, 0),
            Some(Intent::Scroll(Scroll::Left { columns: 1 }))
        );
        assert_eq!(
            button_intent(7, plain(), 0, 0),
            Some(Intent::Scroll(Scroll::Right { columns: 1 }))
        );
        assert_eq!(button_intent(2, plain(), 0, 0), None);
    }

    #[test]
    fn control_with_the_wheel_or_plus_minus_zero_zooms() {
        let control = KeyButMask::CONTROL;
        assert_eq!(
            button_intent(4, control, 0, 0),
            Some(Intent::Zoom(ZoomStep::In))
        );
        assert_eq!(
            button_intent(5, control, 0, 0),
            Some(Intent::Zoom(ZoomStep::Out))
        );
        for keysym in [XK_EQUAL, XK_PLUS, XK_KP_ADD] {
            assert_eq!(key_intent(keysym, None, control), Some(Intent::InsertHere));
        }
        for keysym in [XK_MINUS, XK_KP_SUBTRACT] {
            assert_eq!(key_intent(keysym, None, control), Some(Intent::RemoveHere));
        }
        for keysym in [XK_0, XK_KP_0] {
            assert_eq!(
                key_intent(keysym, None, control),
                Some(Intent::Zoom(ZoomStep::Reset))
            );
        }
        assert_eq!(key_intent(XK_MINUS, None, plain()), None);
    }

    #[test]
    fn palette_keys_and_typed_characters_become_intents() {
        let control_shift =
            KeyButMask::from(u16::from(KeyButMask::CONTROL) | u16::from(KeyButMask::SHIFT));
        assert_eq!(
            key_intent(XK_P, None, control_shift),
            Some(Intent::OpenPalette)
        );
        assert_eq!(key_intent(XK_F1, None, plain()), Some(Intent::OpenPalette));
        assert_eq!(key_intent(XK_ESCAPE, None, plain()), Some(Intent::Cancel));
        assert_eq!(key_intent(XK_RETURN, None, plain()), Some(Intent::Confirm));
        assert_eq!(
            key_intent(XK_KP_ENTER, None, plain()),
            Some(Intent::Confirm)
        );
        assert_eq!(key_intent(XK_BACKSPACE, None, plain()), Some(Intent::Erase));
    }

    #[test]
    fn typed_characters_become_intents_unless_control_is_held() {
        assert_eq!(
            key_intent(0x61, Some('a'), plain()),
            Some(Intent::Type('a'))
        );
        assert_eq!(
            key_intent(0x61, Some('A'), KeyButMask::SHIFT),
            Some(Intent::Type('A'))
        );
        // Ctrl と組んだ字は打たない（Ctrl+C はコピー）。
        assert_eq!(
            key_intent(XK_C, Some('c'), KeyButMask::CONTROL),
            Some(Intent::Copy)
        );
        // Ctrl+G は飛ぶ先の入力欄。
        assert_eq!(
            key_intent(0x67, Some('g'), KeyButMask::CONTROL),
            Some(Intent::OpenGoTo)
        );
        // Ctrl+A はすべてを選ぶ（字は打たない）。
        assert_eq!(
            key_intent(0x61, Some('a'), KeyButMask::CONTROL),
            Some(Intent::SelectAll)
        );
    }

    #[test]
    fn a_left_click_points_at_a_cell() {
        assert_eq!(
            button_intent(1, KeyButMask::SHIFT, 10, 20),
            Some(Intent::Point {
                at: PixelPoint::new(10, 20),
                mode: SelectionMode::Extend,
            })
        );
        assert_eq!(button_intent(1, plain(), -1, 20), None);
        // Ctrl+クリックは範囲を足す。Shift と一緒なら広げる。
        let add = |state| match button_intent(1, state, 10, 20) {
            Some(Intent::Point { mode, .. }) => Some(mode),
            _ => None,
        };
        assert_eq!(add(KeyButMask::CONTROL), Some(SelectionMode::Add));
        assert_eq!(
            add(KeyButMask::CONTROL | KeyButMask::SHIFT),
            Some(SelectionMode::Extend)
        );
    }

    #[test]
    fn control_z_and_y_undo_and_redo() {
        let control_shift =
            KeyButMask::from(u16::from(KeyButMask::CONTROL) | u16::from(KeyButMask::SHIFT));
        assert_eq!(
            key_intent(XK_Z, Some('z'), KeyButMask::CONTROL),
            Some(Intent::Undo)
        );
        assert_eq!(
            key_intent(XK_Y, Some('y'), KeyButMask::CONTROL),
            Some(Intent::Redo)
        );
        assert_eq!(
            key_intent(0x5a, Some('Z'), control_shift),
            Some(Intent::Redo)
        );
        assert_eq!(
            key_intent(XK_S, Some('s'), KeyButMask::CONTROL),
            Some(Intent::Save)
        );
        assert_eq!(
            key_intent(0x53, Some('S'), control_shift),
            Some(Intent::SaveAs)
        );
        assert_eq!(
            key_intent(XK_S, Some('s'), control_shift),
            Some(Intent::SaveAs)
        );
        assert_eq!(
            key_intent(XK_Z, Some('z'), plain()),
            Some(Intent::Type('z'))
        );
    }
}
