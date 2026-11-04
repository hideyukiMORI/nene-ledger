//! 開き直すときの読み方の候補。

use alloc::format;
use alloc::string::String;

use nene_ledger_core::delimiter::Delimiter;
use nene_ledger_core::dialect::Dialect;
use nene_ledger_core::quoting::Quoting;
use nene_ledger_core::text_encoding::TextEncoding;
use nene_ledger_core::text_format::TextFormat;

use crate::reading_aspect::ReadingAspect;
use crate::status_line::{delimiter_name, encoding_name, quoting_short_name};

/// 開き直すときの読み方の候補（読み方の 1 つの面の値）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReadingChoice {
    /// 文字コード。
    Encoding(TextEncoding),
    /// 区切り（1 バイト）。
    Delimiter(u8),
    /// 引用符の扱い（流儀と引用符の字・Issue #68）。引用を使わない流儀の字は `"` に揃える。
    Quoting {
        /// 引用の流儀。
        quoting: Quoting,
        /// 引用符の字。
        quote: u8,
    },
    /// 見出しのレコードの番号（`None` は見出しなし）。
    Header(Option<u64>),
    /// 判別の候補の方言（区切り・引用の流儀・引用符の字を一度に当てる・Issue #3）。
    Dialect {
        /// 区切り（1 バイト）。
        delimiter: u8,
        /// 引用の流儀。
        quoting: Quoting,
        /// 引用符の字。引用を使わない流儀では `"` に揃える。
        quote: u8,
    },
}

impl ReadingChoice {
    /// 引用の流儀 `quoting` と引用符の字 `quote` の候補。引用を使わない流儀の字は `"` に揃える（一覧と比べられるように）。
    #[must_use]
    pub fn quoting(quoting: Quoting, quote: u8) -> Self {
        Self::Quoting {
            quoting,
            quote: aligned_quote(quoting, quote),
        }
    }

    /// 方言 `dialect` の区切り・引用の流儀・引用符の字を一度に当てる候補（判別の候補を選ぶ段・Issue #3）。
    #[must_use]
    pub fn dialect(dialect: Dialect) -> Self {
        Self::Dialect {
            delimiter: dialect.delimiter().byte(),
            quoting: dialect.quoting(),
            quote: aligned_quote(dialect.quoting(), dialect.quote()),
        }
    }

    /// 今の読み方 `format`・見出し `header` の、面 `aspect` の値。
    #[must_use]
    pub fn current(aspect: ReadingAspect, format: TextFormat, header: Option<u64>) -> Self {
        let dialect = format.dialect();
        match aspect {
            ReadingAspect::Encoding => Self::Encoding(format.encoding()),
            ReadingAspect::Delimiter => Self::Delimiter(dialect.delimiter().byte()),
            ReadingAspect::Quoting => Self::quoting(dialect.quoting(), dialect.quote()),
            ReadingAspect::Header => Self::Header(header),
        }
    }

    /// 表示名。
    #[must_use]
    pub fn name(self) -> String {
        match self {
            Self::Encoding(encoding) => String::from(encoding_name(encoding)),
            Self::Delimiter(byte) => delimiter_name(byte),
            Self::Quoting { quoting, quote } => quoting_name(quoting, quote),
            Self::Header(None) => String::from("見出しなし"),
            Self::Header(Some(record)) => format!("{} 行目が見出し", record.saturating_add(1)),
            // 候補の一覧は短く（状態の行と同じ名前）。点数と列の数が右に並ぶ。
            Self::Dialect {
                delimiter,
                quoting,
                quote,
            } => format!(
                "{}・{}",
                delimiter_name(delimiter),
                quoting_short_name(quoting, quote)
            ),
        }
    }

