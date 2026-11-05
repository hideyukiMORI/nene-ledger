//! 行の並び（ADR 0024 の決定 2・層 1・EDIT-03）。原本の行の区間と足した行の列を並べた piece table。
//!
//! 行の挿入と削除は区間を割って並べ替えるだけで、原本は読まない。記憶は編集の回数に比例する（BIG-13）。
//! 位置（画面の行の順）は 0 始まりで、前書きと見出しのレコードも数える（原本のレコードの番号と同じ数え方）。
//! 🔴 位置からの引き当ては区間を頭から辿る（区間の数に比例）。区間が数十万になる編集の速さは、画面に入れる Issue で測る。

use alloc::vec;
use alloc::vec::Vec;

use crate::order_run::OrderRun;
use crate::row_id::RowId;

/// 行の並び。区間の列と、次に足す行の通し番号。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowOrder {
    runs: Vec<OrderRun>,
    next_added: u64,
}

impl RowOrder {
    /// 開いた直後の並び（原本の頭から終わりまでの 1 区間）。
    #[must_use]
    pub fn whole() -> Self {
        Self {
            runs: vec![OrderRun::Source {
                first: 0,
                count: None,
            }],
            next_added: 0,
        }
    }

    /// 区間の列（保存の計画が頭から辿る）。
    #[must_use]
    pub fn runs(&self) -> &[OrderRun] {
        &self.runs
    }

    /// 開いた直後のままか（行を足しも消しもしていない）。
    #[must_use]
    pub fn is_whole(&self) -> bool {
        self.runs == Self::whole().runs
    }

    /// 行の数。原本のレコードの数 `source_records` から数える。
    #[must_use]
    pub fn len(&self, source_records: u64) -> u64 {
        self.runs.iter().fold(0, |sum: u64, run| {
            sum.saturating_add(run.len(source_records))
        })
    }

    /// `index` 番目の行。並びの外なら `None`（終わりまでの区間に落ちれば、原本の終わりを確かめずに返す）。
    #[must_use]
    pub fn row_at(&self, index: u64) -> Option<RowId> {
        let mut rest = index;
        for run in &self.runs {
            let open = matches!(run, OrderRun::Source { count: None, .. });
            let length = run.len(0);
            if open || rest < length {
                return run.row_at(rest);
            }
            rest = rest.saturating_sub(length);
        }
        None
    }

    /// 行 `row` の位置。並びに無ければ `None`（終わりまでの区間は、原本の終わりを確かめずに数える）。
    #[must_use]
    pub fn position_of(&self, row: RowId) -> Option<u64> {
        let mut before = 0_u64;
        for run in &self.runs {
            if let Some(offset) = offset_in(run, row) {
                return Some(before.saturating_add(offset));
            }
            before = before.saturating_add(run.len(0));
        }
        None
    }

    /// `index` 番目から `count` 行の区間（原本の区間は行の数を決めて返す）。原本を読む側が区間ごとに読む。
    /// 終わりまでの区間に落ちる分は、原本の終わりを確かめずに `count` 行まで返す（読む側で短くなる）。
    #[must_use]
    pub fn slice(&self, index: u64, count: u64) -> Vec<OrderRun> {
        let mut copy = self.clone();
        let _head = copy.remove(0, index);
        let end = copy.split(count);
        copy.runs
            .into_iter()
            .take(end)
            .map(|run| match run {
                OrderRun::Source { first, count: None } => OrderRun::Source {
                    first,
                    count: Some(count),
                },
                other @ (OrderRun::Source { .. } | OrderRun::Added(_)) => other,
            })
            .collect()
    }

    /// `index` 番目の前に `count` 行を足し、足した行を返す。並びの終わりより後ろなら終わりに足す。
    pub fn insert(&mut self, index: u64, count: u64) -> Vec<RowId> {
        let first = self.next_added;
        self.next_added = self.next_added.saturating_add(count);
        let added: Vec<u64> = (first..self.next_added).collect();
        let rows = added.iter().map(|&row| RowId::Added(row)).collect();
        self.restore(index, vec![OrderRun::Added(added)]);
        rows
    }

    /// `index` 番目から `count` 行を消し、消した区間を返す（[`Self::restore`] で同じ位置に戻せる・Undo）。
    pub fn remove(&mut self, index: u64, count: u64) -> Vec<OrderRun> {
        let start = self.split(index);
        let end = self.split(index.saturating_add(count));
        let removed: Vec<OrderRun> = self.runs.drain(start..end).collect();
        self.normalize();
        removed
    }

