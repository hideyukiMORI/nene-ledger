//! 貼り付けを取り込んだ後に、合成ルートがすること（ADR 0029 の決定 4）。

/// 貼り付けを取り込んだ後に、合成ルートがすること。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasteRequest {
    /// 貼り終えたか、貼らずに知らせた（描き直すだけ）。
    Done,
    /// 行の並びの位置 `first_line` から `count` 行を読んで `Document::paste_rows` に渡す（書き換える前の値と原本の値を
    /// 履歴に残すため。窓の外の行も同じ 2 段・コピーと同じ）。
    Read {
        /// 最初の行の位置。
        first_line: u64,
        /// 読む行の数（見出しの行を含むことがある・0 なら読まずに渡す）。
        count: u64,
    },
}
