//! OS の明暗の観測（THEME-02）。Windows の「アプリのモード」（`AppsUseLightTheme`）。
//!
//! 高コントラスト（A11Y-02）は `SPI_GETHIGHCONTRAST` の `HCF_HIGHCONTRASTON` で、その明暗は窓の背景の色（`COLOR_WINDOW`）の明るさで決める。

use nene_ledger_core::appearance::Appearance;
use windows::Win32::Graphics::Gdi::{COLOR_WINDOW, GetSysColor};
use windows::Win32::System::Registry::{HKEY_CURRENT_USER, RRF_RT_REG_DWORD, RegGetValueW};
use windows::Win32::UI::Accessibility::{HCF_HIGHCONTRASTON, HIGHCONTRASTW, HIGHCONTRASTW_FLAGS};
use windows::Win32::UI::WindowsAndMessaging::{
    SPI_GETHIGHCONTRAST, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SystemParametersInfoW,
};
use windows::core::{PWSTR, w};

/// 明るい背景とみなす明るさの下限（0〜255 の加重平均）。
const LIGHT_LUMA: u32 = 128;

/// OS の明暗。読めなければ `None`（配色の希望の既定に任せる・ADR 0012）。
#[must_use]
pub fn read() -> Option<Appearance> {
    apps_use_light_theme().map(from_value)
}

/// `AppsUseLightTheme` の値から明暗へ（0 は暗い、それ以外は明るい）。
#[must_use]
pub fn from_value(value: u32) -> Appearance {
    if value == 0 {
        Appearance::Dark
    } else {
        Appearance::Light
    }
}

/// OS の高コントラストが有効なら、その明暗（窓の背景の色から）。無効か読めなければ `None`。
#[must_use]
pub fn high_contrast() -> Option<Appearance> {
    high_contrast_on().then(|| from_window_color(window_color()))
}

/// 窓の背景の色（`COLORREF` の `0x00BBGGRR`）の明るさから明暗へ。
#[must_use]
pub fn from_window_color(color: u32) -> Appearance {
    let [red, green, blue, _] = color.to_le_bytes();
    let luma = (u32::from(red) * 299 + u32::from(green) * 587 + u32::from(blue) * 114) / 1000;
    if luma >= LIGHT_LUMA {
        Appearance::Light
    } else {
        Appearance::Dark
    }
}

/// `SPI_GETHIGHCONTRAST` の `HCF_HIGHCONTRASTON`。読めなければ `false`。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の SystemParametersInfoW を呼ぶ（ADR 0002 の 4）"
)]
fn high_contrast_on() -> bool {
    let Ok(size) = u32::try_from(size_of::<HIGHCONTRASTW>()) else {
        return false;
    };
    let mut settings = HIGHCONTRASTW {
        cbSize: size,
        dwFlags: HIGHCONTRASTW_FLAGS(0),
        lpszDefaultScheme: PWSTR::null(),
    };
    // SAFETY: settings はこの関数の局所で、cbSize と uiParam はその大きさ。名前の文字列は OS の持ち物で、読まない。
    let status = unsafe {
        SystemParametersInfoW(
            SPI_GETHIGHCONTRAST,
            size,
            Some(core::ptr::from_mut(&mut settings).cast()),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
    };
    status.is_ok() && settings.dwFlags.contains(HCF_HIGHCONTRASTON)
}

/// 窓の背景の色（`COLOR_WINDOW`）。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の GetSysColor を呼ぶ（ADR 0002 の 4）"
)]
fn window_color() -> u32 {
    // SAFETY: 引数は定数の色の番号で、ポインタを渡さない。
    unsafe { GetSysColor(COLOR_WINDOW) }
}

/// レジストリの `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize\AppsUseLightTheme`。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の RegGetValueW を呼ぶ（ADR 0002 の 4）"
)]
fn apps_use_light_theme() -> Option<u32> {
    let mut value = 0_u32;
    let mut size = u32::try_from(size_of::<u32>()).ok()?;
    // SAFETY: 鍵と値の名前は静的な文字列。value と size はこの関数の局所で、size は value の大きさ。
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
            w!("AppsUseLightTheme"),
            RRF_RT_REG_DWORD,
            None,
            Some(core::ptr::from_mut(&mut value).cast()),
            Some(&raw mut size),
        )
    };
    status.is_ok().then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_means_dark() {
        assert_eq!(from_value(0), Appearance::Dark);
        assert_eq!(from_value(1), Appearance::Light);
    }

    #[test]
    fn the_window_color_decides_the_high_contrast_tone() {
        assert_eq!(from_window_color(0x0000_0000), Appearance::Dark);
        assert_eq!(from_window_color(0x00FF_FFFF), Appearance::Light);
        // 「砂漠」の淡い黄色（0x00E1FAFF）はライト、「夕暮れ」の濃い紺（0x002D1E1E）はダーク。
        assert_eq!(from_window_color(0x00E1_FAFF), Appearance::Light);
        assert_eq!(from_window_color(0x002D_1E1E), Appearance::Dark);
    }
}