    /// 英字の検索語（日本語の入力が無い間の絞り込み・Issue #41 と同じ）。
    #[must_use]
    pub fn keyword(self) -> &'static str {
        match self {
            Self::Encoding(TextEncoding::Utf8 | TextEncoding::Utf8WithBom) => "utf8 utf-8",
            Self::Encoding(
                TextEncoding::Utf16Le
                | TextEncoding::Utf16Be
                | TextEncoding::Utf16LeWithBom
                | TextEncoding::Utf16BeWithBom,
            ) => "utf16 utf-16 unicode",
            Self::Encoding(TextEncoding::ShiftJis) => "sjis shift_jis cp932",
            Self::Encoding(TextEncoding::EucJp) => "euc-jp eucjp",
            Self::Encoding(TextEncoding::Windows1252) => "windows-1252 latin",
            Self::Delimiter(b',') => "comma",
            Self::Delimiter(b'\t') => "tab tsv",
            Self::Delimiter(b';') => "semicolon",
            Self::Delimiter(b'|') => "pipe bar",
            Self::Delimiter(b' ') => "space",
            Self::Delimiter(_) => "other",
            Self::Quoting { quote: b'\'', .. } => "quote single apostrophe",
            Self::Quoting { .. } => "quote",
            Self::Header(_) => "header",
            Self::Dialect { delimiter, .. } => Self::Delimiter(delimiter).keyword(),
        }
    }

    /// 今の読み方 `format`・見出し `header` に、この候補を当てた読み方と見出し。区切りと引用符が重なるなど、切れない組なら
    /// `None`（候補から外す）。
    #[must_use]
    pub fn apply(
        self,
        format: TextFormat,
        header: Option<u64>,
    ) -> Option<(TextFormat, Option<u64>)> {
        let dialect = format.dialect();
        let rebuilt = |delimiter: u8, quoting: Quoting, quote: u8| {
            Dialect::new(Delimiter::new(delimiter).ok()?, quoting, quote)
                .ok()
                .map(|built| built.with_unclosed_quote(dialect.unclosed_quote()))
        };
        match self {
            Self::Encoding(encoding) => Some((TextFormat::new(encoding, dialect), header)),
            Self::Delimiter(byte) => rebuilt(byte, dialect.quoting(), dialect.quote())
                .map(|built| (TextFormat::new(format.encoding(), built), header)),
            Self::Quoting { quoting, quote } => rebuilt(dialect.delimiter().byte(), quoting, quote)
                .map(|built| (TextFormat::new(format.encoding(), built), header)),
            Self::Header(chosen) => Some((format, chosen)),
            Self::Dialect {
                delimiter,
                quoting,
                quote,
            } => rebuilt(delimiter, quoting, quote)
                .map(|built| (TextFormat::new(format.encoding(), built), header)),
        }
    }
}

/// 引用を使わない流儀（`Backslash` / `Plain`）の引用符の字を `"` に揃える（一覧の候補と比べられるように）。
fn aligned_quote(quoting: Quoting, quote: u8) -> u8 {
    match quoting {
        Quoting::Doubled | Quoting::QuotedBackslash => quote,
        Quoting::Backslash | Quoting::Plain => b'"',
    }
}

