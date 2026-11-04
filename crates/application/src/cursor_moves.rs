//! カーソルの移動先を決める。

use crate::cell_position::CellPosition;
use crate::cursor_step::CursorStep;
use crate::table_extent::TableExtent;

/// カーソル `cursor` を `step` で動かした先。表の外へは出さない。末尾の行へは、本体の行の数が分かってから。
///
/// `page_rows` は 1 画面に全部入る行の数（1 画面の移動の量）。行の数が分からなければ下へは止めない。
#[must_use]
pub fn step_cursor(
    cursor: CellPosition,
    step: CursorStep,
    page_rows: u64,
    extent: TableExtent,
) -> CellPosition {
    let last_row = extent.rows().map(|rows| rows.saturating_sub(1));
    let last_column = extent.columns().saturating_sub(1);
    let down = |by: u64| {
        let target = cursor.body().saturating_add(by);
        last_row.map_or(target, |last| target.min(last.max(cursor.body())))
    };
    let (body, column) = (cursor.body(), cursor.column());
    let page = page_rows.max(1);
    match step {
        CursorStep::Up => CellPosition::new(body.saturating_sub(1), column),
        CursorStep::Down => CellPosition::new(down(1), column),
        CursorStep::Left => CellPosition::new(body, column.saturating_sub(1)),
        CursorStep::Right => CellPosition::new(body, column.saturating_add(1).min(last_column)),
        CursorStep::PageUp => CellPosition::new(body.saturating_sub(page), column),
        CursorStep::PageDown => CellPosition::new(down(page), column),
        CursorStep::RowStart => CellPosition::new(body, 0),
        CursorStep::RowEnd => CellPosition::new(body, last_column),
        CursorStep::Top => CellPosition::new(0, 0),
        CursorStep::Bottom => CellPosition::new(
            last_row.unwrap_or(body),
            last_row.map_or(column, |_| last_column),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KNOWN: TableExtent = TableExtent::new_const(100, 5);

    fn step(body: u64, column: usize, step: CursorStep, extent: TableExtent) -> (u64, usize) {
        let moved = step_cursor(CellPosition::new(body, column), step, 20, extent);
        (moved.body(), moved.column())
    }

    #[test]
    fn single_steps_stay_inside_the_table() {
        assert_eq!(step(0, 0, CursorStep::Up, KNOWN), (0, 0));
        assert_eq!(step(5, 0, CursorStep::Up, KNOWN), (4, 0));
        assert_eq!(step(99, 0, CursorStep::Down, KNOWN), (99, 0));
        assert_eq!(step(5, 0, CursorStep::Down, KNOWN), (6, 0));
        assert_eq!(step(5, 0, CursorStep::Left, KNOWN), (5, 0));
        assert_eq!(step(5, 4, CursorStep::Right, KNOWN), (5, 4));
        assert_eq!(step(5, 2, CursorStep::Right, KNOWN), (5, 3));
    }

    #[test]
    fn pages_and_ends_jump() {
        assert_eq!(step(5, 2, CursorStep::PageDown, KNOWN), (25, 2));
        assert_eq!(step(90, 2, CursorStep::PageDown, KNOWN), (99, 2));
        assert_eq!(step(5, 2, CursorStep::PageUp, KNOWN), (0, 2));
        assert_eq!(step(5, 2, CursorStep::RowStart, KNOWN), (5, 0));
        assert_eq!(step(5, 2, CursorStep::RowEnd, KNOWN), (5, 4));
        assert_eq!(step(50, 2, CursorStep::Top, KNOWN), (0, 0));
        assert_eq!(step(5, 2, CursorStep::Bottom, KNOWN), (99, 4));
    }

    #[test]
    fn without_the_row_count_down_does_not_stop_and_bottom_stays() {
        let unknown = TableExtent::new(None, 5);
        assert_eq!(step(5, 2, CursorStep::PageDown, unknown), (25, 2));
        assert_eq!(step(5, 2, CursorStep::Bottom, unknown), (5, 2));
        let empty = TableExtent::new(Some(0), 0);
        assert_eq!(step(0, 0, CursorStep::Bottom, empty), (0, 0));
        assert_eq!(step(0, 0, CursorStep::Right, empty), (0, 0));
    }
}
