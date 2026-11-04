//! セルの値を数として読む規則（正本・Issue #42）。

use crate::decimal::Decimal;

/// 仮数の桁数の上限（`i128` に収める）。
const MAX_DIGITS: usize = 38;

/// セルの値 `text` を数として読む。半角の `+` / `-` を頭に 1 つまで・半角の数字・半角の小数点を 1 つまで（数字の間か、
/// 前後のどちらかに数字があること）。前後の空白・桁区切りのカンマ・通貨記号・全角の数字・指数表記は数ではない（型を推し量らない・
/// KEEP-01。読む規則を利用者が選べるようにするのは変換の規則の Issue）。
#[must_use]
pub fn read_number(text: &str) -> Option<Decimal> {
    let (negative, body) = match text.as_bytes().first() {
        Some(b'-') => (true, text.get(1..)?),
        Some(b'+') => (false, text.get(1..)?),
        _ => (false, text),
    };
    let (whole, fraction) = body.split_once('.').unwrap_or((body, ""));
    let digits_only = |part: &str| part.bytes().all(|byte| byte.is_ascii_digit());
    if whole.is_empty() && fraction.is_empty() {
        return None;
    }
    if !digits_only(whole)
        || !digits_only(fraction)
        || whole.len().saturating_add(fraction.len()) > MAX_DIGITS
    {
        return None;
    }
    let mut mantissa = 0_i128;
    for byte in whole.bytes().chain(fraction.bytes()) {
        mantissa = mantissa
            .checked_mul(10)?
            .checked_add(i128::from(byte - b'0'))?;
    }
    let scale = u32::try_from(fraction.len()).ok()?;
    Some(Decimal::new(
        if negative { -mantissa } else { mantissa },
        scale,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_half_width_numbers_are_read() {
        assert_eq!(read_number("1234"), Some(Decimal::new(1234, 0)));
        assert_eq!(read_number("-5.5"), Some(Decimal::new(-55, 1)));
        assert_eq!(read_number("+0.25"), Some(Decimal::new(25, 2)));
        assert_eq!(read_number("007"), Some(Decimal::new(7, 0)));
        assert_eq!(read_number(".5"), Some(Decimal::new(5, 1)));
        assert_eq!(read_number("3."), Some(Decimal::new(3, 0)));
    }

    #[test]
    fn anything_else_is_not_a_number() {
        for text in [
            "",
            " 12",
            "12 ",
            "1,234",
            "¥500",
            "$5",
            "１２３",
            "1e3",
            "1.2.3",
            "--1",
            "+",
            "-",
            ".",
            "12a",
            "0x10",
            "1.20E+03",
        ] {
            assert_eq!(read_number(text), None, "{text:?}");
        }
        // 38 桁を超える数は読まない。
        assert_eq!(read_number(&"9".repeat(39)), None);
        assert!(read_number(&"9".repeat(38)).is_some());
    }
}
