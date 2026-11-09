//! セルの値を数として読む規則（正本・Issue #42・GitHub Issue #41）。
//!
//! 選んだ範囲の集計（件数・合計・平均）だけが使う読み方で、セルの値は変えない（KEEP-01・FR-003）。

use alloc::string::String;

use crate::decimal::Decimal;

/// 仮数の桁数の上限（`i128` に収める）。
const MAX_DIGITS: usize = 38;

/// 指数の絶対値の上限（仮数の桁数の上限と同じ。超えれば数として読まない）。
const MAX_EXPONENT: u32 = 38;

/// セルの値 `text` を数として読む。
///
/// 日本の業務の表で数として書かれる形を広く読む（GitHub Issue #41。Excel はこれらの多くを数として読む）:
/// - 前後の空白（ASCII の空白と全角の空白）
/// - 全角の数字・`＋`・`－`・`．`・`，`（半角とみなす）
/// - 負の印: 頭の `-`・`−`（U+2212）・`▲`・`△`、括弧で包む `(100)`・`（100）`。正の印 `+` は頭に 1 つまで
/// - 通貨と単位: 頭の `¥`・`￥`・`$`、終わりの `円`（印の前でも後でもよい: `-¥1,000`・`¥-1,000`・`▲1,000円`）
/// - 桁区切りのカンマ: 3 桁ずつのときだけ（`1,234,567`。`12,34` は数でない）
/// - 指数表記: `1.2E+03`・`5e-2`
///
/// 次は数でない: `%` のつく値（`0.5` か `50` かで合計の意味が変わる）・ほかの通貨や単位・小数点が 2 つ・38 桁を超える仮数。
#[must_use]
pub fn read_number(text: &str) -> Option<Decimal> {
    let half = half_width(text);
    let trimmed = half.trim_matches(|character: char| {
        character.is_ascii_whitespace() || character == IDEOGRAPHIC_SPACE
    });
    let (parenthesized, inner) = match trimmed
        .strip_prefix('(')
        .and_then(|rest| rest.strip_suffix(')'))
    {
        Some(inner) => (true, inner),
        None => (false, trimmed),
    };
    let (signed, body) = signed_amount(inner)?;
    let number = plain_number(body)?;
    if parenthesized && signed {
        // `(-100)` のように印を重ねた形は読まない。
        return None;
    }
    let negative = parenthesized || signed_negative(inner);
    Some(if negative { negate(number) } else { number })
}

/// 値 `text` が、書き換えずにそのまま数と読める素の形か（`-12`・`0012`・`3.5`：半角の数字と、頭の `-` 1 つと、小数点
/// 1 つだけ）。[`read_number`] が数と読む値のうちこれでないもの（`¥3`・`1,200`・`1.5e3`・`▲100`・全角）は、書き方を
/// 読み替えて数にした値（集計がそう添える・GitHub Issue #319）。
#[must_use]
pub fn is_plain_number(text: &str) -> bool {
    let digits = text.strip_prefix('-').unwrap_or(text);
    let (whole, fraction) = digits.split_once('.').unwrap_or((digits, "0"));
    !whole.is_empty()
        && !fraction.is_empty()
        && whole.bytes().all(|byte| byte.is_ascii_digit())
        && fraction.bytes().all(|byte| byte.is_ascii_digit())
}

/// 全角の空白。
const IDEOGRAPHIC_SPACE: char = '\u{3000}';

/// 全角の数字・符号・小数点・桁区切り・括弧と、マイナス記号（U+2212）を半角にした写し。
fn half_width(text: &str) -> String {
    text.chars()
        .map(|character| match character {
            '\u{FF10}'..='\u{FF19}'
            | '\u{FF08}'
            | '\u{FF09}'
            | '\u{FF0B}'
            | '\u{FF0C}'
            | '\u{FF0D}'
            | '\u{FF0E}'
            | '\u{FF25}'
            | '\u{FF45}' => char::from_u32(u32::from(character) - 0xFEE0).unwrap_or(character),
            '\u{2212}' => '-',
            other => other,
        })
        .collect()
}

/// 頭の通貨記号と印（どちらが先でもよい）・終わりの `円` を外した数の部分と、印があったか。
/// 印が 2 つある・通貨記号が 2 つある形は読まない。
fn signed_amount(text: &str) -> Option<(bool, &str)> {
    let mut rest = text.strip_suffix('円').unwrap_or(text);
    let mut signed = false;
    let mut currency = false;
    loop {
        if let Some(after) = strip_sign(rest) {
            if signed {
                return None;
            }
            signed = true;
            rest = after;
        } else if let Some(after) = strip_currency(rest) {
            if currency {
                return None;
            }
            currency = true;
            rest = after;
        } else {
            return Some((signed, rest));
        }
    }
}

/// 頭の印（`+`・`-`・`▲`・`△`）を 1 つ外す。
fn strip_sign(text: &str) -> Option<&str> {
    ['+', '-', '▲', '△']
        .into_iter()
        .find_map(|sign| text.strip_prefix(sign))
}

/// 頭の通貨記号（`¥`・`￥`・`$`）を 1 つ外す。
fn strip_currency(text: &str) -> Option<&str> {
    ['¥', '￥', '$']
        .into_iter()
        .find_map(|sign| text.strip_prefix(sign))
}

/// 印が負（`-`・`▲`・`△`）か。通貨記号の後ろの印も見る。
fn signed_negative(text: &str) -> bool {
    let rest = strip_currency(text).unwrap_or(text);
    ['-', '▲', '△']
        .iter()
        .any(|&sign| text.starts_with(sign) || rest.starts_with(sign))
}

