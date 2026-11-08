//! 1 行のセルの文字列を、1 本の文字列と区切りの位置で持つ（GitHub Issue #22・窓の記憶）。

use alloc::string::String;
use alloc::vec::Vec;
use core::ops::Range;

use crate::held_columns::HeldColumns;

/// 1 行のセルの文字列。セルごとに `String` を持つと、1 セルあたり 24 バイトの頭とヒープの割り当て（短い値でも 30 バイトほど）
/// が要る。数万列の表で窓の記憶が GB 単位になった（GitHub Issue #22）。ここでは全部のセルを 1 本の文字列に続けて置き、
/// セルの終わりの位置だけを持つ（1 セルあたり 8 バイト ＋ 字）。
///
/// 窓の行は、見えている列の範囲（＋余白）のセルだけを持つことがある（ADR 0033 の決定 1）。そのときは行の列の数と持っている
/// 列の範囲を覚え、持っていない列を「セルが無い」と取り違えないよう [`Self::holds`] で確かめられるようにする。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CellRow {
    text: String,
    ends: Vec<usize>,
    /// 一部の列だけを持つなら、行の列の数と持っている列の範囲。全部の列を持つなら `None`。
    held: Option<HeldColumns>,
}

impl CellRow {
    /// セルの無い行（足した行を読んだ行の並びに入れるとき）。
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            text: String::new(),
            ends: Vec::new(),
            held: None,
        }
    }

    /// セルを `cells` 個・字を `bytes` バイトまで入れられる、セルの無い行（後ろに [`Self::push_with`] で足す）。
    #[must_use]
    pub fn with_capacity(cells: usize, bytes: usize) -> Self {
        Self {
            text: String::with_capacity(bytes),
            ends: Vec::with_capacity(cells),
            held: None,
        }
    }

    /// 後ろにセルを 1 つ足す。セルの字は `write` が行の文字列の後ろに書き足す（セルごとの `String` を作らない）。
    pub fn push_with<F: FnOnce(&mut String)>(&mut self, write: F) {
        write(&mut self.text);
        self.ends.push(self.text.len());
    }

    /// 余った場所を手放す。
    pub fn shrink_to_fit(&mut self) {
        self.text.shrink_to_fit();
        self.ends.shrink_to_fit();
    }

    /// セルの文字列の並び `cells` から作る。
    #[must_use]
    pub fn new<S: AsRef<str>>(cells: &[S]) -> Self {
        let bytes = cells.iter().map(|cell| cell.as_ref().len()).sum();
        let mut text = String::with_capacity(bytes);
        let mut ends = Vec::with_capacity(cells.len());
        for cell in cells {
            text.push_str(cell.as_ref());
            ends.push(text.len());
        }
        Self {
            text,
            ends,
            held: None,
        }
    }

    /// 各セルを `write`（セル → 書き足す先）で書き替えた行（表示の文字列など）。セルごとに `String` を作らない。持っている
    /// 列の範囲はそのまま。
    #[must_use]
    pub fn mapped<F: Fn(&str, &mut String)>(&self, write: F) -> Self {
        let mut text = String::with_capacity(self.text.len());
        let mut ends = Vec::with_capacity(self.ends.len());
        for cell in self.held_cells() {
            write(cell, &mut text);
            ends.push(text.len());
        }
        text.shrink_to_fit();
        Self {
            text,
            ends,
            held: self.held.clone(),
        }
    }

    /// セルの文字列を順に出す `cells` から作る（借りた `&str` でも、復号したての `String` でもよい）。
    #[must_use]
    pub fn collect<S: AsRef<str>, I: Iterator<Item = S>>(cells: I) -> Self {
        let mut text = String::new();
        let mut ends = Vec::new();
        for cell in cells {
            text.push_str(cell.as_ref());
            ends.push(text.len());
        }
        text.shrink_to_fit();
        ends.shrink_to_fit();
        Self {
            text,
            ends,
            held: None,
        }
    }

    /// 一部の列だけを持つ行にする：足したセルは、行の列の数 `total` のうちの列の範囲 `spans`（左から・重ならない）を順に
    /// 埋めたもの（範囲は列の数で切る・ADR 0033 の決定 1）。足したセルの数が範囲の列の数と合わなければ、全部の列を持つ
    /// 行のまま返す（`spans` の範囲のセルだけを順に足す呼び手の約束）。
    #[must_use]
    pub fn holding(mut self, total: usize, spans: &[Range<usize>]) -> Self {
        let spans: Vec<Range<usize>> = spans
            .iter()
            .map(|span| span.start.min(total)..span.end.min(total))
            .filter(|span| !span.is_empty())
            .collect();
        let count: usize = spans.iter().map(ExactSizeIterator::len).sum();
        // 範囲が列の数の全部なら、全部の列を持つ行のまま。
        if count == self.ends.len() && count != total {
            self.held = Some(HeldColumns::new(total, spans));
        }
        self
    }

    /// 行の列の数（原本のセルの数・持っていない列も数える）。
    #[must_use]
    pub fn len(&self) -> usize {
        self.held
            .as_ref()
            .map_or(self.ends.len(), HeldColumns::total)
    }

    /// セルが 1 つも無いか。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// `index` 番目の列の中身が分かるか：持っている列か、行の列の数の外（「セルが無い」と分かる）。一部の列だけを持つ行の、
    /// 持っていない列だけ `false`（ADR 0033 の決定 1・3）。
    #[must_use]
    pub fn holds(&self, index: usize) -> bool {
        self.position(index).is_some() || index >= self.len()
    }

    /// `index` 番目のセル（セルが無いか、持っていない列なら `None`・どちらかは [`Self::holds`] で分かる）。
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&str> {
        let at = self.position(index)?;
        let end = *self.ends.get(at)?;
        let start = match at.checked_sub(1) {
            Some(before) => *self.ends.get(before)?,
            None => 0,
        };
        self.text.get(start..end)
    }

    /// `index` 番目の列のセルが `ends` の何番目か（持っていなければ `None`）。
    fn position(&self, index: usize) -> Option<usize> {
        let Some(held) = &self.held else {
            return (index < self.ends.len()).then_some(index);
        };
        let mut before = 0_usize;
        for span in held.spans() {
            if span.contains(&index) {
                return Some(before.saturating_add(index.saturating_sub(span.start)));
            }
            before = before.saturating_add(span.len());
        }
        None
    }

    /// 持っているセルを左から順に（全部の列を持つ行なら全部のセル）。
    fn held_cells(&self) -> impl Iterator<Item = &str> {
        (0..self.ends.len()).filter_map(|at| {
            let end = *self.ends.get(at)?;
            let start = at
                .checked_sub(1)
                .map_or(Some(0), |before| self.ends.get(before).copied())?;
            self.text.get(start..end)
        })
    }

    /// 左から順のセル（持っている列だけ）。
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.held_cells()
    }

    /// 一部の列だけを持つ行なら、持っている列の範囲（左から・重ならない）。全部の列を持つ行なら `None`。
    #[must_use]
    pub fn held_spans(&self) -> Option<&[Range<usize>]> {
        self.held.as_ref().map(HeldColumns::spans)
    }

    /// 全部の列を持つ行か（一部の列だけを持つ行は `false`）。
    #[must_use]
    pub fn is_whole(&self) -> bool {
        self.held.is_none()
    }

    /// セルごとの `String` の写し（保存の計画や見出しの元の値など、持ち主の要る所へ渡すとき）。
    #[must_use]
    pub fn to_strings(&self) -> Vec<String> {
        self.iter().map(String::from).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    /// 空のセル・全角も、元の並びのとおりに引ける。
    #[test]
    fn cells_come_back_as_they_were_given() {
        let cells = vec![
            String::from("1"),
            String::new(),
            String::from("東京"),
            String::new(),
        ];
        let row = CellRow::new(&cells);
        let got: Vec<Option<&str>> = (0..5).map(|index| row.get(index)).collect();
        assert_eq!(got, [Some("1"), Some(""), Some("東京"), Some(""), None]);
        assert_eq!(row.len(), 4);
        assert_eq!(row.to_strings(), cells);
        assert_eq!(row.iter().collect::<Vec<_>>(), ["1", "", "東京", ""]);
    }

    /// 1 つずつ足して作った行は、並びから作った行と同じ（場所を先に取っても・手放しても）。
    #[test]
    fn pushing_cells_one_by_one_builds_the_same_row() {
        let cells = ["1", "", "東京", "x,y", ""];
        let mut row = CellRow::with_capacity(2, 1);
        for cell in cells {
            row.push_with(|text| text.push_str(cell));
        }
        row.shrink_to_fit();
        assert_eq!(row, CellRow::new(&cells));
        assert_eq!(CellRow::with_capacity(9, 9), CellRow::empty());
    }

    /// 一部の列だけを持つ行は、持っている列だけを引け、持っていない列は「セルが無い」と取り違えない（ADR 0033 の決定 1）。
    #[test]
    fn a_row_holding_some_columns_knows_which_it_lacks() {
        let mut row = CellRow::empty();
        for cell in ["c2", "c3", "", "c8"] {
            row.push_with(|text| text.push_str(cell));
        }
        let row = row.holding(10, &[2..4, 7..9]);
        assert_eq!(row.len(), 10);
        assert_eq!(
            (row.get(2), row.get(3), row.get(7), row.get(8)),
            (Some("c2"), Some("c3"), Some(""), Some("c8"))
        );
        assert!(row.holds(3) && !row.holds(0) && !row.holds(5) && row.holds(10));
        assert_eq!(row.get(5), None);
        assert_eq!(row.held_spans(), Some(&[2..4, 7..9][..]));
        // 範囲が列の数の外にはみ出しても、列の数で切る。行の全部の列なら、全部の列を持つ行。
        let mut short = CellRow::empty();
        short.push_with(|text| text.push('z'));
        let short = short.holding(1, &[0..5, 9..9]);
        assert!(short.is_whole() && short.holds(4));
        // 表示の写しも、持っている列の範囲を保つ。
        let shown = row.mapped(|cell, text| text.push_str(cell));
        assert_eq!(shown.held_spans(), row.held_spans());
        assert_eq!(shown.get(8), Some("c8"));
    }

    /// セルの無い行と、空のセル 1 つの行は違う。
    #[test]
    fn an_empty_row_differs_from_one_empty_cell() {
        let empty = CellRow::new::<String>(&[]);
        assert!(empty.is_empty() && empty.get(0).is_none());
        let one = CellRow::new(&[""]);
        assert_eq!((one.len(), one.get(0)), (1, Some("")));
        assert_ne!(empty, one);
    }
}
