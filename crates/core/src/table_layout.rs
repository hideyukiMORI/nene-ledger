//! 表の配置（物理画素）。

use alloc::vec::Vec;

use crate::column_widths::{ColumnWidths, MIN_COLUMN_COLUMNS};
use crate::glyph_metrics::GlyphMetrics;
use crate::pixel_point::PixelPoint;
use crate::pixel_rect::PixelRect;
use crate::pixel_size::PixelSize;

/// 罫線の太さ（物理画素）。各セルの右と下に引く。
pub const GRID_LINE: u32 = 1;

/// 表の配置。左に行番号の列、その右に表示の最初の列からのセルの列。行の高さはどの行も同じ（ADR 0002 の決定 2・ARC-011）。
///
/// 列の幅は、字数で決めた `ColumnWidths` から画素にする（字数を決めるのは開いたとき・VIEW-02）。
/// 横に動くと表示の最初の列が変わり、行番号の列はそのまま左に残る。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableLayout {
    gutter_width: u32,
    first_column: usize,
    column_lefts: Vec<u32>,
    column_widths: Vec<u32>,
    row_height: u32,
    padding: u32,
    grid: u32,
}

impl TableLayout {
    /// 列の幅（字数）・表示の最初の列・最後の行の番号（行番号の列の幅を決める）から配置を決める。
    #[must_use]
    pub fn new(
        widths: &ColumnWidths,
        first_column: usize,
        last_number: u64,
        metrics: GlyphMetrics,
    ) -> Self {
        let padding = metrics.half_width();
        let width_of = |columns: u32| {
            columns
                .saturating_mul(metrics.half_width())
                .saturating_add(padding.saturating_mul(2))
                .saturating_add(metrics.px(GRID_LINE))
        };
        let column_widths: Vec<u32> = widths
            .columns()
            .iter()
            .map(|&columns| width_of(columns))
            .collect();
        let gutter_width = width_of(digits(last_number).max(MIN_COLUMN_COLUMNS));
        let column_lefts = column_widths
            .iter()
            .skip(first_column)
            .scan(gutter_width, |left, &width| {
                let here = *left;
                *left = left.saturating_add(width);
                Some(here)
            })
            .collect();
        Self {
            gutter_width,
            first_column,
            column_lefts,
            column_widths,
            row_height: Self::row_height_for(metrics),
            padding,
            grid: metrics.px(GRID_LINE),
        }
    }

    /// 1 行の高さ（1.5 行分 ＋ 罫線）。表の行と、窓の下の状態の行で同じ高さを使う。
    #[must_use]
    pub fn row_height_for(metrics: GlyphMetrics) -> u32 {
        metrics
            .line_height()
            .saturating_add(metrics.line_height() / 2)
            .saturating_add(metrics.px(GRID_LINE))
    }

    /// 表の幅（行番号の列と、表示の最初の列から後ろの全部の列・罫線を含む）。
    #[must_use]
    pub fn table_width(&self) -> u32 {
        self.column_widths
            .iter()
            .skip(self.first_column)
            .fold(self.gutter_width, |total, &width| {
                total.saturating_add(width)
            })
    }

    /// 行番号の列の幅（罫線を含む）。
    #[must_use]
    pub fn gutter_width(&self) -> u32 {
        self.gutter_width
    }

    /// 罫線の太さ（画面の倍率を掛けた `GRID_LINE`）。
    #[must_use]
    pub fn grid(&self) -> u32 {
        self.grid
    }

    /// 1 行の高さ（罫線を含む）。
    #[must_use]
    pub fn row_height(&self) -> u32 {
        self.row_height
    }

    /// セルの中の左右の余白。
    #[must_use]
    pub fn padding(&self) -> u32 {
        self.padding
    }

    /// 列の数（表示の最初の列より前も数える）。
    #[must_use]
    pub fn column_count(&self) -> usize {
        self.column_widths.len()
    }

    /// 表示の最初の列。
    #[must_use]
    pub fn first_column(&self) -> usize {
        self.first_column
    }

