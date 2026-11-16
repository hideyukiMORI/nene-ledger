//! 書き出しの形式（文字コード・区切り・改行・引用符の付け方・ADR 0036 の決定 1）。

use alloc::string::String;

use nene_ledger_core::delimiter::Delimiter;
use nene_ledger_core::dialect::Dialect;
use nene_ledger_core::line_ending::LineEnding;
use nene_ledger_core::quote_policy::QuotePolicy;
use nene_ledger_core::quoting::Quoting;
use nene_ledger_core::text_encoding::TextEncoding;
use nene_ledger_core::text_format::TextFormat;

use crate::export_aspect::ExportAspect;
use crate::export_shape::ExportShape;
use crate::status_line::encoding_name;

/// 選べる文字コード（段で替わる順）。
const ENCODINGS: [TextEncoding; 5] = [
    TextEncoding::Utf8,
    TextEncoding::Utf8WithBom,
    TextEncoding::ShiftJis,
    TextEncoding::EucJp,
    TextEncoding::Utf16LeWithBom,
];

/// 選べる区切り（段で替わる順）。
const DELIMITERS: [u8; 4] = *b",\t;|";

/// 選べる行の終わり方。
const LINE_BREAKS: [LineEnding; 2] = [LineEnding::CrLf, LineEnding::Lf];

/// 選べる引用符の付け方。
const POLICIES: [QuotePolicy; 2] = [QuotePolicy::AsNeeded, QuotePolicy::All];

/// 引用符（書き出しはいつも `"` を二重にして逃がす）。
const QUOTE: u8 = b'"';

/// 書き出しの形式。どの面も、選べる集合の中の値だけを持つ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExportFormat {
    encoding: TextEncoding,
    delimiter: u8,
    line_break: LineEnding,
    policy: QuotePolicy,
}

impl ExportFormat {
    /// 今の読み方 `format` と、今のファイルでいちばん多い行の終わり方 `line_break` から始める。選べる集合に無い値は、
    /// UTF-8・カンマ・CRLF にする。引用符は「要るときだけ」。
    #[must_use]
    pub fn starting(format: TextFormat, line_break: Option<LineEnding>) -> Self {
        let kept = |wanted: TextEncoding| ENCODINGS.contains(&wanted).then_some(wanted);
        let delimiter = format.dialect().delimiter().byte();
        Self {
            encoding: kept(format.encoding()).unwrap_or(TextEncoding::Utf8),
            delimiter: if DELIMITERS.contains(&delimiter) {
                delimiter
            } else {
                b','
            },
            line_break: match line_break {
                Some(LineEnding::Lf) => LineEnding::Lf,
                Some(LineEnding::CrLf | LineEnding::Cr | LineEnding::EndOfInput) | None => {
                    LineEnding::CrLf
                }
            },
            policy: QuotePolicy::AsNeeded,
        }
    }

    /// 面 `aspect` の値を、選べる集合の次（`forward` でなければ前）に替えた形式。端からは反対の端へ回る。
    #[must_use]
    pub fn turned(self, aspect: ExportAspect, forward: bool) -> Self {
        match aspect {
            ExportAspect::Encoding => Self {
                encoding: neighbour(&ENCODINGS, self.encoding, forward),
                ..self
            },
            ExportAspect::Delimiter => Self {
                delimiter: neighbour(&DELIMITERS, self.delimiter, forward),
                ..self
            },
            ExportAspect::LineBreak => Self {
                line_break: neighbour(&LINE_BREAKS, self.line_break, forward),
                ..self
            },
            ExportAspect::Quotes => Self {
                policy: neighbour(&POLICIES, self.policy, forward),
                ..self
            },
        }
    }

    /// 文字コード。
    #[must_use]
    pub const fn encoding(&self) -> TextEncoding {
        self.encoding
    }

    /// 書き出しの形（方言・引用符の付け方・行の終わり方）。
    #[must_use]
    pub fn shape(&self) -> ExportShape {
        let dialect = Delimiter::new(self.delimiter)
            .and_then(|delimiter| Dialect::new(delimiter, Quoting::Doubled, QUOTE))
            .unwrap_or_else(|_failure| Dialect::rfc4180());
        ExportShape::new(dialect, self.policy, self.line_break)
    }

