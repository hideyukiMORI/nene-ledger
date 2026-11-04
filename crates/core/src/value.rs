//! セルの値（引用符とエスケープを外したバイト列）。

use alloc::vec::Vec;

use crate::dialect::Dialect;
use crate::field::Field;
use crate::quoting::Quoting;
use crate::text_units::{TextUnits, characters};

/// バックスラッシュ。
const BACKSLASH: u8 = b'\\';

/// `\X` の X を値のバイトに直す（`\t` `\n` `\r` 以外は X そのもの）。
fn escaped(byte: u8) -> u8 {
    match byte {
        b't' => b'\t',
        b'n' => b'\n',
        b'r' => b'\r',
        other => other,
    }
}

/// バックスラッシュのエスケープを外す。逃がすのは次の 1 文字。ASCII の文字は `\t` などを直して単位で書き、
/// それ以外の文字はそのままのバイトで書く。
fn unescape_backslash(raw: &[u8], units: TextUnits) -> Vec<u8> {
    let mut out = Vec::with_capacity(raw.len());
    let mut chars = characters(raw, units);
    while let Some(character) = chars.next() {
        if !units.is_ascii_char(character, BACKSLASH) {
            out.extend_from_slice(character);
            continue;
        }
        match chars.next() {
            Some(next) => match units.ascii_of(next) {
                Some(ascii) => units.push_ascii(&mut out, escaped(ascii)),
                None => out.extend_from_slice(next),
            },
            None => units.push_ascii(&mut out, BACKSLASH),
        }
    }
    out
}

/// 引用符で囲まれた `""` の流儀のセルから値を取り出す。閉じた引用符の後ろの文字は、そのまま続ける。
fn unquote_doubled(raw: &[u8], quote: u8, units: TextUnits) -> Vec<u8> {
    let mut out = Vec::with_capacity(raw.len());
    let body = raw.get(units.unit_len()..).unwrap_or_default();
    let mut chars = characters(body, units).peekable();
    while let Some(character) = chars.next() {
        if !units.is_ascii_char(character, quote) {
            out.extend_from_slice(character);
        } else if chars
            .next_if(|next| units.is_ascii_char(next, quote))
            .is_some()
        {
            units.push_ascii(&mut out, quote);
        } else {
            for rest in chars {
                out.extend_from_slice(rest);
            }
            break;
        }
    }
    out
}

/// 引用符で囲まれた `\"` の流儀のセルから値を取り出す。
fn unquote_backslash(raw: &[u8], quote: u8, units: TextUnits) -> Vec<u8> {
    let body = raw.get(units.unit_len()..).unwrap_or_default();
    let mut escaping = false;
    let mut position = 0_usize;
    let mut closing = None;
    for character in characters(body, units) {
        if !escaping && units.is_ascii_char(character, quote) {
            closing = Some(position);
            break;
        }
        escaping = !escaping && units.is_ascii_char(character, BACKSLASH);
        position = position.saturating_add(character.len());
    }
    let Some(close) = closing else {
        return unescape_backslash(body, units);
    };
    let (inside, rest) = body.split_at(close);
    let mut out = unescape_backslash(inside, units);
    out.extend(rest.iter().skip(units.unit_len()));
    out
}

/// セルの値を返す。セルは文字列のまま扱い、型は推し量らない（KEEP-01）。
#[must_use]
pub fn value(input: &[u8], field: Field, dialect: &Dialect) -> Vec<u8> {
    let raw = field.raw().slice(input).unwrap_or_default();
    let units = dialect.units();
    match (dialect.quoting(), field.quoted()) {
        (Quoting::Doubled, true) => unquote_doubled(raw, dialect.quote(), units),
        (Quoting::QuotedBackslash, true) => unquote_backslash(raw, dialect.quote(), units),
        (Quoting::Backslash, _) => unescape_backslash(raw, units),
        (Quoting::Plain | Quoting::Doubled | Quoting::QuotedBackslash, _) => raw.to_vec(),
    }
}
