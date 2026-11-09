//! 全部の置換の内訳（何が何に変わるか・GitHub Issue #289）。

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use nene_ledger_core::cell_change::CellChange;
use nene_ledger_core::display_width::display_columns;

use crate::status_line::group_digits;

/// 確かめの行に並べる組の数。
const SHOWN: usize = 2;
/// 確かめの行で、値を切る桁（全角 = 2）。2 組と件数が案内の幅（64 桁）に収まる長さ。
const VALUE_COLUMNS: u32 = 8;

/// 全部の置換で、セルの値が「前 → 後」にどう変わるかの組を、セルの数の多い順に数えたもの。同じ数なら前の値の順
/// （答えは書き換えの並びに依らない）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaceBreakdown {
    top: Vec<(String, String, usize)>,
    kinds: usize,
    partial: bool,
}

impl ReplaceBreakdown {
    /// 書き換え `changes` の内訳。`partial` は、値の一部だけが当たって変わるセルがあるか（当たった所がセルの全体でない。
    /// 置き換えた後の値が置き換える文字 `replacement` と違う）。正規表現（`literal` でない）は `$1` などで後の値が変わるので
    /// 見ない。
    #[must_use]
    pub fn of(changes: &[(CellChange, String)], replacement: &str, literal: bool) -> Self {
        let mut counts: BTreeMap<(&str, &str), usize> = BTreeMap::new();
        let mut partial = false;
        for (change, _) in changes {
            let count = counts.entry((change.before(), change.after())).or_insert(0);
            *count = count.saturating_add(1);
            partial |= literal && change.after() != replacement;
        }
        let kinds = counts.len();
        let mut ordered: Vec<((&str, &str), usize)> = counts.into_iter().collect();
        // 数の多い順。同じ数なら BTreeMap の順（前の値・後の値）のまま（安定な並べ替え）。
        ordered.sort_by_key(|&(_, count)| core::cmp::Reverse(count));
        let top = ordered
            .into_iter()
            .take(SHOWN)
            .map(|((before, after), count)| (String::from(before), String::from(after), count))
            .collect();
        Self {
            top,
            kinds,
            partial,
        }
    }

    /// 数の多い組（前・後・セルの数）。多くて 2 つ。
    #[must_use]
    pub fn top(&self) -> &[(String, String, usize)] {
        &self.top
    }

    /// 組の数（前と後の値の違う組み合わせの数）。
    #[must_use]
    pub const fn kinds(&self) -> usize {
        self.kinds
    }

    /// 値の一部だけが変わるセルがあるか。
    #[must_use]
    pub const fn partial(&self) -> bool {
        self.partial
    }

    /// 確かめの行（1 行目は組・2 行目は残りの組の数と注意）。`whole_cell` はセル全体の一致か、`replacement` は置き換える文字
    /// （空なら当たった所を消す）。注意は強めの行（`true`）。
    #[must_use]
    pub fn lines(&self, whole_cell: bool, replacement: &str) -> Vec<(String, bool)> {
        let pairs: Vec<String> = self
            .top
            .iter()
            .map(|(before, after, count)| {
                format!(
                    "“{}”→“{}” {} セル",
                    clipped(before),
                    clipped(after),
                    group_digits(u64::try_from(*count).unwrap_or(u64::MAX))
                )
            })
            .collect();
        let mut lines = Vec::new();
        if !pairs.is_empty() {
            lines.push((pairs.join(" · "), false));
        }
        let rest = self.kinds.saturating_sub(self.top.len());
        let others = (rest > 0).then(|| {
            format!(
                "ほか {} 通り",
                group_digits(u64::try_from(rest).unwrap_or(u64::MAX))
            )
        });
        let warning = if replacement.is_empty() {
            Some(String::from("置換後が空なので、当たった所を消します"))
        } else if self.partial && !whole_cell {
            Some(String::from("一部だけ変わるセルあり · Alt+W で全体一致"))
        } else {
            None
        };
        match (others, warning) {
            (Some(others), Some(warning)) => lines.push((format!("{others} · {warning}"), true)),
            (None, Some(warning)) => lines.push((warning, true)),
            (Some(others), None) => lines.push((others, false)),
            (None, None) => {}
        }
        lines
    }
}

