//! 開き直すときに選ぶ、読み方の 1 つの面。

use alloc::vec::Vec;

use nene_ledger_core::quoting::Quoting;
use nene_ledger_core::text_encoding::TextEncoding;

use crate::reading_choice::ReadingChoice;

/// 区切りの候補（カンマ・タブ・セミコロン・縦棒・空白・キャレット・コロン）。
const DELIMITERS: &[u8; 7] = b",\t;| ^:";
/// 見出しの行の候補の数（「見出しなし」の後に 1〜10 行目）。
const HEADER_ROWS: u64 = 10;

/// 開き直すときに選ぶ、読み方の 1 つの面（OPEN-01・05・09・11）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReadingAspect {
    /// 文字コード。
    Encoding,
    /// 区切り。
    Delimiter,
    /// 引用符の扱い。
    Quoting,
    /// 見出しの行。
    Header,
}

impl ReadingAspect {
    /// 候補の一覧（決まった順）。
    #[must_use]
    pub fn options(self) -> Vec<ReadingChoice> {
        match self {
            Self::Encoding => [
                TextEncoding::Utf8,
                TextEncoding::Utf8WithBom,
                TextEncoding::ShiftJis,
                TextEncoding::EucJp,
                TextEncoding::Windows1252,
                TextEncoding::Utf16LeWithBom,
                TextEncoding::Utf16BeWithBom,
                TextEncoding::Utf16Le,
                TextEncoding::Utf16Be,
            ]
            .into_iter()
            .map(ReadingChoice::Encoding)
            .collect(),
            Self::Delimiter => DELIMITERS
                .iter()
                .copied()
                .map(ReadingChoice::Delimiter)
                .collect(),
            Self::Quoting => [
                (Quoting::Doubled, b'"'),
                (Quoting::Doubled, b'\''),
                (Quoting::QuotedBackslash, b'"'),
                (Quoting::Backslash, b'"'),
                (Quoting::Plain, b'"'),
            ]
            .into_iter()
            .map(|(quoting, quote)| ReadingChoice::quoting(quoting, quote))
            .collect(),
            Self::Header => core::iter::once(ReadingChoice::Header(None))
                .chain((0..HEADER_ROWS).map(|record| ReadingChoice::Header(Some(record))))
                .collect(),
        }
    }

    /// 段の名前（候補が無いときの知らせ・件数）。
    #[must_use]
    pub fn noun(self) -> &'static str {
        match self {
            Self::Encoding => "文字コード",
            Self::Delimiter => "区切り",
            Self::Quoting => "引用符の扱い",
            Self::Header => "見出しの行",
        }
    }
}
