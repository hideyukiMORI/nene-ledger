//! ファイルのドラッグ＆ドロップのイベントへの応え（CLIP-04・GitHub Issue #243）。

/// XDND のイベントへの応え。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DropStep {
    /// 取り決めのやり取りだけ（描き直さない）。
    Handled,
    /// 落とされたファイルの道筋を受け取った（空のこともある）。
    Dropped(Vec<String>),
}
