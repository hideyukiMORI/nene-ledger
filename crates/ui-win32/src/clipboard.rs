//! クリップボードに文字列を置き、読む（`CF_UNICODETEXT`・Excel と行き来できる形）。

use windows::Win32::Foundation::{HANDLE, HGLOBAL, HWND};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard,
    SetClipboardData,
};
use windows::Win32::System::Memory::{
    GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock,
};
use windows::Win32::System::Ole::CF_UNICODETEXT;

use crate::win32_failure::Win32Failure;

/// 文字列 `text` を、窓 `window` を持ち主としてクリップボードに置く（UTF-16 ＋ 終端）。置いた後のメモリは OS のもの。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 のクリップボードと GlobalAlloc を呼ぶ（ADR 0002 の 4）"
)]
pub(crate) fn put(window: HWND, text: &str) -> Result<(), Win32Failure> {
    let wide: Vec<u16> = text.encode_utf16().chain(core::iter::once(0)).collect();
    let bytes = wide.len().checked_mul(2).ok_or(Win32Failure::OutOfRange)?;
    // SAFETY: 自分の窓でクリップボードを開く。下で必ず閉じる。
    unsafe { OpenClipboard(Some(window)) }?;
    let placed = (|| -> Result<(), Win32Failure> {
        // SAFETY: 開いたクリップボードを空にする。
        unsafe { EmptyClipboard() }?;
        // SAFETY: 動かせるメモリを確保する。失敗すれば Err。
        let memory: HGLOBAL = unsafe { GlobalAlloc(GMEM_MOVEABLE, bytes) }?;
        // SAFETY: 確保したメモリを固定して書く。長さは wide の UTF-16 の単位の数 × 2 バイトで、確保した大きさと同じ。
        unsafe {
            let target = GlobalLock(memory).cast::<u16>();
            if target.is_null() {
                return Err(Win32Failure::OutOfRange);
            }
            core::ptr::copy_nonoverlapping(wide.as_ptr(), target, wide.len());
            let _still_locked: Result<(), _> = GlobalUnlock(memory);
        }
        // SAFETY: 書いたメモリを渡す。渡した後は OS が持つ（解放しない）。
        unsafe { SetClipboardData(u32::from(CF_UNICODETEXT.0), Some(HANDLE(memory.0))) }?;
        Ok(())
    })();
    // SAFETY: 上で開いたクリップボードを閉じる。
    let _closed: Result<(), _> = unsafe { CloseClipboard() };
    placed
}

/// クリップボードの文字列（`CF_UNICODETEXT`）を、窓 `window` で開いて読む（貼り付け・CLIP-01）。文字列が無い・開けない・
/// 読めなければ `None`。終端の NUL まで（無ければメモリの大きさまで）。対にならないサロゲートは置き換える。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 のクリップボードと GlobalLock を呼ぶ（ADR 0002 の 4・CLIP-01）"
)]
pub(crate) fn take(window: HWND) -> Option<String> {
    let format = u32::from(CF_UNICODETEXT.0);
    // SAFETY: 形の有無を尋ねるだけ。
    unsafe { IsClipboardFormatAvailable(format) }.ok()?;
    // SAFETY: 自分の窓でクリップボードを開く。下で必ず閉じる。
    unsafe { OpenClipboard(Some(window)) }.ok()?;
    let text = (|| -> Option<String> {
        // SAFETY: 開いたクリップボードの持ち物のハンドル。持ち主は OS で、解放しない。
        let handle = unsafe { GetClipboardData(format) }.ok()?;
        let memory = HGLOBAL(handle.0);
        // SAFETY: ハンドルのメモリの大きさを尋ねる。
        let bytes = unsafe { GlobalSize(memory) };
        let units = bytes / 2;
        // SAFETY: 固定して、大きさの内側（UTF-16 の単位 units 個）だけを読み、読んだら固定を外す。
        unsafe {
            let source = GlobalLock(memory).cast::<u16>().cast_const();
            if source.is_null() {
                return None;
            }
            let wide = core::slice::from_raw_parts(source, units);
            let end = wide.iter().position(|&unit| unit == 0).unwrap_or(units);
            let text = String::from_utf16_lossy(wide.get(..end).unwrap_or_default());
            let _still_locked: Result<(), _> = GlobalUnlock(memory);
            Some(text)
        }
    })();
    // SAFETY: 上で開いたクリップボードを閉じる。
    let _closed: Result<(), _> = unsafe { CloseClipboard() };
    text
}
