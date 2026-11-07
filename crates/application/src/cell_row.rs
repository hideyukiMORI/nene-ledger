//! 1 行のセルの文字列を、1 本の文字列と区切りの位置で持つ（GitHub Issue #22・窓の記憶）。

use alloc::string::String;
use alloc::vec::Vec;

/// 1 行のセルの文字列。セルごとに `String` を持つと、1 セルあたり 24 バイトの頭とヒープの割り当て（短い値でも 30 バイトほど）
/// が要る。数万列の表で窓の記憶が GB 単位になった（GitHub Issue #22）。ここでは全部のセルを 1 本の文字列に続けて置き、
/// セルの終わりの位置だけを持つ（1 セルあたり 8 バイト ＋ 字）。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CellRow {
    text: String,
    ends: Vec<usize>,
}

impl CellRow {
    /// セルの文字列の並び `cells` から作る。
    #[must_use]
    pub fn new<S: AsRef<str>>(cells: &[S]) -> Self {
        Self::collect(cells.iter().map(AsRef::as_ref))
    }

    /// セルの文字列を順に出す `cells` から作る。
    #[must_use]
    pub fn collect<'cell, I: Iterator<Item = &'cell str>>(cells: I) -> Self {
        let mut text = String::new();
        let mut ends = Vec::new();
        for cell in cells {
            text.push_str(cell);
            ends.push(text.len());
        }
        text.shrink_to_fit();
        ends.shrink_to_fit();
        Self { text, ends }
    }

    /// セルの数。
    #[must_use]
    pub fn len(&self) -> usize {
        self.ends.len()
    }

    /// セルが 1 つも無いか。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ends.is_empty()
    }

    /// `index` 番目のセル（無ければ `None`）。
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&str> {
        let end = *self.ends.get(index)?;
        let start = match index.checked_sub(1) {
            Some(before) => *self.ends.get(before)?,
            None => 0,
        };
        self.text.get(start..end)
    }

    /// 左から順のセル。
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        (0..self.len()).filter_map(|index| self.get(index))
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