    /// `column` 列目の右端（罫線を含む）。表示の最初の列より前か、列が無ければ `None`。
    #[must_use]
    pub fn column_right(&self, column: usize) -> Option<u32> {
        let left = *self
            .column_lefts
            .get(column.checked_sub(self.first_column)?)?;
        Some(left.saturating_add(*self.column_widths.get(column)?))
    }

    /// 横の位置 `x` の列。行番号の列の上か、最後の列より右なら `None`。
    #[must_use]
    pub fn column_at(&self, x: u32) -> Option<usize> {
        (self.first_column..self.column_count()).find(|&column| {
            let right = self.column_right(column).unwrap_or(0);
            let left = right.saturating_sub(self.column_widths.get(column).copied().unwrap_or(0));
            x >= left.max(self.gutter_width) && x < right
        })
    }

    /// 縦の位置 `y` の行（表の上端から何行目か）。
    #[must_use]
    pub fn row_at(&self, y: u32) -> usize {
        usize::try_from(y / self.row_height.max(1)).unwrap_or(usize::MAX)
    }

    /// 高さ `height` の窓に（一部でも）入る行の数。
    #[must_use]
    pub fn rows_in(&self, height: u32) -> usize {
        let rows = height.div_ceil(self.row_height.max(1));
        usize::try_from(rows).unwrap_or(usize::MAX)
    }

    /// `row` 行目の行番号のセル（罫線を除く）。
    #[must_use]
    pub fn gutter_rect(&self, row: usize) -> PixelRect {
        self.rect(0, self.gutter_width, row)
    }

    /// `row` 行目・`column` 列目のセル（罫線を除く）。表示の最初の列より前か、列が無ければ `None`。
    #[must_use]
    pub fn cell_rect(&self, row: usize, column: usize) -> Option<PixelRect> {
        let left = *self
            .column_lefts
            .get(column.checked_sub(self.first_column)?)?;
        let width = *self.column_widths.get(column)?;
        Some(self.rect(left, width, row))
    }

    /// 左端 `left`・罫線を含む幅 `width` の、`row` 行目の矩形（罫線を除く）。
    fn rect(&self, left: u32, width: u32, row: usize) -> PixelRect {
        let row = u32::try_from(row).unwrap_or(u32::MAX);
        let top = row.saturating_mul(self.row_height);
        PixelRect::new(
            PixelPoint::new(left, top),
            PixelSize::new(
                width.saturating_sub(self.grid),
                self.row_height.saturating_sub(self.grid),
            ),
        )
    }
}