/// 引用符の扱いの名前。
fn quoting_name(quoting: Quoting, quote: u8) -> String {
    match (quoting, quote) {
        (Quoting::Doubled, b'"') => String::from("引用符の中の \"\" で引用符（RFC 4180）"),
        (Quoting::Doubled, b'\'') => String::from("シングルクォート（'）で囲む"),
        (Quoting::Doubled, other) => format!("{0} で囲み、{0}{0} で {0} を書く", char::from(other)),
        (Quoting::QuotedBackslash, b'"') => String::from("引用符の中の \\ で逃がす"),
        (Quoting::QuotedBackslash, other) => {
            format!("{} で囲み、中は \\ で逃がす", char::from(other))
        }
        (Quoting::Backslash, _) => String::from("引用符なし・\\ で逃がす"),
        (Quoting::Plain, _) => String::from("引用もエスケープもしない"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utf8() -> TextFormat {
        TextFormat::new(TextEncoding::Utf8, Dialect::rfc4180())
    }

    #[test]
    fn every_aspect_lists_its_current_value() {
        for aspect in [
            ReadingAspect::Encoding,
            ReadingAspect::Delimiter,
            ReadingAspect::Quoting,
            ReadingAspect::Header,
        ] {
            let current = ReadingChoice::current(aspect, utf8(), Some(0));
            assert!(aspect.options().contains(&current), "{aspect:?}");
            assert!(!current.name().is_empty());
        }
        assert_eq!(ReadingAspect::Encoding.options().len(), 9);
        assert_eq!(ReadingAspect::Header.options().len(), 11);
    }

    #[test]
    fn a_choice_changes_only_its_aspect() {
        let (format, header) = ReadingChoice::Delimiter(b'\t')
            .apply(utf8(), Some(0))
            .unwrap();
        assert_eq!(format.dialect().delimiter().byte(), b'\t');
        assert_eq!((format.encoding(), header), (TextEncoding::Utf8, Some(0)));
        let (sjis, _) = ReadingChoice::Encoding(TextEncoding::ShiftJis)
            .apply(utf8(), None)
            .unwrap();
        assert_eq!(sjis.dialect().delimiter().byte(), b',');
        assert_eq!(
            ReadingChoice::Header(None).apply(utf8(), Some(0)),
            Some((utf8(), None))
        );
        assert_eq!(ReadingChoice::Header(Some(2)).name(), "3 行目が見出し");
        // 区切りが引用符と同じ組は切れない。
        assert_eq!(ReadingChoice::Delimiter(b'"').apply(utf8(), None), None);
    }

    #[test]
    fn the_quote_character_can_be_chosen_and_unused_quotes_are_aligned() {
        let single = ReadingChoice::quoting(Quoting::Doubled, b'\'');
        assert!(ReadingAspect::Quoting.options().contains(&single));
        let (format, _) = single.apply(utf8(), Some(0)).unwrap();
        assert_eq!(
            (format.dialect().quoting(), format.dialect().quote()),
            (Quoting::Doubled, b'\'')
        );
        assert_eq!(single.name(), "シングルクォート（'）で囲む");
        // 区切りを替えても引用符の字は残る。
        let (tab, _) = ReadingChoice::Delimiter(b'\t').apply(format, None).unwrap();
        assert_eq!(tab.dialect().quote(), b'\'');
        // 引用を使わない流儀は、判別が返した引用符の字に依らず一覧の候補と同じ（今の読み方の印が付く）。
        let plain = TextFormat::new(
            TextEncoding::Utf8,
            Dialect::new(Delimiter::TAB, Quoting::Plain, b'\'').unwrap(),
        );
        let current = ReadingChoice::current(ReadingAspect::Quoting, plain, None);
        assert_eq!(current, ReadingChoice::quoting(Quoting::Plain, b'"'));
        assert!(ReadingAspect::Quoting.options().contains(&current));
    }

    #[test]
    fn a_dialect_candidate_changes_the_delimiter_and_the_quoting_together() {
        let tab_plain = Dialect::new(Delimiter::TAB, Quoting::Plain, b'\'').unwrap();
        let choice = ReadingChoice::dialect(tab_plain);
        // 引用を使わない流儀の引用符の字は `"` に揃う（今の読み方と比べられる）。
        assert_eq!(
            choice,
            ReadingChoice::Dialect {
                delimiter: b'\t',
                quoting: Quoting::Plain,
                quote: b'"'
            }
        );
        assert_eq!(choice.name(), "タブ区切り・引用なし");
        assert_eq!(choice.keyword(), "tab tsv");
        let (format, header) = choice.apply(utf8(), Some(0)).unwrap();
        assert_eq!(format.dialect().delimiter().byte(), b'\t');
        assert_eq!(format.dialect().quoting(), Quoting::Plain);
        assert_eq!((format.encoding(), header), (TextEncoding::Utf8, Some(0)));
        assert_eq!(
            ReadingChoice::dialect(Dialect::rfc4180()).name(),
            "カンマ区切り・\"\" で引用符（RFC 4180）"
        );
        // 区切りと引用符が同じ組は切れない。
        let same = ReadingChoice::Dialect {
            delimiter: b'"',
            quoting: Quoting::Doubled,
            quote: b'"',
        };
        assert_eq!(same.apply(utf8(), None), None);
    }
}
