//! IME（IMM32）との対話（FR-020・ADR 0026 の決定 4）。`WM_IME_COMPOSITION` で確定した文字列と確定前の文字列を読み、
//! 候補の窓を編集中のセルの左下に置く。確定前の文字列はセルの中に自分で描くので、IME の組み立ての窓は出させない
//! （`WM_IME_SETCONTEXT`）。出発点は `NeNeDocket` の同じ名の部品（ADR 0020・施主の実機で確かめた）。

use core::ffi::c_void;

use nene_ledger_core::pixel_rect::PixelRect;
use nene_ledger_core::preedit::Preedit;
use windows::Win32::Foundation::{HWND, LPARAM, POINT, RECT};
use windows::Win32::UI::Input::Ime::{
    CANDIDATEFORM, CFS_CANDIDATEPOS, CFS_POINT, COMPOSITIONFORM, GCS_COMPSTR, GCS_CURSORPOS,
    GCS_RESULTSTR, HIMC, IME_COMPOSITION_STRING, ISC_SHOWUICOMPOSITIONWINDOW,
    ImmGetCompositionStringW, ImmGetContext, ImmReleaseContext, ImmSetCandidateWindow,
    ImmSetCompositionWindow,
};

use crate::composition;

/// `WM_IME_SETCONTEXT` の lparam から、IME の組み立ての窓を出す印を落とす。
#[must_use]
pub(crate) fn without_composition_window(lparam: LPARAM) -> LPARAM {
    let shown = isize::try_from(ISC_SHOWUICOMPOSITIONWINDOW).unwrap_or(0);
    LPARAM(lparam.0 & !shown)
}

/// `WM_IME_COMPOSITION`: 確定した文字列（あれば）と、確定前の文字列（組み立てが消えていれば空）。
#[must_use]
pub(crate) fn read_composition(window: HWND, lparam: LPARAM) -> (Option<String>, Preedit) {
    let flags =
        IME_COMPOSITION_STRING(u32::try_from(lparam.0.cast_unsigned() & 0xFFFF_FFFF).unwrap_or(0));
    with_context(window, |context| {
        let result = flags
            .contains(GCS_RESULTSTR)
            .then(|| String::from_utf16_lossy(&string_units(context, GCS_RESULTSTR)));
        let preedit = if flags.contains(GCS_COMPSTR) {
            let units = string_units(context, GCS_COMPSTR);
            let cursor = if flags.contains(GCS_CURSORPOS) {
                value_of(context, GCS_CURSORPOS)
            } else {
                units.len()
            };
            composition::preedit_of(&units, cursor)
        } else {
            Preedit::new("", 0)
        };
        (result, preedit)
    })
    .unwrap_or((None, Preedit::new("", 0)))
}

/// 候補の窓をセルの下に、組み立ての窓（出さないが位置は要る）をセルの左上に置く。
#[expect(
    unsafe_code,
    reason = "RS-010: IMM32 の ImmSetCompositionWindow / ImmSetCandidateWindow を呼ぶ（ADR 0002 の 4）"
)]
pub(crate) fn place_windows(window: HWND, cell: PixelRect) {
    let pixel = |value: u32| i32::try_from(value).unwrap_or(i32::MAX);
    let (left, top) = (pixel(cell.origin().x()), pixel(cell.origin().y()));
    let area = RECT {
        left,
        top,
        right: left.saturating_add(pixel(cell.size().width())),
        bottom: top.saturating_add(pixel(cell.size().height())),
    };
    let composition_form = COMPOSITIONFORM {
        dwStyle: CFS_POINT,
        ptCurrentPos: POINT {
            x: area.left,
            y: area.top,
        },
        rcArea: area,
    };
    let candidate_form = CANDIDATEFORM {
        dwIndex: 0,
        dwStyle: CFS_CANDIDATEPOS,
        ptCurrentPos: POINT {
            x: area.left,
            y: area.bottom,
        },
        rcArea: area,
    };
    let _placed = with_context(window, |context| {
        // SAFETY: 2 つの form はこの関数の局所で、呼び出しの間だけ読まれる。context は with_context が持つ。
        unsafe {
            let _composition_set = ImmSetCompositionWindow(context, &raw const composition_form);
            let _candidate_set = ImmSetCandidateWindow(context, &raw const candidate_form);
        }
    });
}

/// 窓の入力文脈を取り、閉じる。文脈が無ければ（IME の無い環境）`None`。
#[expect(
    unsafe_code,
    reason = "RS-010: IMM32 の ImmGetContext / ImmReleaseContext を呼ぶ（ADR 0002 の 4）"
)]
fn with_context<T>(window: HWND, read: impl FnOnce(HIMC) -> T) -> Option<T> {
    // SAFETY: 自分の窓の文脈を取る。空なら無い。
    let context = unsafe { ImmGetContext(window) };
    if context.0.is_null() {
        return None;
    }
    let value = read(context);
    // SAFETY: 上で取った文脈を同じ窓で返す。
    let _released = unsafe { ImmReleaseContext(window, context) };
    Some(value)
}

/// 文字列の種類（確定前・確定）を UTF-16 の単位で読む。長さを問い、その分の緩衝を渡す。
#[expect(
    unsafe_code,
    reason = "RS-010: IMM32 の ImmGetCompositionStringW を呼ぶ（ADR 0002 の 4）"
)]
fn string_units(context: HIMC, index: IME_COMPOSITION_STRING) -> Vec<u16> {
    // SAFETY: 緩衝を渡さなければ長さ（バイト）が返る。負は失敗。
    let bytes = unsafe { ImmGetCompositionStringW(context, index, None, 0) };
    let Ok(length) = u32::try_from(bytes) else {
        return Vec::new();
    };
    let mut buffer = vec![0_u16; usize::try_from(length / 2).unwrap_or(0)];
    // SAFETY: buffer は length バイト以上あり、呼び出しの間生きている。
    let written = unsafe {
        ImmGetCompositionStringW(
            context,
            index,
            Some(buffer.as_mut_ptr().cast::<c_void>()),
            length,
        )
    };
    let units = usize::try_from(written).map_or(0, |count| count / 2);
    buffer.truncate(units);
    buffer
}

/// 値を返す種類（カーソルの位置）を読む。
#[expect(
    unsafe_code,
    reason = "RS-010: IMM32 の ImmGetCompositionStringW を呼ぶ（ADR 0002 の 4）"
)]
fn value_of(context: HIMC, index: IME_COMPOSITION_STRING) -> usize {
    // SAFETY: この種類では戻り値そのものが値。緩衝は要らない。
    let value = unsafe { ImmGetCompositionStringW(context, index, None, 0) };
    usize::try_from(value).unwrap_or(0)
}
