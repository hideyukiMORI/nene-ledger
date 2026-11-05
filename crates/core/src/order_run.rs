//! 行の並びの 1 つの区間（ADR 0024 の決定 2・EDIT-03）。

use alloc::vec::Vec;

use crate::row_id::RowId;

/// 行の並び（`RowOrder`）の 1 つの区間。原本の連続した行か、足した行の列。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderRun {
    /// 原本のレコード `first` から `count` 行。`count` が `None` なら原本の終わりまで（索引を作り終える前でも表せる・BIG-12）。
    Source {
        /// 最初のレコードの番号。
        first: u64,
        /// 行の数。`None` は終わりまで。
        count: Option<u64>,
    },
    /// 足した行（通し番号の列）。
    Added(Vec<u64>),
}

impl OrderRun {
    /// 区間の行の数。原本の終わりまでの区間は、原本のレコードの数 `source_records` から数える。
    #[must_use]
    pub fn len(&self, source_records: u64) -> u64 {
        match self {
            Self::Source {
                count: Some(count), ..
            } => *count,
            Self::Source { first, count: None } => source_records.saturating_sub(*first),
            Self::Added(rows) => u64::try_from(rows.len()).unwrap_or(u64::MAX),
        }
    }

    /// 区間の中の `offset` 番目の行。区間の外なら `None`（終わりまでの区間は終わりを知らないので、いつも返す）。
    #[must_use]
    pub fn row_at(&self, offset: u64) -> Option<RowId> {
        match self {
            Self::Source { first, count } => count
                .is_none_or(|count| offset < count)
                .then(|| RowId::Source(first.saturating_add(offset))),
            Self::Added(rows) => usize::try_from(offset)
                .ok()
                .and_then(|index| rows.get(index))
                .map(|&added| RowId::Added(added)),
        }
    }

    /// 空の区間か（終わりまでの区間は空とみなさない）。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Source { count, .. } => *count == Some(0),
            Self::Added(rows) => rows.is_empty(),
        }
    }
}
