//! 選んだ範囲の集計（VIEW-14・Issue #42）。

use alloc::string::String;
use core::ops::RangeInclusive;

use nene_ledger_core::decimal::Decimal;
use nene_ledger_core::decimal_sum::DecimalSum;
use nene_ledger_core::number_reading::{is_plain_number, read_number};

/// 選んだ範囲の集計。件数（空でないセル）・数の個数・合計（桁あふれなら `None`）・数として読めないセルの数。
/// 空のセルとセルが無いは数えない（RS-004）。数は 10 進で正確に、足す順に依らずに足す（GitHub Issue #42）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionStats {
    cells: u64,
    numbers: u64,
    sum: DecimalSum,
    not_numbers: u64,
    /// 数と読んだうち、書き方を読み替えたもの（`¥`・桁区切り・指数・全角など・GitHub Issue #319）。
    rewritten: u64,
    /// 範囲に入った行のうち、列の数が見出しと違う行（ずれた値を足しているかもしれない・GitHub Issue #329）。
    ragged_rows: u64,
    /// 空のセル（セルはあるが値が空）の数。欠損に気づけるように（GitHub Issue #389）。セルが無いは数えない（RS-004）。
    empties: u64,
    /// 数の最小と最大（GitHub Issue #389）。比べて桁あふれしたら `None` のまま（見せない）。
    extremes: Option<(Decimal, Decimal)>,
    /// 最小と最大を比べられなかった（桁あふれ）。
    extremes_lost: bool,
}

impl SelectionStats {
    /// 行 `rows`（復号した値）の、列 `columns` のセルを集計する。
    #[must_use]
    pub fn of(rows: &[&[String]], columns: &RangeInclusive<usize>) -> Self {
        let mut stats = Self::empty();
        for row in rows {
            for value in columns.clone().filter_map(|column| row.get(column)) {
                stats.add(value);
            }
        }
        stats
    }

    /// まだ何も数えていない集計（裏の仕事で 1 つずつ足していく）。
    #[must_use]
    pub fn empty() -> Self {
        Self {
            cells: 0,
            numbers: 0,
            sum: DecimalSum::ZERO,
            not_numbers: 0,
            rewritten: 0,
            ragged_rows: 0,
            empties: 0,
            extremes: None,
            extremes_lost: false,
        }
    }

    /// 行 `row`（復号した値）の、列 `columns` のセルを足す。セルが無い列は数えない。
    pub fn add_row(&mut self, row: &[String], columns: &[RangeInclusive<usize>]) {
        self.add_cells(|column| row.get(column).map(String::as_str), columns);
    }

    /// 列 → セルの値（無ければ `None`）の `cell` で引ける 1 行の、列 `columns` のセルを足す（セルを `String` の並びで
    /// 持たない行・GitHub Issue #22）。セルが無い列は数えない。
    pub fn add_cells<'value, F: Fn(usize) -> Option<&'value str>>(
        &mut self,
        cell: F,
        columns: &[RangeInclusive<usize>],
    ) {
        for value in columns.iter().cloned().flatten().filter_map(cell) {
            self.add(value);
        }
    }

    /// セルの値（復号した値）`value` を 1 つ足す。空のセルは件数に入れず、空の数に数える。
    pub fn add(&mut self, value: &str) {
        if value.is_empty() {
            self.empties = self.empties.saturating_add(1);
            return;
        }
        self.cells = self.cells.saturating_add(1);
        match read_number(value) {
            Some(number) => {
                self.numbers = self.numbers.saturating_add(1);
                self.sum.add(number);
                self.widen(number);
                if !is_plain_number(value) {
                    self.rewritten = self.rewritten.saturating_add(1);
                }
            }
            None => self.not_numbers = self.not_numbers.saturating_add(1),
        }
    }

    /// 最小と最大に数 `number` を入れる。比べて桁あふれしたら、以後は最小と最大を見せない。
    fn widen(&mut self, number: Decimal) {
        if self.extremes_lost {
            return;
        }
        let Some((low, high)) = self.extremes else {
            self.extremes = Some((number, number));
            return;
        };
        if let (Some(below), Some(above)) = (number.checked_cmp(low), number.checked_cmp(high)) {
            self.extremes = Some((
                if below.is_lt() { number } else { low },
                if above.is_gt() { number } else { high },
            ));
        } else {
            self.extremes = None;
            self.extremes_lost = true;
        }
    }

    /// 数の最小と最大。数が無いか、比べて桁あふれしたら `None`。
    #[must_use]
    pub fn extremes(self) -> Option<(Decimal, Decimal)> {
        self.extremes
    }

    /// 空のセルの数（セルはあるが値が空）。
    #[must_use]
    pub fn empties(self) -> u64 {
        self.empties
    }

    /// 数と読んだうち、書き方を読み替えたもの（`¥3`・`1,200`・`1.5e3` など）の数（GitHub Issue #319）。
    #[must_use]
    pub fn rewritten(self) -> u64 {
        self.rewritten
    }