/// 値を `VALUE_COLUMNS` 桁で切る（越えれば `…`）。改行とタブは見える印にしない（1 行に収めるため空白に）。
fn clipped(value: &str) -> String {
    let flat: String = value
        .chars()
        .map(|character| {
            if character.is_control() {
                ' '
            } else {
                character
            }
        })
        .collect();
    if display_columns(&flat) <= VALUE_COLUMNS {
        return flat;
    }
    let mut kept = String::new();
    for character in flat.chars() {
        let mut one = [0_u8; 4];
        let next =
            display_columns(&kept).saturating_add(display_columns(character.encode_utf8(&mut one)));
        if next > VALUE_COLUMNS.saturating_sub(1) {
            break;
        }
        kept.push(character);
    }
    kept.push('…');
    kept
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use nene_ledger_core::column_id::ColumnId;
    use nene_ledger_core::row_id::RowId;

    fn change(row: u64, before: &str, after: &str) -> (CellChange, String) {
        (
            CellChange::new(
                RowId::Source(row),
                ColumnId::Source(0),
                String::from(before),
                String::from(after),
            ),
            String::from(before),
        )
    }

    /// 980 → 990 を部分一致で置き換えると 1980 も 1990 になる。それが組と注意で見える（#285 の f4-33count）。
    #[test]
    fn a_partial_match_shows_the_unintended_pair_and_a_warning() {
        let mut changes = vec![change(1, "1980", "1990")];
        changes.extend((2..5).map(|row| change(row, "980", "990")));
        let breakdown = ReplaceBreakdown::of(&changes, "990", true);
        assert_eq!(
            breakdown.top(),
            &[
                (String::from("980"), String::from("990"), 3),
                (String::from("1980"), String::from("1990"), 1)
            ]
        );
        assert!(breakdown.partial());
        assert_eq!(
            breakdown.lines(false, "990"),
            vec![
                (
                    String::from("“980”→“990” 3 セル · “1980”→“1990” 1 セル"),
                    false
                ),
                (
                    String::from("一部だけ変わるセルあり · Alt+W で全体一致"),
                    true
                ),
            ]
        );
        // セル全体の一致なら注意は出さない（部分の印は数えたときのまま）。正規表現は部分を見ない。
        assert_eq!(breakdown.lines(true, "990").len(), 1);
        assert!(!ReplaceBreakdown::of(&changes, "990", false).partial());
    }

    /// 置き換える文字が空なら、消すと言う。残りの組の数も添える。答えは書き換えの並びに依らない。
    #[test]
    fn an_empty_replacement_warns_and_the_rest_are_counted() {
        let changes = vec![
            change(1, "a", ""),
            change(2, "b", ""),
            change(3, "c", ""),
            change(4, "a", ""),
        ];
        let breakdown = ReplaceBreakdown::of(&changes, "", true);
        assert_eq!(breakdown.kinds(), 3);
        assert!(!breakdown.partial());
        assert_eq!(
            breakdown.lines(true, "")[1],
            (
                String::from("ほか 1 通り · 置換後が空なので、当たった所を消します"),
                true
            )
        );
        let mut reversed = changes;
        reversed.reverse();
        assert_eq!(ReplaceBreakdown::of(&reversed, "", true), breakdown);
    }

    /// 内訳の行の長さの上限（表示の桁）。Xvfb の画面写しで、64 桁の注意（全角の括弧を含む）がいちばん狭いパレットで
    /// 切れた（2026-11-09）ので、案内の上限（`palette_painter` の 64）より狭くする。
    const LINE_COLUMNS: u32 = 60;

    /// 長い値と改行は切って 1 行に収め、どの組でも内訳の行の上限に収まる。
    #[test]
    fn long_values_are_clipped_to_fit_the_guide() {
        let long = "とても長いセルの値がここに入ります";
        let changes = vec![
            change(1, long, "x\ny"),
            change(2, "1,000,000,000", long),
            change(3, "1,000,000,000", long),
        ];
        let lines = ReplaceBreakdown::of(&changes, "y", true).lines(false, "y");
        assert_eq!(
            lines[0].0,
            "“1,000,0…”→“とても…” 2 セル · “とても…”→“x y” 1 セル"
        );
        for (line, _) in lines {
            assert!(display_columns(&line) <= LINE_COLUMNS, "{line}");
        }
        // いちばん長い注意（残りの組の数が大きいとき）も収まる。
        let many: Vec<(CellChange, String)> = (0..2_000)
            .map(|row| change(row, &format!("1{row}"), "y1"))
            .collect();
        for (line, _) in ReplaceBreakdown::of(&many, "y", true).lines(false, "y") {
            assert!(display_columns(&line) <= LINE_COLUMNS, "{line}");
        }
    }
}
