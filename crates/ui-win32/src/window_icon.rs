//! 窓のアイコン（タスクバー・Alt+Tab・GitHub Issue #375）。
//!
//! exe に資源のアイコンを埋めるには `build.rs` が要り、規約で置けない（CNF-008）。そこで、`packaging/msix/icon.svg`
//! から作った BGRA の画素（`assets/icon-<辺>.bgra`・上の行から・まっすぐのアルファ）を exe に埋め、窓を作るときに
//! `CreateIcon` で作って `WM_SETICON` で付ける。大きさは窓のある画面の倍率での小さいアイコンと大きいアイコンの辺
//! （`GetSystemMetricsForDpi`）に、持っている辺のうちそれ以上でいちばん小さいもの（無ければいちばん大きいもの）を選ぶ。

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::HiDpi::{GetDpiForWindow, GetSystemMetricsForDpi};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateIcon, ICON_BIG, ICON_SMALL, SM_CXICON, SM_CXSMICON, SYSTEM_METRICS_INDEX, SendMessageW,
    WM_SETICON,
};

/// 持っている辺（画素）と、その BGRA の画素（辺 × 辺 × 4 バイト）。辺の小さい順。
const IMAGES: [(u32, &[u8]); 7] = [
    (16, include_bytes!("../assets/icon-16.bgra")),
    (20, include_bytes!("../assets/icon-20.bgra")),
    (24, include_bytes!("../assets/icon-24.bgra")),
    (32, include_bytes!("../assets/icon-32.bgra")),
    (40, include_bytes!("../assets/icon-40.bgra")),
    (48, include_bytes!("../assets/icon-48.bgra")),
    (64, include_bytes!("../assets/icon-64.bgra")),
];

/// 窓 `window` に小さいアイコンと大きいアイコンを付ける。作れなければ付けない（OS の既定の絵のまま・続ける）。
pub(crate) fn attach(window: HWND) {
    for (which, metric) in [(ICON_SMALL, SM_CXSMICON), (ICON_BIG, SM_CXICON)] {
        let (side, pixels) = image_for(wanted_side(window, metric));
        set_icon(window, which, side, pixels);
    }
}

/// 辺 `wanted` に合う画像：それ以上でいちばん小さい辺、無ければいちばん大きい辺。
fn image_for(wanted: u32) -> (u32, &'static [u8]) {
    IMAGES
        .iter()
        .copied()
        .find(|&(side, _)| side >= wanted)
        .or_else(|| IMAGES.last().copied())
        .unwrap_or((0, &[]))
}

/// 窓のある画面の倍率での、アイコンの辺（`metric`）。読めなければ 32。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の GetDpiForWindow・GetSystemMetricsForDpi を呼ぶ（ADR 0002 の 4）"
)]
fn wanted_side(window: HWND, metric: SYSTEM_METRICS_INDEX) -> u32 {
    // SAFETY: 自分の窓の dpi を読み、その dpi のアイコンの辺を読むだけ。失敗すれば 0 が返る。
    let side = unsafe { GetSystemMetricsForDpi(metric, GetDpiForWindow(window)) };
    u32::try_from(side)
        .ok()
        .filter(|&side| side > 0)
        .unwrap_or(32)
}

/// 辺 `side` の BGRA の画素 `pixels` からアイコンを作り、窓 `window` の `which`（小さい・大きい）に付ける。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の CreateIcon と SendMessageW（WM_SETICON）を呼ぶ（ADR 0002 の 4）"
)]
fn set_icon(window: HWND, which: u32, side: u32, pixels: &[u8]) {
    let Ok(width) = i32::try_from(side) else {
        return;
    };
    if width == 0_i32 {
        return;
    }
    // AND の面（1 画素 1 ビット・行は 2 バイトに揃える）は全部 0。32 ビットの画素はアルファで透ける。
    let stride = usize::try_from(side.div_ceil(16).saturating_mul(2)).unwrap_or(0);
    let mask = vec![0_u8; stride.saturating_mul(usize::try_from(side).unwrap_or(0))];
    // SAFETY: 2 つの面はこの関数の間生きている。大きさは辺 × 辺（XOR は 4 バイト・AND は上の行の幅）。
    let Ok(icon) =
        (unsafe { CreateIcon(None, width, width, 1, 32, mask.as_ptr(), pixels.as_ptr()) })
    else {
        return;
    };
    // SAFETY: 自分の窓にアイコンを付ける。アイコンは窓が生きている間使うので放さない（終われば OS が片付ける）。
    let _previous = unsafe {
        SendMessageW(
            window,
            WM_SETICON,
            Some(WPARAM(usize::try_from(which).unwrap_or(0))),
            Some(LPARAM(isize::try_from(icon.0.addr()).unwrap_or(0))),
        )
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 埋めた画素は辺 × 辺 × 4 バイトで、辺の小さい順。合う辺を選ぶ。
    #[test]
    fn images_have_their_sizes_and_the_nearest_larger_one_is_chosen() {
        for (side, pixels) in IMAGES {
            assert_eq!(pixels.len(), usize::try_from(side * side * 4).unwrap());
        }
        assert!(IMAGES.windows(2).all(|pair| match pair {
            [smaller, larger] => smaller.0 < larger.0,
            _ => false,
        }));
        assert_eq!(image_for(16).0, 16);
        assert_eq!(image_for(30).0, 32);
        assert_eq!(image_for(36).0, 40);
        assert_eq!(image_for(96).0, 64);
    }
}
