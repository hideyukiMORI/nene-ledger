//! 直し始める先（カーソルのセルか、カーソルの列の見出し・EDIT-04・GitHub Issue #496）。

/// [`crate::intent::Intent::Edit`] が直し始める先。意図の種類を増やさずに、既にある意図に持たせる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditTarget {
    /// カーソルのセル（F2・ADR 0026 の決定 1）。編集中なら入力と編集の状態を替える。
    Cell,
    /// カーソルの列の見出し（パレット「列の名前を変える」）。見出しが 2〜3 行なら一番下の行（ADR 0034）。
    Header,
}
