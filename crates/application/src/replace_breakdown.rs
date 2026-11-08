//! 全部の置換の内訳（何が何に変わるか・GitHub Issue #289）。

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use nene_ledger_core::cell_change::CellChange;
use nene_ledger_core::display_width::display_columns;

use crate::status_line::group_digits;

/// 確かめの行で、値を切る桁（全角 = 2）。いちばん長い行（組・100 万セル・残りの組の数）が案内の幅（64 桁）に収まる長さ。
const VALUE_COLUMNS: u32 = 7;

/// 「前 → 後」の組のまとまり（セル全体が変わる組か、一部だけ変わる組か）：いちばん多い組・組の数・セルの数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PairGroup {
    top: (String, String, usize),
    kinds: usize,
    cells: usize,
}

impl PairGroup {
    /// いちばん多い組（前・後・セルの数）。同じ数なら前の値・後の値の順で先のもの。
    #[must_use]
    pub const fn top(&self) -> &(String, String, usize) {
        &self.top
    }

    /// 組の数。
    #[must_use]
    pub const fn kinds(&self) -> usize {
        self.kinds
    }

    /// セルの数。
    #[must_use]
    pub const fn cells(&self) -> usize {
        self.cells
    }
}

/// 全部の置換で、セルの値が「前 → 後」にどう変わるか。**セル全体が変わる組**（後の値が置き換える文字そのもの）と、
/// **一部だけ変わる組**（部分一致で、当たった所がセルの全体でない）に分ける。980 → 990 の部分一致で 1980 → 1990 が
/// 混ざる事故は、一部だけの組として必ず見える（多い順に並べると「ほか」に埋もれた・GitHub Issue #289・#299）。
/// 答えは書き換えの並びに依らない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaceBreakdown {
    whole: Option<PairGroup>,
    partial: Option<PairGroup>,
}

impl ReplaceBreakdown {
    /// 書き換え `changes` の内訳。後の値が置き換える文字 `replacement` と違うセルを一部だけの組にする。正規表現（`literal`
    /// でない）は `$1` などで後の値が変わるので、すべてセル全体の組に数える。
    #[must_use]
    pub fn of(changes: &[(CellChange, String)], replacement: &str, literal: bool) -> Self {
        let mut whole: BTreeMap<(&str, &str), usize> = BTreeMap::new();
        let mut partial: BTreeMap<(&str, &str), usize> = BTreeMap::new();
        for (change, _) in changes {
            let group = if literal && change.after() != replacement {
                &mut partial
            } else {
                &mut whole
            };
            let count = group.entry((change.before(), change.after())).or_insert(0);
            *count = count.saturating_add(1);
        }
        Self {
            whole: group_of(&whole),
            partial: group_of(&partial),
        }
    }

    /// セル全体が変わる組。
    #[must_use]
    pub const fn whole(&self) -> Option<&PairGroup> {
        self.whole.as_ref()
    }

    /// 一部だけ変わる組。
    #[must_use]
    pub const fn partial(&self) -> Option<&PairGroup> {
        self.partial.as_ref()
    }

    /// 確かめの行（多くて 2 行・注意は `true`）。セル全体の組（いちばん多い組と残りの組の数）、一部だけの組（セルの数と
    /// いちばん多い組・強調）、置き換える文字 `replacement` が空なら消す注意（強調）。3 つあれば、注意を残してセル全体の組を
    /// 落とす。
    #[must_use]
    pub fn lines(&self, replacement: &str) -> Vec<(String, bool)> {
        let whole = self.whole.as_ref().map(|group| {
            let rest = group.kinds.saturating_sub(1);
            let others = if rest > 0 {
                format!(" · ほか {} 通り", digits(rest))
            } else {
                String::new()
            };
            (format!("{}{others}", pair(&group.top)), false)
        });
        let partial = self.partial.as_ref().map(|group| {
            (
                format!(
                    "一部だけ変わる {} セル: “{}”→“{}” など",
                    digits(group.cells),
                    clipped(&group.top.0),
                    clipped(&group.top.1)
                ),
                true,
            )
        });
        let erase = replacement
            .is_empty()
            .then(|| (String::from("置換後が空なので、当たった所を消します"), true));
        let mut lines: Vec<(String, bool)> =
            [whole, partial, erase].into_iter().flatten().collect();
        if lines.len() > 2 {
            lines.remove(0);
        }
        lines
    }
}

/// まとまりの組の数え `counts` から、いちばん多い組・組の数・セルの数。空なら `None`。
fn group_of(counts: &BTreeMap<(&str, &str), usize>) -> Option<PairGroup> {
    // 数の多い順。同じ数なら BTreeMap の順（前の値・後の値）で先のもの（`max_by_key` は同じなら後を返すので逆から）。
    let (&(before, after), &count) = counts.iter().rev().max_by_key(|&(_, &count)| count)?;
    Some(PairGroup {
        top: (String::from(before), String::from(after), count),
        kinds: counts.len(),
        cells: counts
            .values()
            .fold(0_usize, |sum, &cells| sum.saturating_add(cells)),
    })
}

