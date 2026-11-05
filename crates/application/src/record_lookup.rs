//! 原本のレコードを番号で引くポート（ADR 0025 の決定 1）。

use alloc::vec::Vec;

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

    /// 原本のレコード `first` から続いた `count` 個（ファイルの終わりで短くなる）。列を足し消しした表の保存は
    /// 全部の行を組み立て直すので、続いた行をまとめて読める実装はこれを上書きする（既定は 1 つずつ引く）。
    ///
    /// # Errors
    ///
    /// 読み出しの失敗。
    fn records(&mut self, first: u64, count: usize) -> Result<Vec<LocatedRecord>, Self::Failure> {
        let mut found = Vec::new();
        for number in (first..).take(count) {
            match self.record(number)? {
                Some(record) => found.push(record),
                None => break,
            }
        }
        Ok(found)
    }
}
