//! クリップボードに文字列を置く（`CF_UNICODETEXT`・Excel に貼れる形）。

use windows::Win32::Foundation::{HANDLE, HGLOBAL, HWND};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
};
use windows::Win32::System::Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalUnlock};
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
