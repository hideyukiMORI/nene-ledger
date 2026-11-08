//! 詳細ペイン（カーソルのセルの全文）を描画命令に変える（VIEW-07）。

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use nene_ledger_core::color::Color;
use nene_ledger_core::display_width::display_columns;
use nene_ledger_core::draw_command::DrawCommand;
use nene_ledger_core::glyph_metrics::GlyphMetrics;
use nene_ledger_core::json_layout::pretty_json;
use nene_ledger_core::palette::Palette;
use nene_ledger_core::pixel_point::PixelPoint;
use nene_ledger_core::pixel_rect::PixelRect;
use nene_ledger_core::pixel_size::PixelSize;
use nene_ledger_core::table_layout::{GRID_LINE, TableLayout};
use nene_ledger_core::text_encoding::TextEncoding;
use nene_ledger_core::text_role::TextRole;

use crate::client_bands::{ClientBands, DETAIL_PADDING, DETAIL_RULE};
use crate::detail_content::DetailContent;
use crate::detail_place::DetailPlace;
use crate::status_line::{byte_encoding_name, group_digits};
use crate::text_wrap::wrap;

/// 本文の左の帯の太さ（カーソルの枠と同じ青。このペインがカーソルのセルの拡大だと示す）。
const BAR: u32 = 3;
/// 文字の左端（帯 3 ＋ 空き 9）。
const TEXT_LEFT: u32 = 12;
/// 右の余白。字の幅は近似で測るので広めにとる。
const RIGHT_PADDING: u32 = 16;
/// 開閉の案内。
const HINT: &str = "Ctrl+Shift+U で閉じる";

/// 詳細ペインの描画命令を、描く順に積む。上から表との境の線・見出しの行（列の名前と補助の情報・開閉の案内）・本文。
#[derive(Debug)]
pub(crate) struct DetailPainter {
    pane: PixelRect,
    metrics: GlyphMetrics,
    palette: Palette,
    encoding: TextEncoding,
    commands: Vec<DrawCommand>,
}

impl DetailPainter {
    /// 矩形 `pane`（窓の中身の中の位置）に描く。値のバイト数はファイルの文字コード `encoding` で数える。
    pub(crate) fn new(
        pane: PixelRect,
        metrics: GlyphMetrics,
        palette: Palette,
        encoding: TextEncoding,
    ) -> Self {
        Self {
            pane,
            metrics,
            palette,
            encoding,
            commands: Vec::new(),
        }
    }

    /// カーソルのセルの場所 `place` と中身 `content` の命令。
    pub(crate) fn paint(
        mut self,
        place: DetailPlace<'_>,
        content: DetailContent<'_>,
    ) -> Vec<DrawCommand> {
        let (left, top) = (self.pane.origin().x(), self.pane.origin().y());
        let width = self.pane.size().width();
        self.fill(self.pane, self.palette.background());
        self.fill(
            PixelRect::new(
                PixelPoint::new(left, top),
                PixelSize::new(width, self.metrics.px(DETAIL_RULE)),
            ),
            self.palette.border(),
        );
        let header_top = top.saturating_add(self.metrics.px(DETAIL_RULE));
        self.header(header_top, place, content);
        let body_top = header_top.saturating_add(TableLayout::row_height_for(self.metrics));
        let bottom = top.saturating_add(self.pane.size().height());
        self.fill(
            PixelRect::new(
                PixelPoint::new(left, body_top),
                PixelSize::new(self.metrics.px(BAR), bottom.saturating_sub(body_top)),
            ),
            self.palette.cursor(),
        );
        self.body(
            body_top.saturating_add(self.metrics.px(DETAIL_PADDING)),
            content,
        );
        self.commands
    }

