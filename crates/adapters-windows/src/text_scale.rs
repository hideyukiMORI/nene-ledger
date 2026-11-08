//! OS の文字の大きさの観測（A11Y-03・GitHub Issue #263）。Windows の「テキストを大きくする」（`TextScaleFactor`）。

use windows::Win32::System::Registry::{HKEY_CURRENT_USER, RRF_RT_REG_DWORD, RegGetValueW};
use windows::core::w;

/// 設定の画面で選べる倍率の範囲（百分率）。
const RANGE: core::ops::RangeInclusive<u32> = 100..=225;

/// OS の文字の倍率（百分率）。読めなければ 100（変えない）。
#[must_use]
pub fn read() -> u32 {
    text_scale_factor().map_or(100, from_value)
}

/// `TextScaleFactor` の値から倍率へ（設定の画面で選べる 100〜225 に収める）。
#[must_use]
pub fn from_value(value: u32) -> u32 {
    value.clamp(*RANGE.start(), *RANGE.end())
}

/// レジストリの `HKCU\Software\Microsoft\Accessibility\TextScaleFactor`（無ければ `None`）。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の RegGetValueW を呼ぶ（ADR 0002 の 4）"
)]
fn text_scale_factor() -> Option<u32> {
    let mut value = 0_u32;
    let mut size = u32::try_from(size_of::<u32>()).ok()?;
    // SAFETY: 鍵と値の名前は静的な文字列。value と size はこの関数の局所で、size は value の大きさ。
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Accessibility"),
            w!("TextScaleFactor"),
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

    /// 設定の画面で選べる 100〜225 に収める。
    #[test]
    fn values_are_kept_in_the_settings_range() {
        assert_eq!(from_value(100), 100);
        assert_eq!(from_value(150), 150);
        assert_eq!(from_value(225), 225);
        assert_eq!(from_value(0), 100);
        assert_eq!(from_value(500), 225);
    }
}
