//! 保存が原本のバイト列を読むポート（ADR 0025 の決定 1。store は scan に依存しない）。

/// 原本を、位置を指定して読むポート。scan の原本が実装し、合成ルートが store に渡す。
pub trait SourceBytes {
    /// 読み出しの失敗。
    type Failure;

    /// 原本の `offset` から `buffer` に読み、読んだバイト数を返す。0 はファイルの終わり。
    ///
    /// # Errors
    ///
    /// 読み出しの失敗。
    fn read_at(&mut self, offset: u64, buffer: &mut [u8]) -> Result<usize, Self::Failure>;

    /// 書き終えた後、置き換える（名前を付ける）直前に、原本が開いたときのままか。読み始めてから外から変わっていれば
    /// `false`（store は書かない）。保存先が原本と同じ普通の保存は、保存先の印で見つけるので、既定は `true`
    /// （名前を付けて保存で、原本が保存先と別のときに実装が確かめる・GitHub Issue #126）。
    fn unchanged(&mut self) -> bool {
        true
    }
}
