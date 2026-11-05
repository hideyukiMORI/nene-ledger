//! 編集の欄の状態（ADR 0026 の決定 1・2）。

/// 編集の欄の 2 つの状態。Excel の「入力」と「編集」と同じ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditMode {
    /// 打ち始めで入った（元の値を置き換える）。矢印は確定して動く。
    Entering,
    /// F2・ダブルクリックで入った（元の値の続きから直す）。矢印は欄の中で動く。
    Editing,
}