    /// 消した区間 `runs` を `index` 番目の前に戻す（足した行の通し番号はそのまま）。
    pub fn restore(&mut self, index: u64, runs: Vec<OrderRun>) {
        let at = self.split(index);
        let tail = self.runs.split_off(at);
        self.runs.extend(runs);
        self.runs.extend(tail);
        self.normalize();
    }

    /// `index` 番目の行の前で区間を割り、その位置の区間の番号を返す。並びより後ろなら区間の数。
    fn split(&mut self, index: u64) -> usize {
        let mut rest = index;
        let mut position = 0_usize;
        while let Some(run) = self.runs.get(position) {
            if rest == 0 {
                return position;
            }
            let open = matches!(run, OrderRun::Source { count: None, .. });
            let length = run.len(0);
            if !open && rest >= length {
                rest = rest.saturating_sub(length);
                position = position.saturating_add(1);
                continue;
            }
            let (head, tail) = split_run(run, rest);
            self.runs.remove(position);
            self.runs.insert(position, tail);
            self.runs.insert(position, head);
            return position.saturating_add(1);
        }
        position
    }

    /// 空の区間を除き、続いた区間をまとめる（同じ並びが同じ形になる・消して戻すと元の形）。
    fn normalize(&mut self) {
        let runs = core::mem::take(&mut self.runs);
        for run in runs {
            if run.is_empty() {
                continue;
            }
            if let Some(joined) = self.runs.last().and_then(|last| joined(last, &run)) {
                let _before = self.runs.pop();
                self.runs.push(joined);
            } else {
                self.runs.push(run);
            }
        }
    }
}

/// 続いた 2 つの区間を 1 つにした区間。続いていなければ `None`。
fn joined(last: &OrderRun, run: &OrderRun) -> Option<OrderRun> {
    match (last, run) {
        (
            OrderRun::Source {
                first,
                count: Some(count),
            },
            OrderRun::Source {
                first: next,
                count: rest,
            },
        ) if first.saturating_add(*count) == *next => Some(OrderRun::Source {
            first: *first,
            count: rest.map(|rest| count.saturating_add(rest)),
        }),
        (OrderRun::Added(rows), OrderRun::Added(more)) => {
            Some(OrderRun::Added(rows.iter().chain(more).copied().collect()))
        }
        (OrderRun::Source { .. } | OrderRun::Added(_), _) => None,
    }
}

/// 区間 `run` の中の、行 `row` の位置。区間に無ければ `None`。
fn offset_in(run: &OrderRun, row: RowId) -> Option<u64> {
    match (run, row) {
        (OrderRun::Source { first, count }, RowId::Source(record)) => {
            let offset = record.checked_sub(*first)?;
            count.is_none_or(|count| offset < count).then_some(offset)
        }
        (OrderRun::Added(rows), RowId::Added(added)) => rows
            .iter()
            .position(|&listed| listed == added)
            .and_then(|index| u64::try_from(index).ok()),
        (OrderRun::Source { .. }, RowId::Added(_)) | (OrderRun::Added(_), RowId::Source(_)) => None,
    }
}

