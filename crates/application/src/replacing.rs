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

    /// フォルダ `folder` のあるボリュームの、今の利用者が使える空きのバイト数（保存を書き始める前に確かめる・SAVE-02・
    /// ADR 0025 の決定 2・GitHub Issue #515）。OS から読めない・読む口が無いなら `None`（確かめずに書く）。既定は `None`
    /// （std には空きを読む API が無い。Unix は `rustix` を足すかを M2 の終わりに決める・ADR 0025 の施主の判断 6）。
    fn free_space(&self, _folder: &Self::Location) -> Option<u64> {
        None
    }
}