    /// 書き出す先のファイルの拡張子（タブ区切りなら `tsv`、ほかは `csv`）。
    #[must_use]
    pub const fn extension(&self) -> &'static str {
        if self.delimiter == b'\t' {
            "tsv"
        } else {
            "csv"
        }
    }

    /// 面 `aspect` の今の値の名前。
    #[must_use]
    pub fn value_name(&self, aspect: ExportAspect) -> &'static str {
        match aspect {
            ExportAspect::Encoding => encoding_name(self.encoding),
            ExportAspect::Delimiter => match self.delimiter {
                b'\t' => "タブ",
                b';' => "セミコロン",
                b'|' => "縦棒",
                _other => "カンマ",
            },
            ExportAspect::LineBreak => match self.line_break {
                LineEnding::Lf => "LF",
                LineEnding::CrLf | LineEnding::Cr | LineEnding::EndOfInput => "CRLF",
            },
            ExportAspect::Quotes => match self.policy {
                QuotePolicy::AsNeeded => "要るときだけ",
                QuotePolicy::All => "すべてのセル",
            },
        }
    }

    /// 形式の全部の面を並べた文言（「UTF-8 · タブ · LF · 引用符 すべてのセル」）。
    #[must_use]
    pub fn describe(&self) -> String {
        alloc::format!(
            "{} · {} · {} · 引用符 {}",
            self.value_name(ExportAspect::Encoding),
            self.value_name(ExportAspect::Delimiter),
            self.value_name(ExportAspect::LineBreak),
            self.value_name(ExportAspect::Quotes)
        )
    }
}

/// 集合 `set` の中の `current` の次（`forward` でなければ前）の値。端からは反対の端へ回る。集合に無ければ最初の値。
fn neighbour<T: Copy + PartialEq>(set: &[T], current: T, forward: bool) -> T {
    let count = set.len().max(1);
    let at = set.iter().position(|&value| value == current).unwrap_or(0);
    let next = if forward {
        at.saturating_add(1) % count
    } else {
        at.saturating_add(count).saturating_sub(1) % count
    };
    set.get(next).copied().unwrap_or(current)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn format_of(encoding: TextEncoding, delimiter: u8) -> TextFormat {
        let dialect =
            Dialect::new(Delimiter::new(delimiter).unwrap(), Quoting::Doubled, b'"').unwrap();
        TextFormat::new(encoding, dialect)
    }

    #[test]
    fn it_starts_from_the_current_format_and_falls_back_to_the_common_one() {
        let kept = ExportFormat::starting(
            format_of(TextEncoding::ShiftJis, b'\t'),
            Some(LineEnding::Lf),
        );
        assert_eq!(
            kept.describe(),
            "Shift_JIS（CP932） · タブ · LF · 引用符 要るときだけ"
        );
        assert_eq!(kept.extension(), "tsv");
        let other = ExportFormat::starting(format_of(TextEncoding::Windows1252, b'#'), None);
        assert_eq!(
            other.describe(),
            "UTF-8 · カンマ · CRLF · 引用符 要るときだけ"
        );
        assert_eq!(other.extension(), "csv");
        assert_eq!(other.shape().line_break(), "\r\n");
    }

    #[test]
    fn each_aspect_turns_through_its_own_values_and_wraps() {
        let start = ExportFormat::starting(format_of(TextEncoding::Utf8, b','), None);
        let mut turned = start;
        let mut seen = alloc::vec::Vec::new();
        for _ in 0..ENCODINGS.len() {
            turned = turned.turned(ExportAspect::Encoding, true);
            seen.push(turned.encoding());
        }
        assert_eq!(seen.last(), Some(&TextEncoding::Utf8));
        assert_eq!(seen.len(), 5);
        assert_eq!(
            start.turned(ExportAspect::Encoding, false).encoding(),
            TextEncoding::Utf16LeWithBom
        );
        let all = start
            .turned(ExportAspect::Delimiter, false)
            .turned(ExportAspect::LineBreak, true)
            .turned(ExportAspect::Quotes, true);
        assert_eq!(all.describe(), "UTF-8 · 縦棒 · LF · 引用符 すべてのセル");
        assert_eq!(all.shape().policy(), QuotePolicy::All);
        assert_eq!(all.shape().dialect().delimiter().byte(), b'|');
    }
}
