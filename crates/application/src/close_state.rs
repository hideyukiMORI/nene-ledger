//! 閉じる手順の状態（ADR 0026 の決定 7）。

/// 閉じる手順のどこにいるか。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseState {
    /// 閉じようとしていない。
    Open,
    /// 未保存の書き換えがあるので、どうするかを尋ねている。
    Asking,
    /// 保存して閉じると選んだ。保存が終われば閉じる（失敗したら `Open` に戻る）。
    AfterSave,
    /// 閉じると決まった（UI が窓を閉じる）。
    Finished,
}