/// 符号を反転した数。
fn negate(number: Decimal) -> Decimal {
    Decimal::new(number.mantissa().saturating_neg(), number.scale())
}

/// 印と通貨記号を外した数の部分（数字・桁区切り・小数点・指数）を読む。
fn plain_number(text: &str) -> Option<Decimal> {
    let (significand, exponent) = match text.find(['e', 'E']) {
        Some(at) => (text.get(..at)?, Some(text.get(at.checked_add(1)?..)?)),
        None => (text, None),
    };
    let (whole, fraction) = significand.split_once('.').unwrap_or((significand, ""));
    let whole = ungrouped(whole)?;
    let digits_only = |part: &str| part.bytes().all(|byte| byte.is_ascii_digit());
    if (whole.is_empty() && fraction.is_empty())
        || !digits_only(&whole)
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
    match exponent {
        Some(exponent) => scaled(mantissa, scale, exponent),
        None => Some(Decimal::new(mantissa, scale)),
    }
}

/// 整数の部分から桁区切りのカンマを外す。カンマがあれば、頭が 1〜3 桁・後ろが 3 桁ずつのときだけ読む。
fn ungrouped(whole: &str) -> Option<String> {
    if !whole.contains(',') {
        return Some(String::from(whole));
    }
    let mut groups = whole.split(',');
    let head = groups.next()?;
    let head_fits = (1..=3).contains(&head.len());
    let rest_fit = groups.clone().all(|group| group.len() == 3);
    (head_fits && rest_fit).then(|| whole.replace(',', ""))
}

/// 仮数 `mantissa`・小数の桁 `scale` に、指数 `exponent`（`+3`・`-2`・`3`）を掛けた数。
fn scaled(mantissa: i128, scale: u32, exponent: &str) -> Option<Decimal> {
    let (negative, digits) = match exponent.as_bytes().first() {
        Some(b'-') => (true, exponent.get(1..)?),
        Some(b'+') => (false, exponent.get(1..)?),
        _ => (false, exponent),
    };
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let power: u32 = digits.parse().ok()?;
    if power > MAX_EXPONENT {
        return None;
    }
    if negative {
        let scale = scale.checked_add(power)?;
        (scale <= MAX_EXPONENT).then(|| Decimal::new(mantissa, scale))
    } else if power <= scale {
        Some(Decimal::new(mantissa, scale - power))
    } else {
        let mantissa = mantissa.checked_mul(10_i128.checked_pow(power - scale)?)?;
        Some(Decimal::new(mantissa, 0))
    }
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

    /// 日本の業務の表の数の書き方（GitHub Issue #41）。
    #[test]
    fn japanese_business_number_forms_are_read() {
        let cases: [(&str, i128, u32); 21] = [
            ("1,234", 1234, 0),
            ("1,234,567.89", 123_456_789, 2),
            ("１２３", 123, 0),
            ("１，２３４．５", 12345, 1),
            ("▲100", -100, 0),
            ("△1,500", -1500, 0),
            ("(100)", -100, 0),
            ("（２００）", -200, 0),
            ("−100", -100, 0),
            ("－100", -100, 0),
            ("¥1,000", 1000, 0),
            ("￥1000", 1000, 0),
            ("-¥1,000", -1000, 0),
            ("¥-1,000", -1000, 0),
            ("▲1,000円", -1000, 0),
            ("1000円", 1000, 0),
            ("$5.50", 550, 2),
            (" 12 ", 12, 0),
            ("\u{3000}12", 12, 0),
            ("1.2E+03", 1200, 0),
            ("5e-2", 5, 2),
        ];
        for (text, mantissa, scale) in cases {
            let read = read_number(text).unwrap_or_else(|| panic!("{text:?} は数"));
            assert_eq!(
                read.to_plain(),
                Decimal::new(mantissa, scale).to_plain(),
                "{text:?}"
            );
        }
    }

    #[test]
    fn anything_else_is_not_a_number() {
        for text in [
            "", " ", "\u{3000}", "12,34", "1,2345", ",123", "1,,234", "50%", "€5", "¥¥5", "--1",
            "▲-1", "(-100)", "(100", "+", "-", ".", "12a", "0x10", "1.2.3", "1e", "1e+", "1e40",
            "e5", "1.2E3.5", "9_999",
        ] {
            assert_eq!(read_number(text), None, "{text:?}");
        }
        // 38 桁を超える数は読まない。
        assert_eq!(read_number(&"9".repeat(39)), None);
        assert!(read_number(&"9".repeat(38)).is_some());
    }

    /// 素の数の形と、読み替えて数にする形を分ける（GitHub Issue #319）。
    #[test]
    fn plain_numbers_are_told_apart_from_rewritten_ones() {
        for plain in ["0", "-12", "0012", "3.5", "-0.25"] {
            assert!(is_plain_number(plain), "{plain}");
            assert!(read_number(plain).is_some());
        }
        for rewritten in [
            "¥3", "1,200", "1.5e3", "▲100", "(100)", "１２", " 12", "+5", "12円",
        ] {
            assert!(!is_plain_number(rewritten), "{rewritten}");
            assert!(read_number(rewritten).is_some(), "{rewritten}");
        }
        for neither in ["", "-", ".5", "5.", "abc", "1.2.3"] {
            assert!(!is_plain_number(neither), "{neither}");
        }
    }
}
