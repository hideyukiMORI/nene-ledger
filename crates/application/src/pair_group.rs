//! 全部の置換の内訳の、「前 → 後」の組のまとまり（GitHub Issue #299）。

use alloc::collections::BTreeMap;
use alloc::string::String;

/// 「前 → 後」の組のまとまり（セル全体が変わる組か、一部だけ変わる組か）：いちばん多い組・組の数・セルの数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PairGroup {
    top: (String, String, usize),
    kinds: usize,
    cells: usize,
}

impl PairGroup {
    /// 組の数え `counts`（前・後 → セルの数）から。空なら `None`。いちばん多い組は、同じ数なら前の値・後の値の順で先のもの
    /// （`max_by_key` は同じなら後を返すので逆から）。
    pub(crate) fn of(counts: &BTreeMap<(&str, &str), usize>) -> Option<Self> {
        let (&(before, after), &count) = counts.iter().rev().max_by_key(|&(_, &count)| count)?;
        Some(Self {
            top: (String::from(before), String::from(after), count),
            kinds: counts.len(),
            cells: counts
                .values()
                .fold(0_usize, |sum, &cells| sum.saturating_add(cells)),
        })
    }

    /// いちばん多い組（前・後・セルの数）。同じ数なら前の値・後の値の順で先のもの。
    #[must_use]
    pub const fn top(&self) -> &(String, String, usize) {
        &self.top
    }

    /// 組の数。
    #[must_use]
    pub const fn kinds(&self) -> usize {
        self.kinds
    }

    /// セルの数。
    #[must_use]
    pub const fn cells(&self) -> usize {
        self.cells
    }
}
