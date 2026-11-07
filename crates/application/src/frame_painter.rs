//! 表（固定した見出しの行と本体の見えている行）を描画命令に変える。

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use nene_ledger_core::cell_edits::CellEdits;
use nene_ledger_core::cell_value::CellValue;
use nene_ledger_core::color::Color;
use nene_ledger_core::column_id::ColumnId;
use nene_ledger_core::display_width::char_columns;
use nene_ledger_core::draw_command::DrawCommand;
use nene_ledger_core::glyph_metrics::GlyphMetrics;
use nene_ledger_core::palette::Palette;
use nene_ledger_core::pixel_point::PixelPoint;
use nene_ledger_core::pixel_rect::PixelRect;
use nene_ledger_core::pixel_size::PixelSize;
use nene_ledger_core::row_id::RowId;
use nene_ledger_core::table_layout::TableLayout;
use nene_ledger_core::text_role::TextRole;

use crate::cell_display::{fitted, single_line};
use crate::cell_mark::CellMark;
use crate::editing_cell::EditingCell;
use crate::ranges::Ranges;
use crate::row_style::RowStyle;
use crate::selection::Selection;
use crate::table_view::TableView;
use crate::view_row::ViewRow;

/// 1 枚の画面の描画命令を、描く順に積む。
#[derive(Debug)]
pub(crate) struct FramePainter<'view> {
    layout: &'view TableLayout,
    metrics: GlyphMetrics,
    palette: Palette,
    size: PixelSize,
    selection: Option<Ranges<'view>>,
    cut: Option<Selection>,
    order: &'view [usize],
    column_ids: &'view [ColumnId],
    edits: Option<&'view CellEdits>,
    editing: Option<&'view EditingCell>,
    commands: Vec<DrawCommand>,
    overlays: Vec<DrawCommand>,
}

/// カーソルの枠の太さ（画素・100% のとき）。
const CURSOR_LINE: u32 = 2;
/// 選んだ範囲の外周の枠の太さ（画素・100% のとき）。
const SELECTION_LINE: u32 = 1;
/// 切り取りの点線の枠の太さ（画素・100% のとき・ADR 0029 の決定 5）。
const CUT_LINE: u32 = 2;
/// 切り取りの点線の 1 つの長さと間（画素・100% のとき）。
const CUT_DASH: u32 = 4;
/// 左に固定した列と横に動く列の境の線の太さ（画素・100% のとき・VIEW-01）。
const FROZEN_LINE: u32 = 2;
/// 編集の欄のカーソルの棒の太さ（論理の画素）。
const CARET_WIDTH: u32 = 2;
/// 確定前の文字列の下線の太さ（論理の画素）。
const UNDERLINE: u32 = 1;

impl<'view> FramePainter<'view> {
    /// 配置・字の寸法・色・窓の大きさから始める。
    pub(crate) fn new(
        layout: &'view TableLayout,
        metrics: GlyphMetrics,
        palette: Palette,
        size: PixelSize,
    ) -> Self {
        Self {
            layout,
            metrics,
            palette,
            size,
            selection: None,
            cut: None,
            order: &[],
            column_ids: &[],
            edits: None,
            editing: None,
            commands: Vec::new(),
            overlays: Vec::new(),
        }
    }

    /// 1 つの範囲の選択を描く（選んだセルの背景とカーソルの枠）。
    #[cfg(test)]
    pub(crate) fn with_selection(self, selection: Selection) -> Self {
        self.with_ranges(Ranges::new(selection, &[]))
    }

    /// 列の表示の順 `order`（表示の位置 → 元の列）で描く（入れ替えは表示だけ・Issue #49）。
    /// 切り取った範囲 `cut`（点線の枠で示す・無ければ `None`）。
    pub(crate) fn with_cut(self, cut: Option<Selection>) -> Self {
        Self { cut, ..self }
    }

    pub(crate) fn with_order(self, order: &'view [usize]) -> Self {
        Self { order, ..self }
    }

    /// 表の列の同一性（表の列の順・列を足し消しした表・ADR 0027）。書き換えをこの同一性で引く。空なら原本の列の順。
    pub(crate) fn with_column_ids(self, column_ids: &'view [ColumnId]) -> Self {
        Self { column_ids, ..self }
    }

    /// 選んだ範囲の並びを描く（範囲ごとの背景と外周の枠・今の範囲のカーソルの枠）。
    /// 書き換えたセル（ADR 0024 の層 2）を、原本の値の代わりに描く。
    pub(crate) fn with_edits(self, edits: &'view CellEdits) -> Self {
        Self {
            edits: Some(edits),
            ..self
        }
    }

