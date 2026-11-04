//! 参照実装の切り出し（1 つのセル）。素朴に 1 文字ずつ見る（文字の幅は方言の文字の単位・ADR 0019）。
//!
//! 🔴 これが切り出しの規則の正本（ADR 0006）。速い経路（scan）は、同じ入力に対して
//! ここと同じ区間と境目を出すことをテストで固定する。

use crate::boundary::Boundary;
use crate::byte_range::ByteRange;
use crate::dialect::Dialect;
use crate::field::Field;
use crate::field_scan::FieldScan;
use crate::line_ending::LineEnding;
use crate::quote_escape::QuoteEscape;
use crate::quoting::Quoting;
use crate::row_shape::RowShape;
use crate::unclosed_quote::UnclosedQuote;

/// バックスラッシュ。
const BACKSLASH: u8 = b'\\';

/// `position` の単位が区切りか改行の始まりなら、その境目を返す（単位の値を ASCII と比べる・ADR 0019）。
fn boundary_at(input: &[u8], position: usize, dialect: Dialect) -> Option<Boundary> {
    let units = dialect.units();
    let unit = units.unit_at(input, position)?;
    if unit == u16::from(b'\n') {
        return Some(Boundary::Terminator(LineEnding::Lf));
    }
    if unit == u16::from(b'\r') {
        let crlf = units.is_ascii_at(input, position.saturating_add(units.unit_len()), b'\n');
        let ending = if crlf {
            LineEnding::CrLf
        } else {
            LineEnding::Cr
        };
        return Some(Boundary::Terminator(ending));
    }
    (unit == u16::from(dialect.delimiter().byte())).then_some(Boundary::Delimiter)
}

/// 引用の外で、区切りか改行か入力の終わりまで、文字ごとに進む。`escape` のバイトは次の 1 文字を逃がす。
fn unquoted_stop(
    input: &[u8],
    from: usize,
    dialect: Dialect,
    escape: Option<u8>,
) -> (usize, Boundary) {
    let units = dialect.units();
    let step = units.unit_len();
    let mut position = from;
    loop {
        let Some(unit) = units.unit_at(input, position) else {
            return (input.len(), Boundary::End);
        };
        if let Some(boundary) = boundary_at(input, position, dialect) {
            return (position, boundary);
        }
        let width = if escape.is_some_and(|escape| unit == u16::from(escape)) {
            units
                .char_width(input, position.saturating_add(step))
                .saturating_add(step)
        } else {
            units.char_width(input, position)
        };
        position = position.saturating_add(width);
    }
}

/// `""` の流儀で、閉じる引用符の位置を文字ごとに探す。閉じなければ `None`。
fn closing_doubled(input: &[u8], from: usize, dialect: Dialect) -> Option<usize> {
    let (quote, units) = (dialect.quote(), dialect.units());
    let step = units.unit_len();
    let mut position = from;
    loop {
        if units.unit_at(input, position)? == u16::from(quote) {
            if !units.is_ascii_at(input, position.saturating_add(step), quote) {
                return Some(position);
            }
            position = position.saturating_add(step.saturating_mul(2));
        } else {
            position = position.saturating_add(units.char_width(input, position));
        }
    }
}

/// `\"` の流儀で、閉じる引用符の位置を文字ごとに探す。バックスラッシュは次の 1 文字を逃がす。閉じなければ `None`。
fn closing_backslash(input: &[u8], from: usize, dialect: Dialect) -> Option<usize> {
    let (quote, units) = (dialect.quote(), dialect.units());
    let step = units.unit_len();
    let mut position = from;
    loop {
        let unit = units.unit_at(input, position)?;
        if unit == u16::from(quote) {
            return Some(position);
        }
        let width = if unit == u16::from(BACKSLASH) {
            units
                .char_width(input, position.saturating_add(step))
                .saturating_add(step)
        } else {
            units.char_width(input, position)
        };
        position = position.saturating_add(width);
    }
}

/// `opened`（開いた引用符の次）から、閉じる引用符の位置を流儀ごとに探す。閉じなければ `None`。
fn closing(input: &[u8], opened: usize, dialect: Dialect, escape: QuoteEscape) -> Option<usize> {
    match escape {
        QuoteEscape::Doubled => closing_doubled(input, opened, dialect),
        QuoteEscape::Backslash => closing_backslash(input, opened, dialect),
    }
}

/// 引用の外のセルを切る。
fn plain(input: &[u8], start: usize, dialect: Dialect, escape: Option<u8>) -> FieldScan {
    let (stop, boundary) = unquoted_stop(input, start, dialect, escape);
    FieldScan::new(
        Field::new(ByteRange::between(start, stop), false),
        stop,
        boundary,
        RowShape::Complete,
    )
}

/// 入力の終わりまで閉じなかった引用符で始まるセル。読み方は方言が決める（ADR 0018）。
fn unclosed(input: &[u8], start: usize, dialect: Dialect) -> FieldScan {
    match dialect.unclosed_quote() {
        UnclosedQuote::ToEnd => {
            let end = input.len();
            FieldScan::new(
                Field::new(ByteRange::between(start, end), true),
                end,
                Boundary::End,
                RowShape::UnclosedQuote,
            )
        }
        UnclosedQuote::Literal => {
            let scan = plain(input, start, dialect, None);
            FieldScan::new(
                scan.field(),
                scan.stop(),
                scan.boundary(),
                RowShape::LiteralQuote,
            )
        }
    }
}

