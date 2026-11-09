//! 字の送り幅（UI が字体から測って値で渡す・GitHub Issue #462）。

use crate::display_width::char_columns;

/// 送り幅を数える細かさ（1 画素を 64 に分ける）。字ごとの端数を足してから画素に丸める。
pub const SUBPIXELS: u32 = 64;
/// 測る ASCII の字（空白から `~` まで）の数。
pub const ASCII_PRINTABLE: usize = 95;
/// 測る ASCII の最初の字。
const ASCII_FIRST: char = ' ';

/// 字の送り幅（`SUBPIXELS` 分の 1 画素）。ASCII の印字できる字は 1 字ずつ、全角は 1 つの値、それ以外の半角は
/// `0` の幅の桁の数の倍。core は字体を知らないので、UI が測って渡す（ADR 0002 の決定 2）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CharAdvances {
    ascii: [u16; ASCII_PRINTABLE],
    wide: u16,
}

impl CharAdvances {
    /// 測っていないとき：半角はどれも `half_width` 画素、全角はその 2 倍（等幅とみなす）。
    #[must_use]
    pub fn uniform(half_width: u32) -> Self {
        let half = subpixels(half_width.saturating_mul(SUBPIXELS));
        Self {
            ascii: [half; ASCII_PRINTABLE],
            wide: subpixels(half_width.saturating_mul(SUBPIXELS).saturating_mul(2)),
        }
    }

    /// UI が測った送り幅（`SUBPIXELS` 分の 1 画素）。`ascii` は空白から `~` までの順、`wide` は全角 1 字。
    #[must_use]
    pub fn measured(ascii: [u32; ASCII_PRINTABLE], wide: u32) -> Self {
        Self {
            ascii: ascii.map(subpixels),
            wide: subpixels(wide),
        }
    }

    /// `character` の送り幅（`SUBPIXELS` 分の 1 画素）。前の字に重なる字は 0。
    #[must_use]
    pub fn of(&self, character: char) -> u32 {
        if let Some(advance) = ascii_slot(character).and_then(|slot| self.ascii.get(slot)) {
            return u32::from(*advance);
        }
        match char_columns(character) {
            0 => 0,
            2 => u32::from(self.wide),
            columns => columns.saturating_mul(self.of('0')),
        }
    }

    /// `text` の幅（画素・端数は足してから丸める）。
    #[must_use]
    pub fn width(&self, text: &str) -> u32 {
        let total = text.chars().fold(0_u32, |sum, character| {
            sum.saturating_add(self.of(character))
        });
        total.saturating_add(SUBPIXELS / 2) / SUBPIXELS
    }
}

/// ASCII の印字できる字なら、表の番目。
fn ascii_slot(character: char) -> Option<usize> {
    let offset = u32::from(character).checked_sub(u32::from(ASCII_FIRST))?;
    usize::try_from(offset)
        .ok()
        .filter(|&slot| slot < ASCII_PRINTABLE)
}

fn subpixels(value: u32) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_advances_are_half_and_double() {
        let advances = CharAdvances::uniform(8);
        assert_eq!(advances.of('a'), 8 * SUBPIXELS);
        assert_eq!(advances.of('あ'), 16 * SUBPIXELS);
        assert_eq!(advances.of('\u{301}'), 0);
        assert_eq!(advances.width("aあ"), 24);
    }

    /// 測った幅は字ごとに使い、端数は足してから丸める（GitHub Issue #462）。
    #[test]
    fn measured_advances_add_fractions_before_rounding() {
        let mut ascii = [8 * SUBPIXELS; ASCII_PRINTABLE];
        ascii[usize::from(b'i' - b' ')] = 3 * SUBPIXELS + 16;
        let advances = CharAdvances::measured(ascii, 14 * SUBPIXELS + 19);
        assert_eq!(advances.of('i'), 3 * SUBPIXELS + 16);
        assert_eq!(advances.width("iiii"), 13);
        assert_eq!(advances.width("国国国国国国国国国国"), 143);
        // ASCII の外の半角（半角カナ）は `0` の幅。
        assert_eq!(advances.of('ｱ'), 8 * SUBPIXELS);
        assert_eq!(advances.width("~"), 8);
        assert_eq!(advances.width("\u{7f}"), 8);
    }
}