    /// 見出しの行。背景は表の見出しの行と同じ色、下に罫線。列の名前（幅の半分まで）・補助の情報・右端に開閉の案内。
    fn header(&mut self, top: u32, place: DetailPlace<'_>, content: DetailContent<'_>) {
        let left = self.pane.origin().x();
        let width = self.pane.size().width();
        let height = TableLayout::row_height_for(self.metrics);
        let band = PixelRect::new(PixelPoint::new(left, top), PixelSize::new(width, height));
        self.fill(band, self.palette.gutter());
        self.fill(
            PixelRect::new(
                PixelPoint::new(
                    left,
                    top.saturating_add(height)
                        .saturating_sub(self.metrics.px(GRID_LINE)),
                ),
                PixelSize::new(width, self.metrics.px(GRID_LINE)),
            ),
            self.palette.grid(),
        );
        let column = format!("{} 列目", place.column().saturating_add(1));
        let name = place.name().map_or_else(|| column.clone(), String::from);
        let name_left = left.saturating_add(self.metrics.px(TEXT_LEFT));
        // 字の幅は近似（半角 1・全角 2 × 数字の幅）なので、英大文字などは近似より長い。名前は幅の半分までを見せ、
        // 補助の情報は近似の 1.5 倍の所から書く（重ならないよう広めにとる）。
        let half = width / 2;
        let estimate = self.columns_width(&name);
        let name_width = estimate.saturating_add(estimate / 2).min(half);
        let line = height.saturating_sub(self.metrics.px(GRID_LINE));
        self.text_in(area(name_left, top, half, line), name, self.palette.text());
        let hint_width = self.columns_width(HINT);
        let hint_left = (left.saturating_add(width))
            .saturating_sub(self.metrics.px(RIGHT_PADDING))
            .saturating_sub(hint_width);
        self.text_in(
            area(hint_left, top, hint_width, line),
            String::from(HINT),
            self.palette.gutter_text(),
        );
        // 列の名前の後ろは全角 2 つ分空ける。
        let aux_left = name_left
            .saturating_add(name_width)
            .saturating_add(self.metrics.half_width().saturating_mul(4));
        let aux = aux_text(place, content, &column, self.encoding);
        let aux_width = hint_left.saturating_sub(aux_left);
        self.text_in(
            area(aux_left, top, aux_width, line),
            aux,
            self.palette.gutter_text(),
        );
    }

    /// 本文。値を折り返して、ペインの高さに入る行の数まで（`ClientBands::body_lines`）。収まらなければ最後の行で
    /// 残りの行の数を言う。値が無いときは断り書き。
    fn body(&mut self, top: u32, content: DetailContent<'_>) {
        let width = self.pane.size().width();
        let text_width = width
            .saturating_sub(self.metrics.px(TEXT_LEFT))
            .saturating_sub(self.metrics.px(RIGHT_PADDING));
        let columns = text_width / self.metrics.half_width().max(1);
        let room = ClientBands::body_lines(self.pane.size().height(), self.metrics);
        let limit = usize::try_from(room).unwrap_or(usize::MAX);
        let (lines, note) = body_lines(content, columns, limit);
        let left = self
            .pane
            .origin()
            .x()
            .saturating_add(self.metrics.px(TEXT_LEFT));
        let line = ClientBands::body_line(self.metrics);
        let mut row_top = top;
        for text in lines {
            self.text_in(
                area(left, row_top, text_width, line),
                text,
                self.palette.text(),
            );
            row_top = row_top.saturating_add(line);
        }
        if let Some(text) = note {
            self.text_in(
                area(left, row_top, text_width, line),
                text,
                self.palette.gutter_text(),
            );
        }
    }

    /// `text` の幅の近似（半角 1・全角 2 の字数 × 半角の幅）。
    fn columns_width(&self, text: &str) -> u32 {
        display_columns(text).saturating_mul(self.metrics.half_width())
    }

    /// 矩形 `rect` の左端から、縦の真ん中で `text` を書く。矩形の外は切り取る。
    fn text_in(&mut self, rect: PixelRect, text: String, color: Color) {
        let (left, top) = (rect.origin().x(), rect.origin().y());
        let spare = rect
            .size()
            .height()
            .saturating_sub(self.metrics.line_height());
        self.commands.push(DrawCommand::Text {
            baseline: PixelPoint::new(
                left,
                top.saturating_add(spare / 2)
                    .saturating_add(self.metrics.ascent()),
            ),
            text,
            color,
            clip: rect,
            role: TextRole::Content,
        });
    }