/// 引用符で始まるならそこから閉じる引用符まで、そうでなければ引用の外として切る。
///
/// 閉じた引用符の後ろに区切りでない文字が続くとき（`"a"b`）は、寛容に区切りまで同じセルに含める。
fn quoted_or_plain(input: &[u8], start: usize, dialect: Dialect, escape: QuoteEscape) -> FieldScan {
    let quote_len = dialect.units().unit_len();
    if !dialect.units().is_ascii_at(input, start, dialect.quote()) {
        return plain(input, start, dialect, None);
    }
    let Some(close) = closing(input, start.saturating_add(quote_len), dialect, escape) else {
        return unclosed(input, start, dialect);
    };
    let (stop, boundary) = unquoted_stop(input, close.saturating_add(quote_len), dialect, None);
    FieldScan::new(
        Field::new(ByteRange::between(start, stop), true),
        stop,
        boundary,
        RowShape::Complete,
    )
}

/// 引用符で始まるセルで、閉じた引用符の後ろ（区切りの前）に文字が続いたか（`"a"b`）。
///
/// 切り出しはこの形を寛容に 1 つのセルとして読むので、行の形には出ない。判別は、方言が合っていない印として
/// 点を減らす（Issue #60）。閉じる位置は切り出しと同じ関数で探す。`field` は `dialect` で切ったセル。
#[must_use]
pub fn trails_closing_quote(input: &[u8], field: Field, dialect: &Dialect) -> bool {
    let escape = match dialect.quoting() {
        Quoting::Doubled => QuoteEscape::Doubled,
        Quoting::QuotedBackslash => QuoteEscape::Backslash,
        Quoting::Plain | Quoting::Backslash => return false,
    };
    if !field.quoted() {
        return false;
    }
    let quote_len = dialect.units().unit_len();
    let opened = field.raw().start().saturating_add(quote_len);
    closing(input, opened, *dialect, escape)
        .is_some_and(|close| close.saturating_add(quote_len) < field.raw().end())
}

/// `start` から 1 つのセルを切る。
#[must_use]
pub fn scan_field(input: &[u8], start: usize, dialect: &Dialect) -> FieldScan {
    let scan = match dialect.quoting() {
        Quoting::Plain => plain(input, start, *dialect, None),
        Quoting::Backslash => plain(input, start, *dialect, Some(BACKSLASH)),
        Quoting::Doubled => quoted_or_plain(input, start, *dialect, QuoteEscape::Doubled),
        Quoting::QuotedBackslash => quoted_or_plain(input, start, *dialect, QuoteEscape::Backslash),
    };
    scan.in_units(dialect.units().unit_len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::delimiter::Delimiter;
    use crate::records::Records;
    use alloc::vec::Vec;

    /// 各レコードの各セルが、閉じた引用符の後ろに文字を続けたか。
    fn trails(input: &[u8], dialect: Dialect) -> Vec<Vec<bool>> {
        Records::new(input, dialect)
            .map(|record| {
                record
                    .fields()
                    .iter()
                    .map(|&field| trails_closing_quote(input, field, &dialect))
                    .collect()
            })
            .collect()
    }

    fn quoted_backslash() -> Dialect {
        Dialect::new(Delimiter::COMMA, Quoting::QuotedBackslash, b'"').unwrap()
    }

    #[test]
    fn doubled_quotes_read_with_the_backslash_style_trail() {
        let input = br#"1,"a ""b"" c",x"#;
        assert_eq!(trails(input, Dialect::rfc4180()), [[false, false, false]]);
        assert_eq!(trails(input, quoted_backslash()), [[false, true, false]]);
    }

    #[test]
    fn backslash_escapes_read_with_the_doubled_style_trail() {
        let input = br#"1,"He said \"hi\" ok",x"#;
        assert_eq!(trails(input, quoted_backslash()), [[false, false, false]]);
        assert_eq!(trails(input, Dialect::rfc4180()), [[false, true, false]]);
    }

    #[test]
    fn unquoted_unclosed_and_unquoting_styles_never_trail() {
        assert_eq!(trails(b"a\"b,c\n", Dialect::rfc4180()), [[false, false]]);
        assert_eq!(trails(b"a,\"b\n", Dialect::rfc4180()), [[false, false]]);
        for quoting in [Quoting::Plain, Quoting::Backslash] {
            let dialect = Dialect::new(Delimiter::COMMA, quoting, b'"').unwrap();
            assert_eq!(trails(b"\"a\"b,c\n", dialect), [[false, false]]);
        }
    }

    #[test]
    fn utf16_cells_are_measured_in_two_byte_units() {
        let text: Vec<u8> = "\"a\"b,\"c\"\n"
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect();
        let dialect = Dialect::rfc4180().with_units(crate::text_units::TextUnits::Utf16Le);
        assert_eq!(trails(&text, dialect), [[true, false]]);
    }
}
