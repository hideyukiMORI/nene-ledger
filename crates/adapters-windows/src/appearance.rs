//! OS の明暗の観測（THEME-02）。Windows の「アプリのモード」（`AppsUseLightTheme`）。

use nene_ledger_core::appearance::Appearance;
use windows::Win32::System::Registry::{HKEY_CURRENT_USER, RRF_RT_REG_DWORD, RegGetValueW};
use windows::core::w;

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
}
