//! 一時ファイルで原本を置き換えるポート（ADR 0025 の決定 4）。

use crate::replace_failure::ReplaceFailure;

/// 一時ファイルで原本を置き換えるポート。Linux の実装は store（`rename`）、Windows の実装は adapters-windows
/// （`ReplaceFileW` と再試行）にあり、合成ルートが OS で選んで store に渡す。
pub trait Replacing {
    /// ファイルの場所（std の道筋）。application は `no_std` なので、型は実装が決める。
    type Location: ?Sized;

    /// `replacement`（同じフォルダの一時ファイル）を、`target` の名前に置き換える。成功したら `replacement` は無くなる。
    ///
    /// # Errors
    ///
    /// [`ReplaceFailure`] のとおり。
    fn replace(
        &self,
        replacement: &Self::Location,
        target: &Self::Location,
    ) -> Result<(), ReplaceFailure>;
}
