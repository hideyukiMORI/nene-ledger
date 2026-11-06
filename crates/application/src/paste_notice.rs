//! 表への貼り付けの知らせ（ADR 0029）。

/// 表への貼り付けの知らせ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasteNotice {
    /// 貼った。貼った範囲の行と列の数と、表の端を越えて足した行と列の数。
    Pasted {
        /// 貼った行の数。
        rows: u64,
        /// 貼った列の数。
        columns: usize,
        /// 足した行の数。
        added_rows: u64,
        /// 足した列の数。
        added_columns: usize,
        /// 切り取った元を空にした（動かした・ADR 0029 の決定 5）。
        moved: bool,
    },
    /// 切り取った（`rows` 行 × `columns` 列・貼るまで元は残る）。
    Cut {
        /// 行の数。
        rows: u64,
        /// 列の数。
        columns: usize,
    },
    /// 離れた範囲は切り取らない（Excel と同じ）。
    CutManyRanges,
    /// 上限（100 万セル・64 MiB）を超えるので、何も書かなかった（ADR 0029 の決定 4）。
    TooLarge,
    /// 索引を作り終えるまで、今読めている所より下に届く貼り付けはできない（足す行の数が分からない）。
    AwaitIndex,
}