    /// カーソルのセルを、編集の欄として描く（ADR 0026）。
    pub(crate) fn with_editing(self, editing: Option<&'view EditingCell>) -> Self {
        Self { editing, ..self }
    }

    pub(crate) fn with_ranges(self, ranges: Ranges<'view>) -> Self {
        Self {
            selection: Some(ranges),
            ..self
        }
    }

    /// 背景と、上に固定した見出しの行（無ければ列の番号の行 `numbers`）と、その下に本体の見えている行の命令。
    pub(crate) fn paint(
        mut self,
        header: Option<&ViewRow>,
        numbers: Option<&ViewRow>,
        view: &TableView,
    ) -> Vec<DrawCommand> {
        self.fill(
            PixelRect::new(PixelPoint::new(0, 0), self.size),
            self.palette.background(),
        );
        let visible = self.layout.rows_in(self.size.height());
        let fixed = header
            .map(|row| (row, RowStyle::Header))
            .or_else(|| numbers.map(|row| (row, RowStyle::Numbers)));
        let styled = fixed.into_iter().chain(
            view.rows()
                .iter()
                .zip(view.first_body()..)
                .map(|(row, body)| (row, RowStyle::Body(body))),
        );
        let mut painted = 0_usize;
        for (slot, (row, style)) in styled.take(visible).enumerate() {
            self.paint_row(slot, row, style);
            painted = slot.saturating_add(1);
        }
        self.frozen_edge(painted);
        // 選んだ範囲の外周とカーソルの枠は、隣の行の塗りに隠されないよう最後に重ねる。
        let mut commands = self.commands;
        commands.append(&mut self.overlays);
        commands
    }

    /// 左に固定した列の右の境に、描いた `rows` 行ぶんの縦の線（固定が無ければ何も描かない・VIEW-01）。
    fn frozen_edge(&mut self, rows: usize) {
        if self.layout.frozen() == 0 {
            return;
        }
        let line = self.metrics.px(FROZEN_LINE);
        let right = self
            .layout
            .gutter_width()
            .saturating_add(self.layout.frozen_width());
        let height = self
            .layout
            .row_height()
            .saturating_mul(u32::try_from(rows).unwrap_or(u32::MAX))
            .min(self.size.height());
        self.fill(
            PixelRect::new(
                PixelPoint::new(right.saturating_sub(line), 0),
                PixelSize::new(line, height),
            ),
            self.palette.border(),
        );
    }

    /// 1 行。罫線の色で行の帯（表の幅まで）を塗り、その上にセルを塗って、残った 1 画素を罫線にする。
    fn paint_row(&mut self, index: usize, row: &ViewRow, style: RowStyle) {
        let gutter = self.layout.gutter_rect(index);
        let band = PixelRect::new(
            PixelPoint::new(0, gutter.origin().y()),
            PixelSize::new(
                self.layout.table_width().min(self.size.width()),
                self.layout.row_height(),
            ),
        );
        self.fill(band, self.palette.grid());
        // 選んだ行の行番号は選択の色にする（色以外の位置の手がかり）。
        let (gutter_fill, gutter_ink) = match style {
            RowStyle::Body(body) if self.covers_row(body) => {
                (self.palette.selection(), self.palette.text())
            }
            RowStyle::Header | RowStyle::Numbers | RowStyle::Body(_) => {
                (self.palette.gutter(), self.palette.gutter_text())
            }
        };
        // 足した行の番号は強調の色にする（ADR 0027 の決定 1）。
        let gutter_ink = match row.row() {
            RowId::Added(_) => self.palette.cursor(),
            RowId::Source(_) => gutter_ink,
        };
        self.fill(gutter, gutter_fill);
        // 行番号はファイルの中のレコードの番号で、1 始まりで見せる（レコードの番号は 0 始まり）。
        if style != RowStyle::Numbers {
            let shown = row.line().saturating_add(1).to_string();
            self.text(gutter, &shown, gutter_ink);
        }
        let mut cursor = None;
        for column in self.layout.visible_columns() {
            let Some(rect) = self.layout.cell_rect(index, column) else {
                break;
            };
            if rect.origin().x() >= self.size.width() {
                break;
            }
            let mark = self.mark(style, column);
            let source = self.order.get(column).copied().unwrap_or(column);
            self.paint_shown_cell(rect, (row, style, source), mark);
            if let RowStyle::Body(body) = style {
                self.edges(rect, body, column);
                self.cut_edges(rect, body, column);
            }
            if mark == CellMark::Cursor {
                cursor = Some(rect);
            }
        }
        if let Some(rect) = cursor {
            self.outline(rect);
        }
    }

