//! 裏の処理の進み具合。

/// 進み具合。読んでいる位置と、開いたときの大きさ（ADR 0014 の決定 3）。途中の知らせも添える（ADR 0018）。
///
/// 開いた後にファイルが伸びると `done` が `total` を超えうる。外部の変更の検出は FR-019 の仕事。
/// `UnclosedQuote::Literal` で閉じなかった引用符から読み直すときは、`done` がその位置まで戻る。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    done: u64,
    total: u64,
    long_quoted_cell: Option<u64>,
    records: u64,
}

impl Progress {
    /// 進み具合を作る。`long_quoted_cell` は、今読んでいる引用の中のセルが長すぎるときの、その引用符の位置。
    #[must_use]
    pub fn new(done: u64, total: u64, long_quoted_cell: Option<u64>) -> Self {
        Self {
            done,
            total,
            long_quoted_cell,
            records: 0,
        }
    }

    /// 読んでいる位置（そこまで読み終えた）。
    #[must_use]
    pub fn done(self) -> u64 {
        self.done
    }

    /// 引用の中のセルが `RECORD_READ_LIMIT` を超えて続いているときの、その引用符の位置。
    /// 閉じない引用符かもしれないという知らせで、読み方は変えない（ADR 0018 の決定 1）。
    #[must_use]
    pub fn long_quoted_cell(self) -> Option<u64> {
        self.long_quoted_cell
    }

    /// 開いたときの大きさ。
    #[must_use]
    pub fn total(self) -> u64 {
        self.total
    }

    /// ここまでに数えたレコードの数を添える（索引を作っている間の総数の見積もりに使う）。
    #[must_use]
    pub fn with_records(self, records: u64) -> Self {
        Self { records, ..self }
    }

    /// ここまでに数えたレコードの数（索引の仕事でなければ 0）。
    #[must_use]
    pub fn records(self) -> u64 {
        self.records
    }
}