    /// 範囲に入った 1 行のセルの数 `cells` が見出しのセルの数 `expected` と違えば、列の数が違う行として数える
    /// （GitHub Issue #329）。
    pub fn add_shape(&mut self, cells: usize, expected: usize) {
        if cells != expected {
            self.ragged_rows = self.ragged_rows.saturating_add(1);
        }
    }

    /// 範囲に入った、列の数が見出しと違う行の数。
    #[must_use]
    pub fn ragged_rows(self) -> u64 {
        self.ragged_rows
    }

    /// 件数（空でないセル）。
    #[must_use]
    pub fn cells(self) -> u64 {
        self.cells
    }

    /// 数として読めたセルの数。
    #[must_use]
    pub fn numbers(self) -> u64 {
        self.numbers
    }

    /// 合計。桁あふれなら `None`。
    #[must_use]
    pub fn sum(self) -> Option<Decimal> {
        self.sum.total()
    }

    /// 平均（数の個数で割る・小数 6 桁で四捨五入して末尾の 0 を落とす）。数が無いか桁あふれなら `None`。
    #[must_use]
    pub fn average(self) -> Option<Decimal> {
        self.sum()?.average(self.numbers).map(Decimal::trimmed)
    }

    /// 数として読めなかったセルの数（空でないもの）。
    #[must_use]
    pub fn not_numbers(self) -> u64 {
        self.not_numbers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use alloc::vec::Vec;

    fn table(cells: &[&[&str]]) -> Vec<Vec<String>> {
        cells
            .iter()
            .map(|row| row.iter().map(|&cell| String::from(cell)).collect())
            .collect()
    }

    fn stats(cells: &[&[&str]], columns: RangeInclusive<usize>) -> SelectionStats {
        let rows = table(cells);
        let slices: Vec<&[String]> = rows.iter().map(Vec::as_slice).collect();
        SelectionStats::of(&slices, &columns)
    }

    #[test]
    fn numbers_are_summed_exactly_and_the_rest_are_counted() {
        let figures = stats(&[&["0.1", "x"], &["0.2", ""], &["1,234", "5"]], 0..=1);
        // 空でないセル 5（空の 1 つは数えない）。数は 0.1・0.2・1,234（桁区切り・GitHub Issue #41）・5 の 4 つ、数でないのは x。
        assert_eq!(figures.cells(), 5);
        assert_eq!(figures.numbers(), 4);
        assert_eq!(figures.not_numbers(), 1);
        assert_eq!(figures.sum(), Some(Decimal::new(12_393, 1)));
        // 1239.3 / 4 = 309.825。
        assert_eq!(
            figures.average().map(Decimal::to_plain),
            Some(String::from("309.825"))
        );
    }

    #[test]
    fn missing_cells_and_columns_outside_the_range_are_ignored() {
        let figures = stats(&[&["1", "2", "3"], &["4"]], 1..=2);
        assert_eq!((figures.cells(), figures.numbers()), (2, 2));
        assert_eq!(figures.sum(), Some(Decimal::new(5, 0)));
        assert_eq!(figures.average(), Some(Decimal::new(25, 1)));
        let words = stats(&[&["a"], &["b"]], 0..=0);
        assert_eq!((words.numbers(), words.average()), (0, None));
    }

    #[test]
    fn an_overflowing_sum_is_reported_as_none() {
        let huge = "9".repeat(38);
        let figures = stats(&[&[huge.as_str()], &[huge.as_str()]], 0..=0);
        assert_eq!(figures.numbers(), 2);
        assert_eq!(figures.sum(), None);
        assert_eq!(figures.average(), None);
    }

    /// 合計は足す順に依らない（GitHub Issue #42。前は 1e21・1e-18・-1e21 の順で「桁あふれ」）。
    #[test]
    fn the_sum_does_not_depend_on_the_order_of_the_cells() {
        let (huge, tiny, minus) = (
            "1000000000000000000000",
            "0.000000000000000001",
            "-1000000000000000000000",
        );
        for order in [[huge, tiny, minus], [huge, minus, tiny]] {
            let figures = stats(&[&[order[0]], &[order[1]], &[order[2]]], 0..=0);
            assert_eq!(
                figures.sum().map(Decimal::to_plain),
                Some(String::from("0.000000000000000001")),
                "{order:?}"
            );
        }
    }

    /// 範囲の数の最小と最大、空のセルの数を数える（アナリストのペルソナの相談・GitHub Issue #389）。セルが無いは空に数えない。
    #[test]
    fn extremes_and_empty_cells_are_counted() {
        let rows: Vec<Vec<String>> = [["12.5", ""], ["-3", "x"], ["1,200", "7"]]
            .iter()
            .map(|row| row.iter().map(|&value| String::from(value)).collect())
            .collect();
        let mut short = rows.clone();
        short.push(vec![String::from("4")]);
        let borrowed: Vec<&[String]> = short.iter().map(Vec::as_slice).collect();
        let stats = SelectionStats::of(&borrowed, &(0..=1));
        assert_eq!(
            stats.extremes(),
            Some((Decimal::new(-3, 0), Decimal::new(1_200, 0)))
        );
        assert_eq!(stats.empties(), 1);
        assert_eq!(stats.cells(), 6);
        assert_eq!(SelectionStats::empty().extremes(), None);
    }
}
