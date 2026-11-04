//! 方言の候補 1 つ。

use crate::dialect::Dialect;

/// 方言の候補と、その点数（千分率）と、いちばん多い列の数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DialectCandidate {
    dialect: Dialect,
    score: u32,
    columns: usize,
}

impl DialectCandidate {
    /// 候補を作る。`score` は 0〜1000 に丸める。
    #[must_use]
    pub fn new(dialect: Dialect, score: u32, columns: usize) -> Self {
        Self {
            dialect,
            score: score.min(1000),
            columns,
        }
    }

    /// 方言。
    #[must_use]
    pub fn dialect(self) -> Dialect {
        self.dialect
    }

    /// 点数（0〜1000）。行の形の一貫性 × セルの形のもっともらしさ。
    #[must_use]
    pub fn score(self) -> u32 {
        self.score
    }

    /// いちばん多い列の数。
    #[must_use]
    pub fn columns(self) -> usize {
        self.columns
    }
}