    /// 行 `row`（`style`）の元の列 `source` のセル。編集中のカーソルのセルは欄として、書き換えたセルは書き換えた値で描く。
    fn paint_shown_cell(
        &mut self,
        rect: PixelRect,
        (row, style, source): (&ViewRow, RowStyle, usize),
        mark: CellMark,
    ) {
        if let (CellMark::Cursor, Some(editing)) = (mark, self.editing) {
            self.paint_editing(rect, editing);
            return;
        }
        match self.edited(style, row.row(), source) {
            Some(CellValue::Text(value)) => {
                let shown = single_line(value);
                self.paint_cell(rect, Some(&shown), mark);
            }
            Some(CellValue::Absent) => self.paint_cell(rect, None, mark),
            None => self.paint_cell(rect, row.cells().get(source), mark),
        }
    }

    /// 本体の行のセル（行 `row` の元の列 `source`）の書き換え。見出しの行は書き換えない（EDIT-04 はまだ）。
    fn edited(&self, style: RowStyle, row: RowId, source: usize) -> Option<&'view CellValue> {
        match style {
            RowStyle::Body(_) => {
                let column = self
                    .column_ids
                    .get(source)
                    .copied()
                    .unwrap_or(ColumnId::Source(source));
                self.edits?.get(row, column)
            }
            RowStyle::Header | RowStyle::Numbers => None,
        }
    }

    /// 編集中のセル。地の色で塗り、欄の文字列とカーソルの棒と、確定前の文字列の下線を描く。
    /// 欄より長い文字列は、カーソルが右端に見えるまで頭の字を飛ばして描く（`…` は付けない）。
    fn paint_editing(&mut self, rect: PixelRect, editing: &EditingCell) {
        self.fill(rect, self.palette.background());
        let half = self.metrics.half_width().max(1);
        let room = rect
            .size()
            .width()
            .saturating_sub(self.layout.padding().saturating_mul(2))
            / half;
        let (shown, skipped) = scrolled_to_caret(editing.text(), editing.caret(), room);
        if !shown.is_empty() {
            self.text(rect, &shown, self.palette.text());
        }
        let caret = editing.caret().saturating_sub(skipped);
        let left = rect.origin().x().saturating_add(self.layout.padding());
        let right = rect.origin().x().saturating_add(rect.size().width());
        let spare = rect
            .size()
            .height()
            .saturating_sub(self.metrics.line_height());
        let top = rect.origin().y().saturating_add(spare / 2);
        let caret_width = self.metrics.px(CARET_WIDTH);
        let at = |columns: u32| {
            left.saturating_add(columns.saturating_mul(half))
                .min(right.saturating_sub(caret_width))
        };
        if let Some((start, end)) = editing.underline() {
            let line = self.metrics.px(UNDERLINE);
            let (from, to) = (
                at(start.saturating_sub(skipped)),
                at(end.saturating_sub(skipped)),
            );
            self.fill(
                PixelRect::new(
                    PixelPoint::new(
                        from,
                        top.saturating_add(self.metrics.line_height())
                            .saturating_sub(line),
                    ),
                    PixelSize::new(to.saturating_sub(from), line),
                ),
                self.palette.text(),
            );
        }
        self.overlays.push(DrawCommand::Fill {
            rect: PixelRect::new(
                PixelPoint::new(at(caret), top),
                PixelSize::new(caret_width, self.metrics.line_height()),
            ),
            color: self.palette.cursor(),
        });
    }

    /// 選択が本体の `body` 番目の行を覆うか。
    fn covers_row(&self, body: u64) -> bool {
        self.selection.is_some_and(|ranges| ranges.covers_row(body))
    }

    /// 選択が `column` 列目を覆うか。
    fn covers_column(&self, column: usize) -> bool {
        self.selection
            .is_some_and(|ranges| ranges.covers_column(column))
    }

    /// 選んだ範囲の外周に当たるセルの辺に、`SELECTION_LINE` の線を後から重ねる（見えている部分だけ）。
    fn edges(&mut self, rect: PixelRect, body: u64, column: usize) {
        let Some(ranges) = self.selection else {
            return;
        };
        for selection in ranges.all().filter(|range| range.contains(body, column)) {
            self.range_edges(rect, (body, column), selection);
        }
    }

    /// 範囲 `selection` の外周に当たる、セル `at`（本体の行・列）の辺。
    fn range_edges(&mut self, rect: PixelRect, at: (u64, usize), selection: Selection) {
        let (body, column) = at;
        let (top_left, bottom_right) = (selection.top_left(), selection.bottom_right());
        let (left, top) = (rect.origin().x(), rect.origin().y());
        let (width, height) = (rect.size().width(), rect.size().height());
        let line = self.metrics.px(SELECTION_LINE).min(width).min(height);
        let right = left.saturating_add(width).saturating_sub(line);
        let bottom = top.saturating_add(height).saturating_sub(line);
        if body == top_left.body() {
            self.overlay(PixelRect::new(
                PixelPoint::new(left, top),
                PixelSize::new(width, line),
            ));
        }
        if body == bottom_right.body() {
            self.overlay(PixelRect::new(
                PixelPoint::new(left, bottom),
                PixelSize::new(width, line),
            ));
        }
        if column == top_left.column() {
            self.overlay(PixelRect::new(
                PixelPoint::new(left, top),
                PixelSize::new(line, height),
            ));
        }
        if column == bottom_right.column() {
            self.overlay(PixelRect::new(
                PixelPoint::new(right, top),
                PixelSize::new(line, height),
            ));
        }
    }

    /// 切り取った範囲の外周に当たる、セル（本体の `body` 行・`column` 列）の辺を点線で描く。
    fn cut_edges(&mut self, rect: PixelRect, body: u64, column: usize) {
        let Some(cut) = self.cut.filter(|cut| cut.contains(body, column)) else {
            return;
        };
        let (top_left, bottom_right) = (cut.top_left(), cut.bottom_right());
        let (left, top) = (rect.origin().x(), rect.origin().y());
        let (width, height) = (rect.size().width(), rect.size().height());
        let line = self.metrics.px(CUT_LINE).min(width).min(height);
        let right = left.saturating_add(width).saturating_sub(line);
        let bottom = top.saturating_add(height).saturating_sub(line);
        if body == top_left.body() {
            self.dashed(PixelPoint::new(left, top), (width, line), true);
        }
        if body == bottom_right.body() {
            self.dashed(PixelPoint::new(left, bottom), (width, line), true);
        }
        if column == top_left.column() {
            self.dashed(PixelPoint::new(left, top), (line, height), false);
        }
        if column == bottom_right.column() {
            self.dashed(PixelPoint::new(right, top), (line, height), false);
        }
    }

    /// `from` から幅 × 高さ `size` の帯を、横（`across`）か縦に点線で描く。
    fn dashed(&mut self, from: PixelPoint, (width, height): (u32, u32), across: bool) {
        let dash = self.metrics.px(CUT_DASH).max(1);
        let length = if across { width } else { height };
        let mut offset = 0_u32;
        while offset < length {
            let piece = dash.min(length.saturating_sub(offset));
            let (origin, size) = if across {
                (
                    PixelPoint::new(from.x().saturating_add(offset), from.y()),
                    PixelSize::new(piece, height),
                )
            } else {
                (
                    PixelPoint::new(from.x(), from.y().saturating_add(offset)),
                    PixelSize::new(width, piece),
                )
            };
            self.overlay(PixelRect::new(origin, size));
            offset = offset.saturating_add(dash.saturating_mul(2));
        }
    }

    /// カーソルの色の矩形を、後から重ねる命令に積む。
    fn overlay(&mut self, rect: PixelRect) {
        let color = self.palette.cursor();
        self.overlays.push(DrawCommand::Fill { rect, color });
    }

    /// `style` の行の `column` 列目のセルの印。見出しの行は選ばない。
    fn mark(&self, style: RowStyle, column: usize) -> CellMark {
        let body = match style {
            RowStyle::Header | RowStyle::Numbers if self.covers_column(column) => {
                return CellMark::SelectedHeader;
            }
            RowStyle::Header | RowStyle::Numbers => return CellMark::Header,
            RowStyle::Body(body) => body,
        };
        let Some(ranges) = self.selection else {
            return CellMark::Unselected;
        };
        let cursor = ranges.active().cursor();
        if cursor.body() == body && cursor.column() == column {
            CellMark::Cursor
        } else if ranges.contains(body, column) {
            CellMark::Selected
        } else if self.covers_row(body) || self.covers_column(column) {
            CellMark::Band
        } else {
            CellMark::Unselected
        }
    }

    /// カーソルのセルの枠（セルの内側に `CURSOR_LINE` の太さで 4 辺）。
    fn outline(&mut self, rect: PixelRect) {
        let (left, top) = (rect.origin().x(), rect.origin().y());
        let (width, height) = (rect.size().width(), rect.size().height());
        let line = self.metrics.px(CURSOR_LINE).min(width).min(height);
        let right = left.saturating_add(width).saturating_sub(line);
        let bottom = top.saturating_add(height).saturating_sub(line);
        for edge in [
            PixelRect::new(PixelPoint::new(left, top), PixelSize::new(width, line)),
            PixelRect::new(PixelPoint::new(left, bottom), PixelSize::new(width, line)),
            PixelRect::new(PixelPoint::new(left, top), PixelSize::new(line, height)),
            PixelRect::new(PixelPoint::new(right, top), PixelSize::new(line, height)),
        ] {
            self.overlay(edge);
        }
    }

    /// 1 つのセル。列の足りない行（セルが無い）は、空のセルと区別できるよう行番号の列の色で塗る（KEEP-05）。
    /// 見出しの行のセルは行番号の列の色、選んだセルは選択の色を背景にする。
    fn paint_cell(&mut self, rect: PixelRect, cell: Option<&String>, mark: CellMark) {
        let (background, missing) = match mark {
            CellMark::Header => (self.palette.gutter(), self.palette.gutter()),
            CellMark::SelectedHeader => (self.palette.selection(), self.palette.selection()),
            CellMark::Unselected => (self.palette.background(), self.palette.gutter()),
            // 帯の中でも、セルが無い所は「セルが無い」色のまま（KEEP-05）。
            CellMark::Band => (self.palette.band(), self.palette.gutter()),
            CellMark::Selected | CellMark::Cursor => {
                (self.palette.selection(), self.palette.selection())
            }
        };
        let Some(text) = cell else {
            self.fill(rect, missing);
            return;
        };
        self.fill(rect, background);
        if !text.is_empty() {
            self.text(rect, text, self.palette.text());
        }
    }

    /// 矩形を塗る。
    fn fill(&mut self, rect: PixelRect, color: Color) {
        self.commands.push(DrawCommand::Fill { rect, color });
    }

    /// `rect` の中に、左の余白を空けて縦の真ん中に文字を書く。左右の余白の内に収まらなければ `…` で省略し
    /// （GitHub Issue #7）、`rect` の外は切り取る（幅は近似なので）。
    fn text(&mut self, rect: PixelRect, text: &str, color: Color) {
        let room = rect
            .size()
            .width()
            .saturating_sub(self.layout.padding().saturating_mul(2));
        let columns = room / self.metrics.half_width().max(1);
        let text = fitted(text, columns);
        let spare = rect
            .size()
            .height()
            .saturating_sub(self.metrics.line_height());
        let baseline = PixelPoint::new(
            rect.origin().x().saturating_add(self.layout.padding()),
            rect.origin()
                .y()
                .saturating_add(spare / 2)
                .saturating_add(self.metrics.ascent()),
        );
        self.commands.push(DrawCommand::Text {
            baseline,
            text,
            color,
            clip: rect,
            role: TextRole::Content,
        });
    }
}

