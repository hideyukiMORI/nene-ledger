//! 原本のレコードを番号で引くポート（ADR 0025 の決定 1）。

use crate::located_record::LocatedRecord;

/// 原本のレコードを番号で引くポート（scan が実装し、合成ルートが結ぶ）。
pub trait RecordLookup {
    /// 読み出しの失敗。
    type Failure;

    /// 原本のレコード `number`（前書きと見出しも数える）。ファイルの外なら `None`。
    ///
    /// # Errors
    ///
    /// 読み出しの失敗。
    fn record(&mut self, number: u64) -> Result<Option<LocatedRecord>, Self::Failure>;
}
