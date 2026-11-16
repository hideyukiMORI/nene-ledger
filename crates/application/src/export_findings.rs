//! 書き出しで書けないセルの数と、最初の所（SAVE-04・ADR 0036 の決定 5）。

/// 書き出しで書けない（文字コードで表せない字のある）セルの数と、最初の所。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExportFindings {
    count: u64,
    first: Option<(u64, usize)>,
}

impl ExportFindings {
    /// 1 つも無い。
    #[must_use]
    pub const fn none() -> Self {
        Self {
            count: 0,
            first: None,
        }
    }

    /// 表の行の位置 `row`・列の位置 `column` のセルを数える。
    pub fn add(&mut self, row: u64, column: usize) {
        self.count = self.count.saturating_add(1);
        self.first = self.first.or(Some((row, column)));
    }

    /// セルの数。
    #[must_use]
    pub const fn count(&self) -> u64 {
        self.count
    }

    /// 最初のセルの所（書き出す表の行の位置・列の位置。どちらも 0 始まりで、前書きと見出しの行も数える）。
    #[must_use]
    pub const fn first(&self) -> Option<(u64, usize)> {
        self.first
    }
}
