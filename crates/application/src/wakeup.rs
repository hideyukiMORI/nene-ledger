//! 窓を起こす口。

/// 作業のスレッドから画面のスレッドを起こす口（port・ADR 0014 の決定 3）。
///
/// X11 は `ClientMessage`、Windows は `PostMessage` で実装する。起こすだけで、状態は渡さない
/// （画面のスレッドが起きてから `Session::refresh` で取り込む）。
pub trait Wakeup: Send + Sync {
    /// 画面のスレッドを起こす。何度呼んでもよい（届く前の合図は 1 つにまとめてよい）。
    fn wake(&self);
}