/// 組 1 つの短い形（値は切る）。
fn pair((before, after, count): &(String, String, usize)) -> String {
    format!(
        "“{}”→“{}” {} セル",
        clipped(before),
        clipped(after),
        digits(*count)
    )
}

fn digits(number: usize) -> String {
    group_digits(u64::try_from(number).unwrap_or(u64::MAX))
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

    /// 980 → 990 を部分一致で置き換えると 1980 も 29800 も変わる。一部だけの組は、少なくても必ず強調の行に出る
    /// （#285 の 2 巡目：多い順の 2 組では 1980→1990 が「ほか」に埋もれた）。
    #[test]
    fn partial_pairs_are_always_shown_apart_from_whole_cell_pairs() {
        let mut changes = vec![change(1, "1980", "1990")];
        changes.extend((2..5).map(|row| change(row, "980", "990")));
        changes.extend((5..7).map(|row| change(row, "29800", "29900")));
        let breakdown = ReplaceBreakdown::of(&changes, "990", true);
        let whole = breakdown.whole().unwrap();
        assert_eq!((whole.kinds(), whole.cells()), (1, 3));
        let partial = breakdown.partial().unwrap();
        assert_eq!((partial.kinds(), partial.cells()), (2, 3));
        assert_eq!(
            partial.top(),
            &(String::from("29800"), String::from("29900"), 2)
        );
        assert_eq!(
            breakdown.lines("990"),
            vec![
                (String::from("“980”→“990” 3 セル"), false),
                (
                    String::from("一部だけ変わる 3 セル: “29800”→“29900” など"),
                    true
                ),
            ]
        );
        // 正規表現は一部だけを見ない（後の値が $1 などで変わる）。
        assert!(
            ReplaceBreakdown::of(&changes, "990", false)
                .partial()
                .is_none()
        );
    }

    /// 置き換える文字が空なら消すと言い、3 行になるならセル全体の組を落とす。同じ数の組は前の値の順で先のもの。答えは
    /// 書き換えの並びに依らない。
    #[test]
    fn an_empty_replacement_warns_and_ties_are_stable() {
        let changes = vec![
            change(1, "b", ""),
            change(2, "a", ""),
            change(3, "xa", "x"),
            change(4, "c", ""),
        ];
        let breakdown = ReplaceBreakdown::of(&changes, "", true);
        assert_eq!(
            breakdown.whole().unwrap().top(),
            &(String::from("a"), String::new(), 1)
        );
        assert_eq!(breakdown.whole().unwrap().kinds(), 3);
        let lines = breakdown.lines("");
        assert_eq!(lines.len(), 2);
        assert!(lines[0].0.starts_with("一部だけ変わる 1 セル"));
        assert_eq!(
            lines[1],
            (String::from("置換後が空なので、当たった所を消します"), true)
        );
        let mut reversed = changes;
        reversed.reverse();
        assert_eq!(ReplaceBreakdown::of(&reversed, "", true), breakdown);
        // 一部だけの組が無ければ、セル全体の組と消す注意。
        let simple = ReplaceBreakdown::of(&[change(1, "a", "")], "", true).lines("");
        assert_eq!(simple[0], (String::from("“a”→“” 1 セル"), false));
    }

    /// 内訳の行の長さの上限（表示の桁）。案内の上限（`palette_painter` の 64・Xvfb の画面写しで測った）と同じ。
    const LINE_COLUMNS: u32 = 64;

    /// 長い値と改行は切って 1 行に収め、どの組でも内訳の行の上限に収まる。
    #[test]
    fn long_values_are_clipped_to_fit_the_guide() {
        let long = "とても長いセルの値がここに入ります";
        let changes = vec![
            change(1, long, "x\ny"),
            change(2, "1,000,000,000", long),
            change(3, "1,000,000,000", long),
        ];
        // 3 つとも一部だけ変わる（後の値が y でない）。
        let lines = ReplaceBreakdown::of(&changes, "y", true).lines("y");
        assert_eq!(
            lines[0].0,
            "一部だけ変わる 3 セル: “1,000,…”→“とても…” など"
        );
        for (line, _) in lines {
            assert!(display_columns(&line) <= LINE_COLUMNS, "{line}");
        }
        // いちばん長い行（残りの組の数とセルの数が大きいとき・空の置換）も収まる。
        let many: Vec<(CellChange, String)> = (0..2_000)
            .flat_map(|row| {
                [
                    change(row, &format!("{long}{row}"), ""),
                    change(row + 10_000, &format!("1{long}"), "1"),
                ]
            })
            .collect();
        for (line, _) in ReplaceBreakdown::of(&many, "", true).lines("") {
            assert!(display_columns(&line) <= LINE_COLUMNS, "{line}");
        }
        let wide: Vec<(CellChange, String)> = (0..2_000)
            .map(|row| change(row, &format!("{long}{row}"), "y"))
            .collect();
        for (line, _) in ReplaceBreakdown::of(&wide, "y", true).lines("y") {
            assert!(display_columns(&line) <= LINE_COLUMNS, "{line}");
        }
    }
}
