//! Git 規約を検証する範囲。

/// 検証する枝と基点。ローカルでは `origin/main..HEAD`、CI では PR のメタデータから。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GitRange {
    branch: String,
    base: String,
}

impl GitRange {
    /// 枝の名前と、`rev-list <base>..HEAD` の基点を保持する。
    pub(crate) fn new(branch: String, base: String) -> Self {
        Self { branch, base }
    }

    /// 検証する枝の名前。
    pub(crate) fn branch(&self) -> &str {
        &self.branch
    }

    /// `rev-list` の基点（`origin/main` か PR の base SHA）。
    pub(crate) fn base(&self) -> &str {
        &self.base
    }
}
