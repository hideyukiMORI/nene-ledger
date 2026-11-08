//! 窓に落とされたファイル（`DragAcceptFiles`・`WM_DROPFILES`・CLIP-04・GitHub Issue #243）。

use windows::Win32::Foundation::{HWND, WPARAM};
use windows::Win32::UI::Shell::{DragAcceptFiles, DragFinish, DragQueryFileW, HDROP};

/// `DragQueryFileW` の数を問う番号（落とされたファイルの数を返す）。
const COUNT_FILES: u32 = u32::MAX;

/// 窓 `window` にファイルを落とせるようにする（落とされると `WM_DROPFILES` が届く）。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の DragAcceptFiles を呼ぶ（ADR 0002 の 4）"
)]
pub(crate) fn accept(window: HWND) {
    // SAFETY: 自分の窓の印を変えるだけ。
    unsafe {
        DragAcceptFiles(window, true);
    }
}

/// `WM_DROPFILES` の `wparam`（落とされた物のハンドル）から、ファイルの道筋を落とした順に取り出し、ハンドルを手放す。
/// UTF-16 として読めない字は置き換える。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の DragQueryFileW / DragFinish を呼ぶ（ADR 0002 の 4）"
)]
pub(crate) fn take(wparam: WPARAM) -> Vec<String> {
    let drop = HDROP(core::ptr::with_exposed_provenance_mut(wparam.0));
    // SAFETY: `drop` は `WM_DROPFILES` で渡されたハンドルで、`DragFinish` までは生きている。数を問うときは書き込み先を
    // 渡さない。
    let count = unsafe { DragQueryFileW(drop, COUNT_FILES, None) };
    let paths = (0..count)
        .filter_map(|index| {
            // SAFETY: 同じハンドル。長さ（終わりの 0 を除く字数）を問うときは書き込み先を渡さない。
            let length = unsafe { DragQueryFileW(drop, index, None) };
            let mut wide = vec![0_u16; usize::try_from(length).ok()?.saturating_add(1)];
            // SAFETY: `wide` は長さ ＋ 終わりの 0 の分の、この呼び出しの間だけ生きている書き込み先。
            let written = unsafe { DragQueryFileW(drop, index, Some(&mut wide)) };
            let written = usize::try_from(written).ok()?;
            Some(String::from_utf16_lossy(wide.get(..written)?))
        })
        .collect();
    // SAFETY: 取り出し終えたハンドルを手放す（以後は使わない）。
    unsafe {
        DragFinish(drop);
    }
    paths
}