/// 欄の文字列 `text` を、カーソル（左からの字数 `caret`）が幅 `room` 字の内に見えるよう、頭の字を飛ばした残りと、
/// 飛ばした字数。カーソルの棒の分として 1 字を空ける。
fn scrolled_to_caret(text: &str, caret: u32, room: u32) -> (String, u32) {
    let limit = room.saturating_sub(1);
    let mut skipped = 0_u32;
    let mut characters = text.chars();
    while caret.saturating_sub(skipped) > limit {
        let Some(character) = characters.next() else {
            break;
        };
        skipped = skipped.saturating_add(char_columns(character));
    }
    (characters.collect(), skipped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use nene_ledger_core::column_widths::ColumnWidths;
    use nene_ledger_core::table_layout::GRID_LINE;

    #[test]
    fn a_long_field_skips_its_head_so_that_the_caret_stays_visible() {
        assert_eq!(scrolled_to_caret("abc", 3, 10), (String::from("abc"), 0));
        assert_eq!(scrolled_to_caret("abcdef", 6, 4), (String::from("def"), 3));
        // 全角は 2 字ぶん飛ぶ。
        assert_eq!(scrolled_to_caret("日本語x", 7, 4), (String::from("語x"), 4));
        assert_eq!(scrolled_to_caret("", 0, 0), (String::new(), 0));
    }

    /// 試しの配色（表の先頭のライト `neutral-light` を導いたもの）。
    fn palette() -> Palette {
        crate::theme_state::ThemeState::at(1).palette()
    }

    fn metrics() -> GlyphMetrics {
        GlyphMetrics::new(8, 16, 12)
    }

    fn row(record: u64, cells: &[&str]) -> ViewRow {
        let cells: Vec<String> = cells.iter().map(|&cell| String::from(cell)).collect();
        ViewRow::new(record, &cells)
    }

    /// 本体の続いた行 `rows`（最初のレコードの番号から本体の番目も同じ）。
    fn view(rows: &[ViewRow]) -> TableView {
        let cells: Vec<Vec<String>> = rows.iter().map(|row| row.cells().to_vec()).collect();
        TableView::from_records(rows.first().map_or(0, ViewRow::line), &cells, None)
    }

    /// 見出しと本体の行から列の幅を決め、窓の大きさ `size` に描く。
    fn paint(size: PixelSize, header: Option<&ViewRow>, rows: &[ViewRow]) -> Vec<DrawCommand> {
        paint_at(size, header, rows, None)
    }

    /// 窓の大きさ 800 × 600 に、表示の最初の列 `first_column` から、選択 `selection` を付けて描く。
    fn paint_selected(
        header: Option<&ViewRow>,
        rows: &[ViewRow],
        selection: Option<Selection>,
        first_column: usize,
    ) -> Vec<DrawCommand> {
        paint_from(
            PixelSize::new(800, 600),
            header,
            rows,
            selection,
            first_column,
        )
    }

    /// 窓の大きさ `size` に、選択 `selection` を付けて描く。
    fn paint_at(
        size: PixelSize,
        header: Option<&ViewRow>,
        rows: &[ViewRow],
        selection: Option<Selection>,
    ) -> Vec<DrawCommand> {
        paint_from(size, header, rows, selection, 0)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "RS-011: 試しの描画の組み立てだけ。製品の経路ではない"
    )]
    fn paint_from(
        size: PixelSize,
        header: Option<&ViewRow>,
        rows: &[ViewRow],
        selection: Option<Selection>,
        first_column: usize,
    ) -> Vec<DrawCommand> {
        let cells: Vec<&[String]> = header.into_iter().chain(rows).map(ViewRow::cells).collect();
        let last = rows.last().map_or(0, |row| row.line() + 1);
        let layout = TableLayout::new(&ColumnWidths::fit(&cells), first_column, last, metrics());
        let painter = FramePainter::new(&layout, metrics(), palette(), size);
        let painter = match selection {
            Some(selection) => painter.with_selection(selection),
            None => painter,
        };
        painter.paint(header, None, &view(rows))
    }

    #[test]
    fn frozen_columns_are_painted_left_of_the_scrolled_ones_with_an_edge() {
        let header = row(0, &["aaaaa", "bbbbbbbbbb", "c", "dd"]);
        let body = [row(1, &["x1", "x2", "x3", "x4"])];
        let cells: Vec<&[String]> = core::iter::once(&header)
            .chain(&body)
            .map(ViewRow::cells)
            .collect();
        // 1 列目を固定し、表示は 3 列目から。2 列目は描かない。
        let layout = TableLayout::new(&ColumnWidths::fit(&cells), 2, 2, metrics()).with_frozen(1);
        let commands = FramePainter::new(&layout, metrics(), palette(), PixelSize::new(800, 600))
            .paint(Some(&header), None, &view(&body));
        let shown = texts(&commands);
        assert!(shown.contains(&"aaaaa") && shown.contains(&"c") && shown.contains(&"dd"));
        assert!(shown.contains(&"x1") && shown.contains(&"x3"));
        assert!(!shown.contains(&"bbbbbbbbbb") && !shown.contains(&"x2"));
        // 固定した列の右の境に、描いた 2 行ぶんの縦の線。
        let right = layout.gutter_width() + layout.frozen_width();
        let edge = PixelRect::new(
            PixelPoint::new(right - 2, 0),
            PixelSize::new(2, layout.row_height() * 2),
        );
        assert!(fills_of(&commands, palette().border()).contains(&edge));
        // 固定が無ければ線も無い。
        let plain = TableLayout::new(&ColumnWidths::fit(&cells), 0, 2, metrics());
        let unfrozen = FramePainter::new(&plain, metrics(), palette(), PixelSize::new(800, 600))
            .paint(Some(&header), None, &view(&body));
        assert!(fills_of(&unfrozen, palette().border()).is_empty());
    }

    fn texts(commands: &[DrawCommand]) -> Vec<&str> {
        commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Text { text, .. } => Some(text.as_str()),
                DrawCommand::Fill { .. }
                | DrawCommand::Line { .. }
                | DrawCommand::TextSpan { .. } => None,
            })
            .collect()
    }

    fn fills_of(commands: &[DrawCommand], wanted: Color) -> Vec<PixelRect> {
        commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Fill { rect, color } if *color == wanted => Some(*rect),
                DrawCommand::Fill { .. }
                | DrawCommand::Text { .. }
                | DrawCommand::Line { .. }
                | DrawCommand::TextSpan { .. } => None,
            })
            .collect()
    }

    #[test]
    fn rows_numbers_and_cells_are_drawn_in_order() {
        let rows = [row(0, &["id", "名前"]), row(1, &["1", "東京\n都"])];
        let commands = paint(PixelSize::new(800, 600), None, &rows);
        assert_eq!(
            texts(&commands),
            vec!["1", "id", "名前", "2", "1", "東京¶都"]
        );
        let Some(DrawCommand::Fill { color, .. }) = commands.first() else {
            panic!("最初は背景");
        };
        assert_eq!(*color, palette().background());
    }

    #[test]
    fn the_header_stays_on_top_with_its_record_number() {
        let header = row(2, &["id", "名前"]);
        let rows = [row(40, &["7", "a"]), row(41, &["8", "b"])];
        let commands = paint(PixelSize::new(800, 600), Some(&header), &rows);
        // 見出しは最初の行に、行番号はレコードの番号（1 始まり）で出す。本体はその下から。
        assert_eq!(
            texts(&commands),
            vec!["3", "id", "名前", "41", "7", "a", "42", "8", "b"]
        );
        // 見出しのセル 2 つは行番号の列の色。行番号の列 3 つも同じ色。
        assert_eq!(fills_of(&commands, palette().gutter()).len(), 5);
    }

    #[test]
    fn text_is_clipped_to_its_cell_and_sits_on_a_baseline() {
        let commands = paint(PixelSize::new(800, 600), None, &[row(0, &["abc"])]);
        let (baseline, clip) = commands
            .iter()
            .find_map(|command| match command {
                DrawCommand::Text {
                    text,
                    baseline,
                    clip,
                    ..
                } if text == "abc" => Some((*baseline, *clip)),
                DrawCommand::Text { .. }
                | DrawCommand::Fill { .. }
                | DrawCommand::Line { .. }
                | DrawCommand::TextSpan { .. } => None,
            })
            .unwrap();
        assert_eq!(baseline.x(), clip.origin().x() + 8);
        // 行の高さ 25（罫線を除くと 24）。余り 8 の半分 4 ＋ ascent 12。
        assert_eq!(baseline.y(), 4 + 12);
        assert_eq!(clip.size().height(), 24);
    }

    #[test]
    fn only_rows_and_columns_inside_the_window_are_drawn() {
        let tall: Vec<ViewRow> = (0..100_u64)
            .map(|n| row(41 + n, &[n.to_string().as_str()]))
            .collect();
        // 高さ 60 に行の高さ 25 は 3 行（最後は一部）。見出しがあれば本体は 2 行。
        let bare = paint(PixelSize::new(800, 60), None, &tall);
        assert_eq!(texts(&bare), vec!["42", "0", "43", "1", "44", "2"]);
        let header = row(0, &["n"]);
        let with_header = paint(PixelSize::new(800, 60), Some(&header), &tall);
        assert_eq!(texts(&with_header), vec!["1", "n", "42", "0", "43", "1"]);
        let narrow = paint(
            PixelSize::new(90, 600),
            None,
            &[row(0, &["aaaa", "bbbb", "cccc"])],
        );
        assert_eq!(texts(&narrow), vec!["1", "aaaa"]);
    }

    #[test]
    fn a_missing_cell_differs_from_an_empty_cell() {
        let commands = paint(
            PixelSize::new(800, 600),
            None,
            &[row(0, &["a", ""]), row(1, &["b"])],
        );
        // 行番号の列 2 つ ＋ 2 行目の欠けたセル 1 つ。1 行目の空のセルは背景の色。
        assert_eq!(fills_of(&commands, palette().gutter()).len(), 3);
        assert_eq!(texts(&commands), vec!["1", "a", "2", "b"]);
    }

    #[test]
    fn the_row_band_stops_at_the_table_width() {
        let commands = paint(PixelSize::new(800, 600), None, &[row(0, &["a"])]);
        let band = fills_of(&commands, palette().grid());
        // 行番号の列 41 ＋ 列 1 つ（下限 3 字）41。窓の幅 800 までは塗らない。
        assert_eq!(band.first().map(|rect| rect.size().width()), Some(82));
        assert_eq!(GRID_LINE, 1);
    }

    #[test]
    fn an_empty_table_is_only_the_background() {
        assert_eq!(paint(PixelSize::new(100, 100), None, &[]).len(), 1);
    }

    #[test]
    fn the_selection_and_the_cursor_are_marked() {
        use crate::cell_position::CellPosition;
        use crate::selection_mode::SelectionMode;
        let header = row(0, &["a", "b"]);
        let rows = [row(1, &["1", "2"]), row(2, &["3", "4"]), row(3, &["5"])];
        // 試しの本体はレコードの番号と同じ番目（1〜3）。その 2 列目を選び、カーソルは 3 番目（欠けたセル）。
        let selection = Selection::at(CellPosition::new(1, 1))
            .moved(CellPosition::new(3, 1), SelectionMode::Extend);
        let commands = paint_selected(Some(&header), &rows, Some(selection), 0);
        // 選んだセル 3 つ（欠けたセルも選択の色）＋ 選んだ行の行番号 3 つ ＋ 選んだ列の見出し 1 つ。
        assert_eq!(fills_of(&commands, palette().selection()).len(), 7);
        // カーソルの枠は太さ 2 の 4 辺。本体の 3 番目の行（表の 4 行目・y = 75）の 2 列目。
        let cursor_color = fills_of(&commands, palette().cursor());
        let outline: Vec<&PixelRect> = cursor_color
            .iter()
            .filter(|edge| edge.size().width().min(edge.size().height()) == 2)
            .collect();
        assert_eq!(outline.len(), 4);
        assert!(
            outline
                .iter()
                .all(|edge| edge.origin().y() >= 75 && edge.origin().y() < 100)
        );
        // 外周の枠は太さ 1。1 列の選択なので 3 つのセルの左右 6 辺 ＋ 上端 1 ＋ 下端 1。
        assert_eq!(cursor_color.len() - outline.len(), 8);
        // 枠はどの塗りよりも後ろに重ねる。
        assert!(matches!(
            commands.last(),
            Some(DrawCommand::Fill { color, .. }) if *color == palette().cursor()
        ));
    }

    #[test]
    fn selected_rows_and_columns_mark_their_numbers_and_headers() {
        use crate::cell_position::CellPosition;
        let header = row(0, &["a", "b"]);
        let rows = [row(1, &["1", "2"]), row(2, &["3", "4"])];
        let selection = Selection::at(CellPosition::new(2, 0));
        let commands = paint_selected(Some(&header), &rows, Some(selection), 0);
        let marked = fills_of(&commands, palette().selection());
        // 選んだセル ＋ その行の行番号（表の 3 行目・y = 50）＋ その列の見出し（y = 0）。
        assert_eq!(marked.len(), 3);
        assert!(
            marked
                .iter()
                .any(|rect| rect.origin() == PixelPoint::new(0, 50))
        );
        assert!(
            marked
                .iter()
                .any(|rect| rect.origin().y() == 0 && rect.origin().x() > 0)
        );
    }

    #[test]
    fn scrolling_right_hides_the_left_columns_but_keeps_row_numbers() {
        let rows = [row(0, &["left", "right"])];
        let commands = paint_selected(None, &rows, None, 1);
        assert_eq!(texts(&commands), vec!["1", "right"]);
    }

    #[test]
    fn the_rows_and_columns_of_the_selection_get_a_faint_band() {
        use crate::cell_position::CellPosition;
        let rows = [
            row(0, &["a", "b", "c"]),
            row(1, &["d", "e", "f"]),
            row(2, &["g", "h"]),
        ];
        // 本体の 1 番目の 2 列目（e）だけを選ぶ。
        let selection = Selection::at(CellPosition::new(1, 1));
        let commands = paint_selected(None, &rows, Some(selection), 0);
        // 同じ行の d・f と、同じ列の b・h が帯。2 行目（g の行）の 3 列目は欠けたセルで帯の外。
        assert_eq!(fills_of(&commands, palette().band()).len(), 4);
        // 帯の中の欠けたセルは「セルが無い」色のまま。
        let tall = [row(0, &["a", "b"]), row(1, &["c"])];
        let column_band =
            paint_selected(None, &tall, Some(Selection::at(CellPosition::new(0, 1))), 0);
        assert_eq!(fills_of(&column_band, palette().band()).len(), 1);
        // 選択が無ければ帯も無い。
        assert!(
            fills_of(
                &paint(PixelSize::new(800, 600), None, &rows),
                palette().band()
            )
            .is_empty()
        );
    }
}
