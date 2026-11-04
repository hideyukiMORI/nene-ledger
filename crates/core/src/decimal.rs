//! 10 進の固定小数点の数（合計を正確に出すため・浮動小数を使わない）。

use alloc::string::String;

/// 平均を出す小数点以下の桁数。
pub const AVERAGE_SCALE: u32 = 6;

/// 10 進の固定小数点の数。値は `mantissa / 10^scale`。`0.1 + 0.2` は正確に `0.3` になる。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Decimal {
    mantissa: i128,
    scale: u32,
}

impl Decimal {
    /// 0。
    pub const ZERO: Self = Self {
        mantissa: 0,
        scale: 0,
    };

    /// 仮数 `mantissa` と小数点以下の桁数 `scale` から作る。
    #[must_use]
    pub fn new(mantissa: i128, scale: u32) -> Self {
        Self { mantissa, scale }
    }

    /// 仮数。
    #[must_use]
    pub fn mantissa(self) -> i128 {
        self.mantissa
    }

    /// 小数点以下の桁数。
    #[must_use]
    pub fn scale(self) -> u32 {
        self.scale
    }

    /// 足し算。小数点以下の桁の多いほうにそろえる。桁あふれなら `None`。
    #[must_use]
    pub fn checked_add(self, other: Self) -> Option<Self> {
        let scale = self.scale.max(other.scale);
        let left = self.rescaled(scale)?;
        let right = other.rescaled(scale)?;
        Some(Self {
            mantissa: left.checked_add(right)?,
            scale,
        })
    }

    /// `count` 個で割った平均を、小数点以下 `AVERAGE_SCALE` 桁で四捨五入する（0.5 は 0 から遠い側へ）。
    /// `count` が 0 か桁あふれなら `None`。
    #[must_use]
    pub fn average(self, count: u64) -> Option<Self> {
        if count == 0 {
            return None;
        }
        let (numerator, divisor) = if self.scale <= AVERAGE_SCALE {
            (self.rescaled(AVERAGE_SCALE)?, i128::from(count))
        } else {
            let extra = 10_i128.checked_pow(self.scale.saturating_sub(AVERAGE_SCALE))?;
            (self.mantissa, i128::from(count).checked_mul(extra)?)
        };
        let doubled = numerator
            .checked_abs()?
            .checked_mul(2)?
            .checked_add(divisor)?;
        let rounded = doubled.checked_div(divisor.checked_mul(2)?)?;
        let mantissa = if numerator < 0 { -rounded } else { rounded };
        Some(Self {
            mantissa,
            scale: AVERAGE_SCALE,
        })
    }

    /// 小数点以下の末尾の 0 を落とした数（`1.500000` → `1.5`・`2.000000` → `2`）。
    #[must_use]
    pub fn trimmed(self) -> Self {
        let mut value = self;
        while value.scale > 0 && value.mantissa % 10 == 0 {
            value.mantissa /= 10;
            value.scale = value.scale.saturating_sub(1);
        }
        value
    }

    /// 符号・整数部・小数部の文字列（`-1234.50`）。区切りは付けない。
    #[must_use]
    pub fn to_plain(self) -> String {
        let digits = alloc::format!("{}", self.mantissa.unsigned_abs());
        let scale = usize::try_from(self.scale).unwrap_or(usize::MAX);
        let padded = if digits.len() <= scale {
            alloc::format!(
                "{}{digits}",
                "0".repeat(scale.saturating_sub(digits.len()).saturating_add(1))
            )
        } else {
            digits
        };
        let (whole, fraction) = padded.split_at(padded.len().saturating_sub(scale));
        let sign = if self.mantissa < 0 { "-" } else { "" };
        if fraction.is_empty() {
            alloc::format!("{sign}{whole}")
        } else {
            alloc::format!("{sign}{whole}.{fraction}")
        }
    }

    /// 小数点以下を `scale` 桁にした仮数。桁あふれなら `None`。
    fn rescaled(self, scale: u32) -> Option<i128> {
        let factor = 10_i128.checked_pow(scale.checked_sub(self.scale)?)?;
        self.mantissa.checked_mul(factor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn number(mantissa: i128, scale: u32) -> Decimal {
        Decimal::new(mantissa, scale)
    }

    #[test]
    fn addition_is_exact_in_decimal() {
        // 0.1 + 0.2 = 0.3（浮動小数なら 0.30000000000000004）。
        assert_eq!(number(1, 1).checked_add(number(2, 1)), Some(number(3, 1)));
        assert_eq!(
            number(150, 2).checked_add(number(2, 0)),
            Some(number(350, 2))
        );
        assert_eq!(
            Decimal::ZERO.checked_add(number(-5, 0)),
            Some(number(-5, 0))
        );
        assert_eq!(number(i128::MAX, 0).checked_add(number(1, 0)), None);
        assert_eq!(number(1, 0).checked_add(number(1, 40)), None);
    }

    #[test]
    fn averages_round_half_away_from_zero_at_six_places() {
        // 10 / 3 = 3.333333…、2 / 3 = 0.666667、-2 / 3 = -0.666667、1 / 8 = 0.125。
        assert_eq!(number(10, 0).average(3), Some(number(3_333_333, 6)));
        assert_eq!(number(2, 0).average(3), Some(number(666_667, 6)));
        assert_eq!(number(-2, 0).average(3), Some(number(-666_667, 6)));
        assert_eq!(
            number(1, 0).average(8).map(Decimal::trimmed),
            Some(number(125, 3))
        );
        // 小数点以下が 6 桁より多い値は 6 桁に丸める。
        assert_eq!(number(12_345_678, 7).average(1), Some(number(1_234_568, 6)));
        assert_eq!(number(1, 0).average(0), None);
    }

    #[test]
    fn numbers_print_plainly() {
        assert_eq!(number(-123_450, 2).to_plain(), "-1234.50");
        assert_eq!(number(5, 3).to_plain(), "0.005");
        assert_eq!(number(42, 0).to_plain(), "42");
        assert_eq!(number(2_000_000, 6).trimmed().to_plain(), "2");
        assert_eq!(number(0, 0).to_plain(), "0");
        assert_eq!(number(-5, 1).to_plain(), "-0.5");
    }
}