/// 10 進の桁数（0 は 1 桁）。
fn digits(number: u64) -> u32 {
    number
        .checked_ilog10()
        .map_or(1, |log| log.saturating_add(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display_scale::DisplayScale;
    use alloc::string::String;
    use alloc::vec;

    fn metrics() -> GlyphMetrics {
        GlyphMetrics::new(8, 16, 12)
    }

    fn layout(cells: &[&[&str]], last_number: u64) -> TableLayout {
        let rows: Vec<Vec<String>> = cells
            .iter()
            .map(|row| row.iter().map(|&cell| String::from(cell)).collect())
            .collect();
        let slices: Vec<&[String]> = rows.iter().map(Vec::as_slice).collect();
        TableLayout::new(&ColumnWidths::fit(&slices), 0, last_number, metrics())
    }

    #[test]
    fn columns_fit_their_content_within_bounds() {
        let long = "x".repeat(100);
        let layout = layout(
            &[&["id", "東京都千代田区", long.as_str()], &["12345", "a"]],
            2,
        );
        assert_eq!(layout.column_count(), 3);
        // 幅 = 字数 × 8 ＋ 余白 8 × 2 ＋ 罫線 1。下限 3 字・上限 40 字。
        let widths: Vec<u32> = (0..3)
            .map(|column| layout.cell_rect(0, column).unwrap().size().width() + GRID_LINE)
            .collect();
        assert_eq!(widths, vec![5 * 8 + 17, 14 * 8 + 17, 40 * 8 + 17]);
        assert_eq!(layout.gutter_width(), 3 * 8 + 17);
        assert_eq!(
            layout.cell_rect(0, 0).unwrap().origin().x(),
            layout.gutter_width()
        );
        assert_eq!(layout.cell_rect(0, 3), None);
        assert_eq!(layout.table_width(), 41 + 57 + 129 + 337);
    }

    #[test]
    fn rows_stack_and_partial_rows_count() {
        let layout = layout(&[&["a"]], 1);
        assert_eq!(layout.row_height(), 16 + 8 + 1);
        assert_eq!(layout.padding(), 8);
        assert_eq!(layout.rows_in(25), 1);
        assert_eq!(layout.rows_in(26), 2);
        assert_eq!(layout.rows_in(0), 0);
        let second = layout.gutter_rect(1);
        assert_eq!((second.origin().y(), second.size().height()), (25, 24));
    }

    #[test]
    fn the_gutter_grows_with_the_record_number() {
        assert_eq!(digits(0), 1);
        assert_eq!(digits(9), 1);
        assert_eq!(digits(10_000), 5);
        let layout = layout(&[], 123_456);
        assert_eq!(layout.gutter_width(), 6 * 8 + 17);
        assert_eq!(layout.column_count(), 0);
    }

    #[test]
    fn scrolling_right_keeps_the_gutter_and_shifts_the_columns() {
        let rows: Vec<Vec<String>> = vec![vec![
            String::from("aaaaa"),
            String::from("bbbbbbbbbb"),
            String::from("c"),
        ]];
        let slices: Vec<&[String]> = rows.iter().map(Vec::as_slice).collect();
        let widths = ColumnWidths::fit(&slices);
        let layout = TableLayout::new(&widths, 1, 1, metrics());
        // 行番号の列 41 のすぐ右に 2 列目（10 字 → 97）、その右に 3 列目（3 字 → 41）。
        assert_eq!(layout.first_column(), 1);
        assert_eq!(layout.cell_rect(0, 0), None);
        assert_eq!(layout.cell_rect(0, 1).unwrap().origin().x(), 41);
        assert_eq!(layout.column_right(1), Some(41 + 97));
        assert_eq!(layout.column_right(2), Some(41 + 97 + 41));
        assert_eq!(layout.column_right(0), None);
        assert_eq!(layout.column_right(3), None);
        assert_eq!(layout.table_width(), 41 + 97 + 41);
    }

    #[test]
    fn positions_map_to_columns_and_rows() {
        let layout = layout(&[&["aaaaa", "b"]], 1);
        // 行番号の列 [0, 41)・1 列目 [41, 98)・2 列目 [98, 139)。
        assert_eq!(layout.column_at(40), None);
        assert_eq!(layout.column_at(41), Some(0));
        assert_eq!(layout.column_at(97), Some(0));
        assert_eq!(layout.column_at(98), Some(1));
        assert_eq!(layout.column_at(139), None);
        assert_eq!(layout.row_at(0), 0);
        assert_eq!(layout.row_at(24), 0);
        assert_eq!(layout.row_at(25), 1);
    }

    #[test]
    fn the_grid_line_grows_with_the_display_scale() {
        let metrics = GlyphMetrics::new(12, 24, 18).with_scale(DisplayScale::of_percent(150));
        // 罫線 1 は 2。1 行は字の 1.5 倍 36 ＋ 罫線 2。
        assert_eq!(TableLayout::row_height_for(metrics), 38);
        let rows = [vec![String::from("abc")]];
        let slices: Vec<&[String]> = rows.iter().map(Vec::as_slice).collect();
        let scaled = TableLayout::new(&ColumnWidths::fit(&slices), 0, 9, metrics);
        assert_eq!(scaled.grid(), 2);
        assert_eq!(layout(&[&["abc"]], 9).grid(), 1);
    }
}