    fn fill(&mut self, rect: PixelRect, color: Color) {
        self.commands.push(DrawCommand::Fill { rect, color });
    }
}

/// 本文の行（幅 `columns` で折り返し、`limit` 行まで）と、最後に添える断り書き（値が無い・あと N 行）。
fn body_lines(
    content: DetailContent<'_>,
    columns: u32,
    limit: usize,
) -> (Vec<String>, Option<String>) {
    match content {
        DetailContent::OutOfView => (
            Vec::new(),
            Some(String::from("〈カーソルの行は見えている範囲の外〉")),
        ),
        DetailContent::Missing => (
            Vec::new(),
            Some(String::from("〈この行にはこの列のセルが無い〉")),
        ),
        DetailContent::NoRows => (
            Vec::new(),
            Some(String::from("〈表に行が無い（見出しの行だけ）〉")),
        ),
        DetailContent::Value("") | DetailContent::Cut { value: "", .. } => {
            (Vec::new(), Some(String::from("〈空のセル〉")))
        }
        DetailContent::Value(value) | DetailContent::Cut { value, .. } => {
            // JSON のオブジェクトか配列のセルは整形して見せる（値は変えない・切り詰めた値は整形しない・VIEW-07）。
            let pretty = match content {
                DetailContent::Value(_) => pretty_json(value),
                DetailContent::Cut { .. }
                | DetailContent::OutOfView
                | DetailContent::Missing
                | DetailContent::NoRows => None,
            };
            let wrapped = wrap(pretty.as_deref().unwrap_or(value), columns, limit);
            if wrapped.hidden() == 0 {
                return (wrapped.lines().to_vec(), None);
            }
            let shown = wrapped
                .lines()
                .get(..limit.saturating_sub(1))
                .unwrap_or_default();
            let rest = wrapped.hidden().saturating_add(1);
            (
                shown.to_vec(),
                Some(format!(
                    "〈あと {} 行〉",
                    group_digits(u64::try_from(rest).unwrap_or(u64::MAX))
                )),
            )
        }
    }
}

/// 見出しの行の補助の情報。行（と見出しがあれば列）の番号、値があれば文字数とファイルの文字コード `encoding` での
/// バイト数（Issue #59）。全角の空きで区切る。
fn aux_text(
    place: DetailPlace<'_>,
    content: DetailContent<'_>,
    column: &str,
    encoding: TextEncoding,
) -> String {
    let row = format!("{} 行目", group_digits(place.record().saturating_add(1)));
    let position = match place.name() {
        Some(_) => format!("{row}　{column}"),
        None => row,
    };
    match content {
        DetailContent::Value(value) => {
            let characters = u64::try_from(value.chars().count()).unwrap_or(u64::MAX);
            // 整形して見せている JSON には印を添える（数は元の値のもの・VIEW-07）。
            let json = if pretty_json(value).is_some() {
                "　JSON（整形して表示）"
            } else {
                ""
            };
            format!(
                "{position}　{} 文字　{} で {} バイト{json}",
                group_digits(characters),
                byte_encoding_name(encoding),
                group_digits(encoding.encoded_length(value))
            )
        }
        // 切り詰めた行の値は先頭の部分だけなので、文字数とバイト数の代わりに切り詰めたことと全体の大きさを出す（GitHub Issue #36）。
        DetailContent::Cut { cut, .. } => format!("{position}　{}", cut.describe()),
        DetailContent::OutOfView | DetailContent::Missing => position,
        // 本体の行が無いので、行の番号は言わない（列の名前だけ・GitHub Issue #256）。
        DetailContent::NoRows => match place.name() {
            Some(_) => String::from(column),
            None => String::new(),
        },
    }
}

/// 左端 `left`・上端 `top`・幅 `width`・高さ `height` の矩形。
fn area(left: u32, top: u32, width: u32, height: u32) -> PixelRect {
    PixelRect::new(PixelPoint::new(left, top), PixelSize::new(width, height))
}