/// 区間 `run` を、頭の `offset` 行とその後ろに割る。
fn split_run(run: &OrderRun, offset: u64) -> (OrderRun, OrderRun) {
    match run {
        OrderRun::Source { first, count } => (
            OrderRun::Source {
                first: *first,
                count: Some(offset),
            },
            OrderRun::Source {
                first: first.saturating_add(offset),
                count: count.map(|count| count.saturating_sub(offset)),
            },
        ),
        OrderRun::Added(rows) => {
            let at = usize::try_from(offset)
                .unwrap_or(usize::MAX)
                .min(rows.len());
            let (head, tail) = rows.split_at(at);
            (
                OrderRun::Added(head.to_vec()),
                OrderRun::Added(tail.to_vec()),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: u64 = 40;

    fn listed(order: &RowOrder) -> Vec<RowId> {
        (0..order.len(SOURCE))
            .map(|index| order.row_at(index).unwrap())
            .collect()
    }

    #[test]
    fn a_fresh_order_is_the_whole_source() {
        let order = RowOrder::whole();
        assert!(order.is_whole());
        assert_eq!(order.len(SOURCE), SOURCE);
        assert_eq!(order.row_at(7), Some(RowId::Source(7)));
        // 索引を作り終える前は、終わりを確かめずに返す。
        assert_eq!(order.row_at(1_000), Some(RowId::Source(1_000)));
    }

    #[test]
    fn inserting_and_removing_split_the_runs_without_reading_the_source() {
        let mut order = RowOrder::whole();
        let added = order.insert(2, 2);
        assert_eq!(added, vec![RowId::Added(0), RowId::Added(1)]);
        assert_eq!(order.row_at(1), Some(RowId::Source(1)));
        assert_eq!(order.row_at(2), Some(RowId::Added(0)));
        assert_eq!(order.row_at(4), Some(RowId::Source(2)));
        let removed = order.remove(1, 4);
        assert_eq!(order.row_at(1), Some(RowId::Source(3)));
        assert_eq!(order.len(SOURCE), SOURCE - 2);
        order.restore(1, removed);
        assert_eq!(order.row_at(2), Some(RowId::Added(0)));
        // 足した行を消して戻すと、元の 1 区間に戻る。
        let _gone = order.remove(2, 2);
        assert!(order.is_whole());
    }

    #[test]
    fn rows_can_be_added_at_the_end_once_the_length_is_known() {
        let mut order = RowOrder::whole();
        let _cut = order.remove(SOURCE - 1, 1);
        assert_eq!(order.len(SOURCE), SOURCE - 1);
        let added = order.insert(SOURCE - 1, 1);
        assert_eq!(order.row_at(SOURCE - 1), added.first().copied());
    }

    #[test]
    fn positions_and_slices_follow_the_runs() {
        let mut order = RowOrder::whole();
        let added = order.insert(2, 2);
        let _gone = order.remove(5, 1);
        // 並び: S0 S1 A0 A1 S2 S4 S5 …
        assert_eq!(order.position_of(RowId::Source(1)), Some(1));
        assert_eq!(order.position_of(added[1]), Some(3));
        assert_eq!(order.position_of(RowId::Source(3)), None);
        assert_eq!(order.position_of(RowId::Source(4)), Some(5));
        assert_eq!(
            order.slice(1, 5),
            vec![
                OrderRun::Source {
                    first: 1,
                    count: Some(1)
                },
                OrderRun::Added(vec![0, 1]),
                OrderRun::Source {
                    first: 2,
                    count: Some(1)
                },
                OrderRun::Source {
                    first: 4,
                    count: Some(1)
                },
            ]
        );
        // 終わりまでの区間は、求めた数まで。
        assert_eq!(
            RowOrder::whole().slice(10, 3),
            vec![OrderRun::Source {
                first: 10,
                count: Some(3)
            }]
        );
    }

    /// 決まった種から作る疑似乱数（テストの決定性）。
    struct Xorshift(u64);

    impl Xorshift {
        fn below(&mut self, bound: u64) -> u64 {
            self.0 ^= self.0 << 13_u32;
            self.0 ^= self.0 >> 7_u32;
            self.0 ^= self.0 << 17_u32;
            self.0 % bound.max(1)
        }
    }

    /// 1 手の記録（位置・行の数・消した区間。足したなら `None`）。
    type Step = (u64, u64, Option<Vec<OrderRun>>);

    /// 無作為に 1 手（足すか消す）進め、素朴な列 `model` にも同じことをする。
    fn random_step(rng: &mut Xorshift, order: &mut RowOrder, model: &mut Vec<RowId>) -> Step {
        let length = u64::try_from(model.len()).unwrap();
        let at = rng.below(length + 1);
        let place = usize::try_from(at).unwrap();
        if rng.below(2) == 0 {
            let count = 1 + rng.below(3);
            let added = order.insert(at, count);
            model.splice(place..place, added);
            return (at, count, None);
        }
        let count = rng.below(length - at + 1);
        let removed = order.remove(at, count);
        model.drain(place..usize::try_from(at + count).unwrap());
        (at, count, Some(removed))
    }

    /// 1 手戻す。
    fn undo((at, count, removed): Step, order: &mut RowOrder) {
        match removed {
            Some(runs) => order.restore(at, runs),
            None => drop(order.remove(at, count)),
        }
    }

    #[test]
    fn random_edits_match_a_plain_list_and_undo_back_to_the_whole() {
        let mut rng = Xorshift(0x0DD5_EED0_1234_5678);
        for _ in 0..300_u32 {
            let mut order = RowOrder::whole();
            let mut model: Vec<RowId> = (0..SOURCE).map(RowId::Source).collect();
            let mut steps: Vec<Step> = Vec::new();
            for _ in 0..rng.below(12) {
                steps.push(random_step(&mut rng, &mut order, &mut model));
                assert_eq!(listed(&order), model);
            }
            for step in steps.into_iter().rev() {
                undo(step, &mut order);
            }
            assert!(order.is_whole(), "{order:?}");
        }
    }
}
