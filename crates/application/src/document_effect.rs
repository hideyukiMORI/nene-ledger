//! 意図を取り込んだ後に、合成ルートがすること。

/// 意図を取り込んだ後に、合成ルートがすること。読むのは合成ルート（原本と索引を持つ）だけ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentEffect {
    /// 何もしない（描き直すだけ）。
    Stay,
    /// 本体の `first_body` 番目からの行を読んで `Document::show` に渡す（縦の表示が動いた）。
    Read {
        /// 本体の何番目から。
        first_body: u64,
    },
    /// 書き換え（`Document::edits`）を保存する（Ctrl+S）。始めたら `Document::begin_saving`、終わったら
    /// `Document::saved` か `Document::save_failed` を呼ぶ。
    Save,
    /// 裏の集計を止める（Esc）。
    StopTotals,
    /// 行の並びの位置 `first_line` から `count` 行を読んで `Document::copy_text` に渡す（コピー）。
    Copy {
        /// 最初の行の位置。
        first_line: u64,
        /// 読む行の数（見出しの行を含むことがある）。
        count: u64,
    },
}
