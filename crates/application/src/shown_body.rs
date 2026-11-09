//! 詳細ペインの本文の見せる所（GitHub Issue #454）。

use alloc::string::String;
use alloc::vec::Vec;

/// 詳細ペインの本文の見せる行（折り返した `first` 行目から）と、最後に添える断り書き（値が無い・あと N 行）、折り返した
/// 行の全部の数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ShownBody {
    lines: Vec<String>,
    note: Option<String>,
    total: usize,
    first: usize,
}

impl ShownBody {
    /// 見せる行 `lines`・断り書き `note`・全部の行の数 `total`・見せる最初の行 `first`。
    pub(crate) const fn new(
        lines: Vec<String>,
        note: Option<String>,
        total: usize,
        first: usize,
    ) -> Self {
        Self {
            lines,
            note,
            total,
            first,
        }
    }

    /// 断り書きだけ（値が無いとき）。
    pub(crate) fn note_only(note: &str) -> Self {
        Self::new(Vec::new(), Some(String::from(note)), 0, 0)
    }

    /// 折り返した行の全部の数。
    pub(crate) const fn total(&self) -> usize {
        self.total
    }

    /// 見せる最初の行。
    pub(crate) const fn first(&self) -> usize {
        self.first
    }

    /// 見せる行と断り書き。
    pub(crate) fn into_lines(self) -> (Vec<String>, Option<String>) {
        (self.lines, self.note)
    }
}
