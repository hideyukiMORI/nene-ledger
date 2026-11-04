//! 判別のためだけに見るセルの形。

/// 方言の判別で見るセルの形。**値は変えない**（KEEP-01）。試しに切った結果がもっともらしいかを測るためだけに使う。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CellKind {
    /// 空。
    Empty,
    /// 数に見える（符号・数字・`.` `,`・指数）。
    Number,
    /// 日付や時刻に見える（数字と `-` `/` `:` `.` `T` と空白）。
    DateLike,
    /// 区切りや引用符になりうる文字を含まない文字列。
    Word,
    /// 区切りや引用符になりうる文字か制御文字を含む（切り方を誤った兆し）。
    Other,
}

/// 区切りや引用符になりうる記号。
const SUSPICIOUS: [u8; 7] = *b",;\t|\"'^";

/// 数字か。
fn digit(byte: u8) -> bool {
    byte.is_ascii_digit()
}

/// 数に見えるか。
fn looks_like_number(value: &[u8]) -> bool {
    let body = value
        .strip_prefix(b"-")
        .or_else(|| value.strip_prefix(b"+"))
        .unwrap_or(value);
    let (mantissa, exponent) = match body.iter().position(|&b| b == b'e' || b == b'E') {
        Some(at) => (
            body.get(..at).unwrap_or_default(),
            body.get(at.saturating_add(1)..),
        ),
        None => (body, None),
    };
    let mantissa_ok = mantissa.first().copied().is_some_and(digit)
        && mantissa.iter().all(|&b| digit(b) || b == b'.' || b == b',');
    let exponent_ok = exponent.is_none_or(|e| {
        let digits = e
            .strip_prefix(b"-")
            .or_else(|| e.strip_prefix(b"+"))
            .unwrap_or(e);
        !digits.is_empty() && digits.iter().all(|&b| digit(b))
    });
    mantissa_ok && exponent_ok
}

/// 日付や時刻に見えるか（区切りの記号を 1 つ以上含む数字の並び）。
fn looks_like_date(value: &[u8]) -> bool {
    let allowed = |b: u8| digit(b) || matches!(b, b'-' | b'/' | b':' | b'.' | b'T' | b' ');
    let separators = value
        .iter()
        .filter(|&&b| matches!(b, b'-' | b'/' | b':'))
        .count();
    value.first().copied().is_some_and(digit)
        && separators >= 1
        && value.iter().all(|&b| allowed(b))
}

impl CellKind {
    /// セルの値の形を見る。
    #[must_use]
    pub fn of(value: &[u8]) -> Self {
        if value.is_empty() {
            return Self::Empty;
        }
        if looks_like_number(value) {
            return Self::Number;
        }
        if looks_like_date(value) {
            return Self::DateLike;
        }
        let suspicious = value
            .iter()
            .any(|&b| SUSPICIOUS.contains(&b) || (b.is_ascii_control() && b != b' '));
        if suspicious { Self::Other } else { Self::Word }
    }

    /// 切り方がもっともらしいと言える形か（`Other` 以外）。
    #[must_use]
    pub fn is_plausible(self) -> bool {
        self != Self::Other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_cells() {
        assert_eq!(CellKind::of(b""), CellKind::Empty);
        for number in [&b"0"[..], b"-12", b"+3.5", b"1,234", b"1.20E+03", b"7e5"] {
            assert_eq!(CellKind::of(number), CellKind::Number, "{number:?}");
        }
        for date in [
            &b"2026/1/2"[..],
            b"2026-10-22",
            b"12:30",
            b"2026-10-22T09:00",
        ] {
            assert_eq!(CellKind::of(date), CellKind::DateLike, "{date:?}");
        }
        for word in [&b"abc"[..], "東京都".as_bytes(), b"a-b_c", b"x y"] {
            assert_eq!(CellKind::of(word), CellKind::Word, "{word:?}");
        }
        for other in [&b"a,b"[..], b"a;b", b"\"x", b"a\tb", b"a\x01"] {
            assert_eq!(CellKind::of(other), CellKind::Other, "{other:?}");
        }
    }

    #[test]
    fn rejects_malformed_numbers() {
        for value in [&b"-"[..], b"1e", b"1e+", b".5x", b"e5"] {
            assert_ne!(CellKind::of(value), CellKind::Number, "{value:?}");
        }
        assert!(CellKind::Word.is_plausible());
        assert!(!CellKind::Other.is_plausible());
    }
}
