//! 保存の計画の書き先（ADR 0025 の決定 1・GitHub Issue #130）。

/// 保存の計画の書き先。計画を作る側は、原本の区間を写すことと文字列を書くことを、書く順に渡す。
///
/// 記憶に歩の列を持つ書き先（[`crate::save_plan::SavePlan`]）と、一時ファイルに直に書く書き先（store）がある。
/// 書き先が書けなくなったら、それを覚えて以後の歩を捨て、[`Self::stopped`] で知らせる（計画を作る側は早めにやめる）。
pub trait PlanSink {
    /// 原本の `offset` から `length` バイトを写す。
    fn copy(&mut self, offset: u64, length: u64);

    /// 文字列 `text` を書く（書き先がファイルの文字コードで符号化する）。
    fn text(&mut self, text: &str);

    /// 書けなくなって、以後の歩を捨てているか。既定は `false`。
    fn stopped(&self) -> bool {
        false
    }
}
