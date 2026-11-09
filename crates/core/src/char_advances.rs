//! 字の送り幅（UI が字体から測って渡す・GitHub Issue #462・#464）。

use crate::display_width::char_columns;

/// 送り幅を数える細かさ（1 画素を 64 に分ける）。字ごとの端数を足してから画素に丸める。
pub const SUBPIXELS: u32 = 64;

/// 1 字ずつ測る字の範囲。ASCII の印字できる字・かなと和文の約物（U+3000〜U+30FF）・全角と半角の形（U+FF00〜U+FFEF）。
/// 和文の UI 用の字体（Yu Gothic UI など）は、かなと約物を漢字より狭く描く（プロポーショナル）ので、漢字の幅 1 つでは
/// 足りない（GitHub Issue #464）。
const RANGES: [(char, char); 3] = [
    (' ', '~'),
    ('\u{3000}', '\u{30FF}'),
    ('\u{FF00}', '\u{FFEF}'),
];
/// 表の大きさ（`RANGES` の字の数の和）。
const SLOTS: usize = 95 + 256 + 240;

/// 字の送り幅（`SUBPIXELS` 分の 1 画素）。`RANGES` の字は 1 字ずつ、それ以外の全角（漢字など）は「国」の幅、それ以外の
/// 半角は `0` の幅の桁の数の倍。core は字体を知らないので、UI が測って渡す（ADR 0002 の決定 2）。
/// 大きいので、値で渡さず参照で渡す。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CharAdvances {
    table: [u16; SLOTS],
    wide: u16,
}

impl CharAdvances {
    /// 測っていないとき：半角はどれも `half_width` 画素、全角はその 2 倍（等幅とみなす）。
    #[must_use]
    pub fn uniform(half_width: u32) -> Self {
        let half = half_width.saturating_mul(SUBPIXELS);
        let mut advances = Self {
            table: [0; SLOTS],
            wide: subpixels(half.saturating_mul(2)),
        };
        for character in Self::measured_characters() {
            advances.set(character, char_columns(character).saturating_mul(half));
        }
        advances
    }

    /// UI が 1 字ずつ測る字（この順で [`Self::set`] に渡す）。これらの外の全角は [`Self::set_wide`] の 1 つの値。
    pub fn measured_characters() -> impl Iterator<Item = char> {
        RANGES.into_iter().flat_map(|(first, last)| first..=last)
    }

    /// `character` の送り幅を `advance`（`SUBPIXELS` 分の 1 画素）にする。1 字ずつ測る字でなければ何もしない。
    pub fn set(&mut self, character: char, advance: u32) {
        if let Some(cell) = slot(character).and_then(|index| self.table.get_mut(index)) {
            *cell = subpixels(advance);
        }
    }

    /// 1 字ずつ測らない全角（漢字など）の送り幅を `advance`（`SUBPIXELS` 分の 1 画素）にする。
    pub fn set_wide(&mut self, advance: u32) {
        self.wide = subpixels(advance);
    }

    /// `character` の送り幅（`SUBPIXELS` 分の 1 画素）。前の字に重なる字は 0。
    #[must_use]
    pub fn of(&self, character: char) -> u32 {
        if let Some(advance) = slot(character).and_then(|index| self.table.get(index)) {
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

/// 1 字ずつ測る字なら、表の番目。
fn slot(character: char) -> Option<usize> {
    let code = u32::from(character);
    let mut base = 0_usize;
    for (first, last) in RANGES {
        let (first, last) = (u32::from(first), u32::from(last));
        if (first..=last).contains(&code) {
            return usize::try_from(code.saturating_sub(first))
                .ok()
                .map(|offset| base.saturating_add(offset));
        }
        base = base.saturating_add(
            usize::try_from(last.saturating_sub(first))
                .ok()?
                .saturating_add(1),
        );
    }
    None
}

fn subpixels(value: u32) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ranges_fill_the_table_exactly() {
        assert_eq!(CharAdvances::measured_characters().count(), SLOTS);
        assert_eq!(slot('\u{FFEF}'), Some(SLOTS - 1));
        assert_eq!(slot('\u{3000}'), Some(95));
        assert_eq!(slot('国'), None);
    }

    #[test]
    fn uniform_advances_are_half_and_double() {
        let advances = CharAdvances::uniform(8);
        assert_eq!(advances.of('a'), 8 * SUBPIXELS);
        assert_eq!(advances.of('あ'), 16 * SUBPIXELS);
        assert_eq!(advances.of('国'), 16 * SUBPIXELS);
        assert_eq!(advances.of('ｱ'), 8 * SUBPIXELS);
        assert_eq!(advances.of('\u{301}'), 0);
        assert_eq!(advances.width("aあ"), 24);
    }

    /// 測った幅は字ごとに使い、端数は足してから丸める。かなと約物は漢字と別に測る（GitHub Issue #462・#464）。
    #[test]
    fn measured_advances_are_used_per_character() {
        let mut advances = CharAdvances::uniform(8);
        advances.set('i', 3 * SUBPIXELS + 16);
        advances.set('す', 11 * SUBPIXELS);
        advances.set('。', 7 * SUBPIXELS);
        advances.set_wide(14 * SUBPIXELS + 19);
        advances.set('国', 1);
        assert_eq!(advances.width("iiii"), 13);
        assert_eq!(advances.width("国国国国国国国国国国"), 143);
        assert_eq!(advances.width("です。"), 16 + 11 + 7);
    }
}
