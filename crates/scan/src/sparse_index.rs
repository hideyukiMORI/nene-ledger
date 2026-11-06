//! まばらな行の索引。

use nene_ledger_core::line_ending_counts::LineEndingCounts;

use crate::mark::Mark;

/// 目印を置く間隔（レコードの数）。ADR 0017 の決定 2。
pub const MARK_INTERVAL: u64 = 1024;

/// まばらな行の索引。`MARK_INTERVAL` 件ごとのレコードの始まりだけを持つ（ADR 0017）。
///
/// 1 億行で目印は約 10 万個。メモリは行の数ではなく目印の数に比例する（BIG-04）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SparseIndex {
    marks: Vec<u64>,
    records: u64,
    bytes: u64,
    unclosed_quotes: Vec<u64>,
    line_endings: LineEndingCounts,
}

impl SparseIndex {
    /// 索引を作る。`marks` はレコード 0・`MARK_INTERVAL`・… の始まりのバイト位置。
    #[must_use]
    pub(crate) fn new(marks: Vec<u64>, records: u64, bytes: u64) -> Self {
        Self {
            marks,
            records,
            bytes,
            unclosed_quotes: Vec::new(),
            line_endings: LineEndingCounts::new(0, 0, 0),
        }
    }

    /// 作り始める前の索引（目印もレコードも無い）。索引を待たずに先頭を出すときに使う（BIG-01）。
    /// 範囲の取り出しは、目印が無ければ BOM の後ろから切り進める。
    #[must_use]
    pub fn empty() -> Self {
        Self::new(Vec::new(), 0, 0)
    }

    /// 閉じなかった引用符の位置を添える。
    #[must_use]
    pub(crate) fn with_unclosed_quotes(self, unclosed_quotes: Vec<u64>) -> Self {
        Self {
            unclosed_quotes,
            ..self
        }
    }

    /// ファイル全体のレコードの改行の内訳を添える（OPEN-08）。
    #[must_use]
    pub(crate) fn with_line_endings(self, line_endings: LineEndingCounts) -> Self {
        Self {
            line_endings,
            ..self
        }
    }

    /// ファイル全体のレコードの改行の内訳（引用の中の改行は数えない・OPEN-08）。作り始める前の索引は 0。
    #[must_use]
    pub fn line_endings(&self) -> LineEndingCounts {
        self.line_endings
    }

    /// 目印のバイト位置。`i` 番目はレコード `i × MARK_INTERVAL` の始まり。
    #[must_use]
    pub fn marks(&self) -> &[u64] {
        &self.marks
    }

    /// レコードの数（物理的な行の数ではない・ADR 0017 の決定 5）。
    #[must_use]
    pub fn records(&self) -> u64 {
        self.records
    }

    /// 読んだバイト数（ファイルの大きさ）。
    #[must_use]
    pub fn bytes(&self) -> u64 {
        self.bytes
    }

    /// 入力の終わりまで閉じなかった引用符のバイト位置（ADR 0018 の決定 1）。
    ///
    /// `UnclosedQuote::ToEnd` では高々 1 つで、最後のレコードがそこから入力の終わりまで続く。
    /// `UnclosedQuote::Literal` では文字として読んだ引用符の全部（位置の順）。
    #[must_use]
    pub fn unclosed_quotes(&self) -> &[u64] {
        &self.unclosed_quotes
    }

    /// `record` 以前で最も近い目印。目印が無ければ（レコードが無い・作り始めたばかり）`None`。
    #[must_use]
    pub fn mark_at_or_before(&self, record: u64) -> Option<Mark> {
        let wanted = usize::try_from(record / MARK_INTERVAL).unwrap_or(usize::MAX);
        let position = wanted.min(self.marks.len().saturating_sub(1));
        let &offset = self.marks.get(position)?;
        // 目印の番号は usize から u64 へ必ず収まる（目印の数はレコードの数より少ない）。
        let number = u64::try_from(position).unwrap_or(u64::MAX);
        Some(Mark::new(number.saturating_mul(MARK_INTERVAL), offset))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_nearest_mark_at_or_before_a_record() {
        let index = SparseIndex::new(vec![0, 500, 900], 2100, 1000);
        assert_eq!(index.mark_at_or_before(0), Some(Mark::new(0, 0)));
        assert_eq!(index.mark_at_or_before(1023), Some(Mark::new(0, 0)));
        assert_eq!(index.mark_at_or_before(1024), Some(Mark::new(1024, 500)));
        assert_eq!(index.mark_at_or_before(2099), Some(Mark::new(2048, 900)));
        assert_eq!(
            index.mark_at_or_before(u64::MAX),
            Some(Mark::new(2048, 900))
        );
    }

    #[test]
    fn empty_index_has_no_mark() {
        let index = SparseIndex::empty();
        assert_eq!(index.mark_at_or_before(5), None);
        assert_eq!((index.records(), index.bytes()), (0, 0));
    }
}
