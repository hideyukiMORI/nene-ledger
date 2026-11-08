//! 縦と横の表示の位置を決める。

/// 縦の表示の先頭を `current` から `target` へ動かすときの、実際の先頭。
///
/// `page_rows` は 1 画面に全部入る行の数。`total` は本体の行の数で、分かっていれば最後の画面が埋まる所で止める
/// （それより下へは動かさないが、今より上へは戻さない）。分からなければ下へは止めない
/// （読んで行が無ければ合成ルートが動かさない）。
#[must_use]
pub fn clamp_first(current: u64, target: u64, page_rows: u64, total: Option<u64>) -> u64 {
    if target <= current {
        return target;
    }
    let last = total.map_or(u64::MAX, |rows| rows.saturating_sub(page_rows.max(1)));
    target.min(last.max(current))
}

/// 行 `row` が見えるようにした縦の表示の先頭。今の先頭 `first` から 1 画面 `page_rows` に入っていれば動かさない。
#[must_use]
pub fn reveal_row(first: u64, row: u64, page_rows: u64) -> u64 {
    let page = page_rows.max(1);
    if row < first {
        row
    } else if row >= first.saturating_add(page) {
        row.saturating_add(1).saturating_sub(page)
    } else {
        first
    }
}

/// 行 `row` が、上の `covered` 行（表に重なった面の下）を除いた所に見えるようにした縦の表示の先頭（GitHub Issue #286）。
/// 面の下に入るなら面のすぐ下へ出るまで戻し、下へ外れるなら [`reveal_row`] と同じ。`covered` は 1 画面より 1 行少なくまで。
/// 表の先頭の行（`row < covered`）は送れないので、面の下に残る。
#[must_use]
pub fn reveal_row_below(first: u64, row: u64, page_rows: u64, covered: u64) -> u64 {
    let page = page_rows.max(1);
    let covered = covered.min(page.saturating_sub(1));
    if row < first.saturating_add(covered) {
        row.saturating_sub(covered)
    } else {
        reveal_row(first, row, page)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn going_down_stops_where_the_last_page_is_full() {
        assert_eq!(clamp_first(0, 3, 20, Some(100)), 3);
        assert_eq!(clamp_first(70, 90, 20, Some(100)), 80);
        assert_eq!(clamp_first(80, 81, 20, Some(100)), 80);
        assert_eq!(clamp_first(80, 0, 20, Some(100)), 0);
    }

    #[test]
    fn without_the_row_count_going_down_does_not_stop() {
        assert_eq!(clamp_first(5, 25, 20, None), 25);
    }

    #[test]
    fn a_short_table_does_not_scroll_and_a_zero_page_counts_as_one() {
        assert_eq!(clamp_first(0, 20, 20, Some(7)), 0);
        assert_eq!(clamp_first(0, 1, 0, None), 1);
        // 末尾より下に居るときは、下へ動かしても戻さない。
        assert_eq!(clamp_first(95, 99, 20, Some(100)), 95);
    }

    #[test]
    fn revealing_a_row_moves_the_least() {
        assert_eq!(reveal_row(10, 15, 20), 10);
        assert_eq!(reveal_row(10, 5, 20), 5);
        assert_eq!(reveal_row(10, 30, 20), 11);
        assert_eq!(reveal_row(10, 29, 20), 10);
        assert_eq!(reveal_row(10, 10, 0), 10);
        assert_eq!(reveal_row(10, 11, 0), 11);
    }

    #[test]
    fn a_row_under_the_cover_comes_out_just_below_it() {
        // 面が上の 3 行を覆う。1 画面 20 行。
        assert_eq!(reveal_row_below(10, 11, 20, 3), 8);
        assert_eq!(reveal_row_below(10, 13, 20, 3), 10);
        assert_eq!(reveal_row_below(10, 40, 20, 3), 21);
        assert_eq!(reveal_row_below(10, 5, 20, 3), 2);
        // 表の先頭の行は送れない（面の下に残る）。
        assert_eq!(reveal_row_below(0, 1, 20, 3), 0);
        // 面が無ければ reveal_row と同じ。面が 1 画面より大きくても 1 行は見せる。
        assert_eq!(reveal_row_below(10, 11, 20, 0), reveal_row(10, 11, 20));
        assert_eq!(reveal_row_below(10, 40, 5, 9), 36);
    }
}
