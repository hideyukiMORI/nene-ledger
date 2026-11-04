//! 一括処理の進み具合。

/// 一括処理の進み具合（ADR 0014 の決定 3）。読んだ位置と全体の大きさ、長すぎる引用のセルの知らせ（ADR 0018）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JobProgress {
    done: u64,
    total: u64,
    long_quoted_cell: Option<u64>,
    records: u64,
}

impl JobProgress {
    /// 進み具合を作る。`long_quoted_cell` は、今読んでいる引用の中のセルが長すぎるときのその引用符の位置。
    #[must_use]
    pub fn new(done: u64, total: u64, long_quoted_cell: Option<u64>) -> Self {
        Self {
            done,
            total,
            long_quoted_cell,
            records: 0,
        }
    }

    /// 読んだ位置。
    #[must_use]
    pub fn done(self) -> u64 {
        self.done
    }

    /// 全体の大きさ。
    #[must_use]
    pub fn total(self) -> u64 {
        self.total
    }

    /// 長すぎる引用のセルの引用符の位置（閉じない引用符かもしれない）。
    #[must_use]
    pub fn long_quoted_cell(self) -> Option<u64> {
        self.long_quoted_cell
    }

    /// 百分率（0〜100）。全体が 0 なら 100。読んだ位置が全体を超えても 100（開いた後にファイルが伸びた）。
    #[must_use]
    pub fn percent(self) -> u64 {
        if self.total == 0 {
            return 100;
        }
        // 掛け算があふれないよう u128 で計算する（u64 の範囲では 100 倍しても u128 に収まる）。
        let percent = u128::from(self.done.min(self.total)) * 100 / u128::from(self.total);
        u64::try_from(percent).unwrap_or(100)
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
