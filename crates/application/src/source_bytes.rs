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
}
