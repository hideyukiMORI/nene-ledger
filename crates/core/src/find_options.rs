//! 検索の条件（ADR 0030 の決定 3）。

/// 検索の条件。既定（`standard`）は、大文字小文字を区別しない・部分一致・全角半角を区別しない・ひらがなカタカナを
/// 区別する（Excel の日本語版に近い）。それぞれ切り替えられる。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FindOptions {
    bits: u8,
}

/// 大文字と小文字を区別する。
const MATCH_CASE: u8 = 0b0001;
/// セルの全体が一致するものだけ。
const WHOLE_CELL: u8 = 0b0010;
/// 全角と半角を区別する。
const MATCH_WIDTH: u8 = 0b0100;
/// ひらがなとカタカナを区別する。
const MATCH_KANA: u8 = 0b1000;

impl FindOptions {
    /// 既定の条件（ADR 0030 の決定 3）。
    #[must_use]
    pub const fn standard() -> Self {
        Self { bits: MATCH_KANA }
    }

    /// 大文字と小文字を区別するか。
    #[must_use]
    pub const fn match_case(self) -> bool {
        self.bits & MATCH_CASE != 0
    }

    /// セルの全体が一致するものだけか。
    #[must_use]
    pub const fn whole_cell(self) -> bool {
        self.bits & WHOLE_CELL != 0
    }

    /// 全角と半角を区別するか。
    #[must_use]
    pub const fn match_width(self) -> bool {
        self.bits & MATCH_WIDTH != 0
    }

    /// ひらがなとカタカナを区別するか。
    #[must_use]
    pub const fn match_kana(self) -> bool {
        self.bits & MATCH_KANA != 0
    }

    /// 大文字と小文字の区別を切り替えた条件（Alt+C）。
    #[must_use]
    pub const fn toggled_case(self) -> Self {
        Self {
            bits: self.bits ^ MATCH_CASE,
        }
    }

    /// 部分一致とセル全体を切り替えた条件（Alt+W）。
    #[must_use]
    pub const fn toggled_whole_cell(self) -> Self {
        Self {
            bits: self.bits ^ WHOLE_CELL,
        }
    }

    /// 全角と半角の区別を切り替えた条件。
    #[must_use]
    pub const fn toggled_width(self) -> Self {
        Self {
            bits: self.bits ^ MATCH_WIDTH,
        }
    }

    /// ひらがなとカタカナの区別を切り替えた条件。
    #[must_use]
    pub const fn toggled_kana(self) -> Self {
        Self {
            bits: self.bits ^ MATCH_KANA,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_standard_ignores_case_and_width_and_keeps_kana_and_toggles_flip_one_each() {
        let standard = FindOptions::standard();
        assert_eq!(
            (
                standard.match_case(),
                standard.whole_cell(),
                standard.match_width(),
                standard.match_kana()
            ),
            (false, false, false, true)
        );
        let toggled = standard
            .toggled_case()
            .toggled_whole_cell()
            .toggled_width()
            .toggled_kana();
        assert_eq!(
            (
                toggled.match_case(),
                toggled.whole_cell(),
                toggled.match_width(),
                toggled.match_kana()
            ),
            (true, true, true, false)
        );
        assert_eq!(toggled.toggled_case().toggled_case(), toggled);
    }
}
