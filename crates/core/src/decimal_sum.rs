//! 10 進の数を足し合わせる（足す順に依らない合計・GitHub Issue #42）。

use crate::decimal::Decimal;

/// 小数点以下の最大の桁数（10^37 の 2 倍が i128 に収まる）。これより細かい数を足すと桁あふれとする。
const MAX_SCALE: u32 = 37;

/// 10 進の数の合計。整数部と小数部を別に足す（`|小数部| < 10^scale`）。
///
/// 1 つの [`Decimal`] に足していくと、大きい数の後に小数の細かい数が来たとき、大きい数を細かい桁にそろえて
/// i128 を越え、後で打ち消す数が来ても戻らない（`[1e21, 1e-18, -1e21]` が桁あふれで、順を変えると `1e-18`）。
/// 整数部と小数部を分けて持てば、途中の合計は桁をそろえずに済み、最後の合計が表せる限り順に依らない。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecimalSum {
    whole: i128,
    fraction: i128,
    scale: u32,
    overflowed: bool,
}

impl DecimalSum {
    /// 0。
    pub const ZERO: Self = Self {
        whole: 0,
        fraction: 0,
        scale: 0,
        overflowed: false,
    };

    /// `number` を足す。整数部が i128 を越えるか、小数点以下が [`MAX_SCALE`] 桁を越えれば、以後は桁あふれ。
    pub fn add(&mut self, number: Decimal) {
        if self.overflowed {
            return;
        }
        if self.added(number).is_none() {
            self.overflowed = true;
        }
    }

    fn added(&mut self, number: Decimal) -> Option<()> {
        let scale = number.scale();
        if scale > MAX_SCALE {
            return None;
        }
        let own = 10_i128.checked_pow(scale)?;
        let (whole, fraction) = (number.mantissa() / own, number.mantissa() % own);
        if scale > self.scale {
            self.fraction = self
                .fraction
                .checked_mul(10_i128.checked_pow(scale - self.scale)?)?;
            self.scale = scale;
        }
        let fraction = fraction.checked_mul(10_i128.checked_pow(self.scale - scale)?)?;
        let unit = 10_i128.checked_pow(self.scale)?;
        let sum = self.fraction.checked_add(fraction)?;
        self.whole = self.whole.checked_add(whole)?.checked_add(sum / unit)?;
        self.fraction = sum % unit;
        Some(())
    }

    /// 合計。小数点以下の桁は足した数のうち最も多い桁。桁あふれか、合計が 1 つの [`Decimal`] に表せなければ `None`。
    #[must_use]
    pub fn total(self) -> Option<Decimal> {
        if self.overflowed {
            return None;
        }
        let unit = 10_i128.checked_pow(self.scale)?;
        let mantissa = self.whole.checked_mul(unit)?.checked_add(self.fraction)?;
        Some(Decimal::new(mantissa, self.scale))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn number(mantissa: i128, scale: u32) -> Decimal {
        Decimal::new(mantissa, scale)
    }

    fn sum_of(numbers: &[Decimal]) -> Option<Decimal> {
        let mut sum = DecimalSum::ZERO;
        for &value in numbers {
            sum.add(value);
        }
        sum.total()
    }

    #[test]
    fn small_sums_are_exact_and_keep_the_finest_scale() {
        assert_eq!(sum_of(&[]), Some(number(0, 0)));
        assert_eq!(sum_of(&[number(1, 1), number(2, 1)]), Some(number(3, 1)));
        assert_eq!(
            sum_of(&[number(150, 2), number(2, 0)]),
            Some(number(350, 2))
        );
        assert_eq!(
            sum_of(&[number(-15, 1), number(5, 1)]),
            Some(number(-10, 1))
        );
        assert_eq!(
            sum_of(&[number(-25, 1), number(1, 0), number(-75, 2)]),
            Some(number(-225, 2))
        );
    }

    /// 足す順に依らない（GitHub Issue #42。前は 1 つ目の順が桁あふれ）。
    #[test]
    fn the_order_of_huge_and_tiny_numbers_does_not_matter() {
        let (huge, tiny, minus) = (
            number(10_i128.pow(21), 0),
            number(1, 18),
            number(-(10_i128.pow(21)), 0),
        );
        for order in [
            [huge, tiny, minus],
            [huge, minus, tiny],
            [tiny, huge, minus],
            [minus, tiny, huge],
        ] {
            assert_eq!(sum_of(&order), Some(number(1, 18)), "{order:?}");
        }
        // 最後の合計が表せなければ、どの順でも None。
        assert_eq!(sum_of(&[huge, tiny]), None);
        assert_eq!(sum_of(&[tiny, huge]), None);
    }

    #[test]
    fn overflow_is_kept() {
        let largest = number(i128::MAX, 0);
        assert_eq!(sum_of(&[largest, number(1, 0)]), None);
        assert_eq!(sum_of(&[largest, number(1, 0), number(-1, 0)]), None);
        assert_eq!(sum_of(&[number(1, 38)]), None);
    }
}
