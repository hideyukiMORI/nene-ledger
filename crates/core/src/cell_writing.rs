//! 編集したセルを、保存で原本の方言どおりに書く規則（ADR 0024 の決定 4）。

use alloc::string::String;

use crate::dialect::Dialect;
use crate::line_ending::LineEnding;
use crate::quoting::Quoting;

/// 編集したセルを書けない理由。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellWriteFailure {
    /// 引用もエスケープもしない方言（`Quoting::Plain`）で、値が区切りか改行を含む。
    NeedsQuoting,
}

/// 値 `value` を、方言 `dialect` のセルとして書いた文字列（符号化の前）。
///
/// - 引用の方言（`Doubled` / `QuotedBackslash`）では、原本のセルが引用されていたら（`was_quoted`）引用のまま書く。
///   引用されていなければ、区切り・引用符・改行を含むときだけ引用する（施主の判断 1）
/// - セルの中の改行（`\r\n`・`\r`・`\n`）は、その行の終わり方 `line_break` に揃える（施主の判断 2）。
///   終わり方の無い最後の行では `\n`
/// - `Backslash` の方言では、`\`・タブ・改行・区切りをバックスラッシュで逃がす（読み方の `value` の逆）
///
/// # Errors
///
/// `Plain` の方言で、値が区切りか改行を含むとき [`CellWriteFailure::NeedsQuoting`]。
pub fn written_cell(
    value: &str,
    was_quoted: bool,
    dialect: &Dialect,
    line_break: LineEnding,
) -> Result<String, CellWriteFailure> {
    let delimiter = char::from(dialect.delimiter().byte());
    let quote = char::from(dialect.quote());
    let has_break = value.contains(['\r', '\n']);
    match dialect.quoting() {
        Quoting::Doubled | Quoting::QuotedBackslash
            if !(was_quoted || has_break || value.contains([delimiter, quote])) =>
        {
            Ok(String::from(value))
        }
        Quoting::Doubled => Ok(quoted(
            &broken(value, line_break),
            quote,
            |out, character| {
                if character == quote {
                    out.push(quote);
                }
            },
        )),
        Quoting::QuotedBackslash => Ok(quoted(
            &broken(value, line_break),
            quote,
            |out, character| {
                if character == quote || character == '\\' {
                    out.push('\\');
                }
            },
        )),
        Quoting::Backslash => Ok(backslashed(&broken(value, LineEnding::Lf), delimiter)),
        Quoting::Plain if has_break || value.contains(delimiter) => {
            Err(CellWriteFailure::NeedsQuoting)
        }
        Quoting::Plain => Ok(String::from(value)),
    }
}

/// 改行（`\r\n`・`\r`・`\n`）を `line_break` に揃える。終わり方の無い行では `\n`。
fn broken(value: &str, line_break: LineEnding) -> String {
    let unified = value.replace("\r\n", "\n").replace('\r', "\n");
    let wanted = match line_break {
        LineEnding::CrLf => "\r\n",
        LineEnding::Cr => "\r",
        LineEnding::Lf | LineEnding::EndOfInput => "\n",
    };
    if wanted == "\n" {
        unified
    } else {
        unified.replace('\n', wanted)
    }
}

/// `quote` で囲み、各字の前に `escape` が足すものを置く。
fn quoted<F: Fn(&mut String, char)>(value: &str, quote: char, escape: F) -> String {
    let mut out = String::with_capacity(value.len().saturating_add(2));
    out.push(quote);
    for character in value.chars() {
        escape(&mut out, character);
        out.push(character);
    }
    out.push(quote);
    out
}

/// `\`・タブ・改行・区切りをバックスラッシュで逃がす。
fn backslashed(value: &str, delimiter: char) -> String {
    let mut out = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\\' => out.push_str("\\\\"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            other if other == delimiter => {
                out.push('\\');
                out.push(other);
            }
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::delimiter::Delimiter;

    fn dialect(quoting: Quoting, delimiter: u8) -> Dialect {
        Dialect::new(Delimiter::new(delimiter).unwrap(), quoting, b'"').unwrap()
    }

    fn written(
        value: &str,
        was_quoted: bool,
        quoting: Quoting,
    ) -> Result<String, CellWriteFailure> {
        written_cell(value, was_quoted, &dialect(quoting, b','), LineEnding::CrLf)
    }

    #[test]
    fn doubled_quotes_only_when_needed_or_when_the_original_was_quoted() {
        assert_eq!(written("abc", false, Quoting::Doubled).unwrap(), "abc");
        assert_eq!(written("abc", true, Quoting::Doubled).unwrap(), "\"abc\"");
        assert_eq!(written("a,b", false, Quoting::Doubled).unwrap(), "\"a,b\"");
        assert_eq!(
            written("say \"hi\"", false, Quoting::Doubled).unwrap(),
            "\"say \"\"hi\"\"\""
        );
        assert_eq!(written("", true, Quoting::Doubled).unwrap(), "\"\"");
        assert_eq!(written("", false, Quoting::Doubled).unwrap(), "");
        assert_eq!(
            written(" 0012 ", false, Quoting::Doubled).unwrap(),
            " 0012 "
        );
    }

    #[test]
    fn line_breaks_follow_the_row() {
        assert_eq!(
            written("a\nb\r\nc\rd", false, Quoting::Doubled).unwrap(),
            "\"a\r\nb\r\nc\r\nd\""
        );
        let lf = written_cell(
            "a\r\nb",
            false,
            &dialect(Quoting::Doubled, b','),
            LineEnding::Lf,
        );
        assert_eq!(lf.unwrap(), "\"a\nb\"");
        let last = written_cell(
            "a\nb",
            false,
            &dialect(Quoting::Doubled, b','),
            LineEnding::EndOfInput,
        );
        assert_eq!(last.unwrap(), "\"a\nb\"");
    }

    #[test]
    fn quoted_backslash_escapes_quotes_and_backslashes() {
        assert_eq!(
            written("a\\b", false, Quoting::QuotedBackslash).unwrap(),
            "a\\b"
        );
        assert_eq!(
            written("a\"b\\", false, Quoting::QuotedBackslash).unwrap(),
            "\"a\\\"b\\\\\""
        );
    }

    #[test]
    fn backslash_and_plain_dialects() {
        let tsv = dialect(Quoting::Backslash, b'\t');
        let escaped = written_cell("a\tb\\c\r\nd", false, &tsv, LineEnding::CrLf);
        assert_eq!(escaped.unwrap(), "a\\tb\\\\c\\nd");
        let plain = dialect(Quoting::Plain, b'\t');
        assert_eq!(
            written_cell("a b", false, &plain, LineEnding::Lf).unwrap(),
            "a b"
        );
        assert_eq!(
            written_cell("a\tb", false, &plain, LineEnding::Lf),
            Err(CellWriteFailure::NeedsQuoting)
        );
        assert_eq!(
            written_cell("a\nb", false, &plain, LineEnding::Lf),
            Err(CellWriteFailure::NeedsQuoting)
        );
    }
}
