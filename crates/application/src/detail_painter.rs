//! 詳細ペイン（カーソルのセルの全文）を描画命令に変える（VIEW-07）。

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use nene_ledger_core::char_advances::{CharAdvances, SUBPIXELS};
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
use nene_ledger_core::text_caret::TextCaret;
use nene_ledger_core::text_encoding::TextEncoding;
use nene_ledger_core::text_role::TextRole;

use crate::cell_display::mark_legend;
use crate::client_bands::{ClientBands, DETAIL_PADDING, DETAIL_RULE};
use crate::detail_content::DetailContent;
use crate::detail_place::DetailPlace;
use crate::shown_body::ShownBody;
use crate::status_line::{byte_encoding_name, group_digits};
use crate::text_wrap::{caret_piece, caret_place, index_at, wrap};

/// 本文の左の帯の太さ（カーソルの枠と同じ青。このペインがカーソルのセルの拡大だと示す）。
const BAR: u32 = 3;
/// 文字の左端（帯 3 ＋ 空き 9）。
const TEXT_LEFT: u32 = 12;
/// 右の余白。字の幅は近似で測るので広めにとる。
const RIGHT_PADDING: u32 = 16;
/// 開閉の案内。
const HINT: &str = "Ctrl+Shift+U で閉じる";
/// ペインで直している間の、見出しの行の右の案内（GitHub Issue #458）。
const EDITING_HINT: &str = "直している · Enter 確定 · Alt+Enter 改行 · Esc やめる";
/// ペインで直している間のカーソルの棒の幅（画素・100% のとき・GitHub Issue #458）。
const CARET: u32 = 2;
/// 本文のスクロールバーの幅とつまみの最小の長さ（画素・100% のとき・GitHub Issue #454）。
const PANE_SCROLLBAR: u32 = 4;
const PANE_SCROLLBAR_MIN: u32 = 12;

/// 詳細ペインの描画命令を、描く順に積む。上から表との境の線・見出しの行（列の名前と補助の情報・開閉の案内）・本文。
#[derive(Debug)]
pub(crate) struct DetailPainter {
    pane: PixelRect,
    metrics: GlyphMetrics,
    palette: Palette,
    encoding: TextEncoding,
    commands: Vec<DrawCommand>,
    /// 本文の見せる最初の行（折り返した行の番目・GitHub Issue #454）。
    scroll: usize,
    /// ペインで直している間のカーソル（棒の前の字の数と選択・本文の値の中・GitHub Issue #458・#460）。
    caret: Option<TextCaret>,
    /// 中身の字の送り幅（カーソルの棒と選択の位置・GitHub Issue #462）。
    advances: CharAdvances,
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
            scroll: 0,
            caret: None,
            advances: CharAdvances::uniform(metrics.half_width()),
        }
    }

    /// ペインで直している：本文の `caret` の字の前にカーソルの棒を描き、選んだ字の下を選択の色で塗り、棒が見えるように
    /// 本文を動かし、見出しの行に直し方を出す（GitHub Issue #458・#460）。
    pub(crate) fn with_caret(self, caret: Option<TextCaret>) -> Self {
        Self { caret, ..self }
    }

    /// カーソルの棒と選択の位置を、UI が測った字の送り幅 `advances` で決める（GitHub Issue #462）。
    pub(crate) fn with_advances(self, advances: CharAdvances) -> Self {
        Self { advances, ..self }
    }

    /// 本文を、折り返した `scroll` 行目から見せる（ペインの中のスクロール・GitHub Issue #454）。
    pub(crate) fn with_scroll(self, scroll: usize) -> Self {
        Self { scroll, ..self }
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
        // ペインで直している間は、閉じ方の代わりに直し方を強調の色で（見ているときと見分ける・GitHub Issue #458）。
        let (hint, hint_color) = if self.caret.is_some() {
            (EDITING_HINT, self.palette.cursor())
        } else {
            (HINT, self.palette.gutter_text())
        };
        let hint_width = self.columns_width(hint);
        let hint_left = (left.saturating_add(width))
            .saturating_sub(self.metrics.px(RIGHT_PADDING))
            .saturating_sub(hint_width);
        self.text_in(
            area(hint_left, top, hint_width, line),
            String::from(hint),
            hint_color,
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

    /// 本文。値を折り返して、`scroll` 行目からペインの高さに入る行の数まで（`ClientBands::body_lines`）。下に続きが
    /// あれば最後の行で残りの行の数を言い、右にスクロールバーを出す（GitHub Issue #454）。値が無いときは断り書き。
    fn body(&mut self, top: u32, content: DetailContent<'_>) {
        let text_width = text_width(self.pane, self.metrics);
        let columns = text_columns(self.pane, self.metrics);
        let room = ClientBands::body_lines(self.pane.size().height(), self.metrics);
        let limit = usize::try_from(room).unwrap_or(usize::MAX);
        // 直している間は、カーソルの棒の行が見えるように本文を動かす。
        let editing = self.caret.zip(match content {
            DetailContent::Value(value) => Some(value),
            DetailContent::Cut { .. }
            | DetailContent::OutOfView
            | DetailContent::Missing
            | DetailContent::NoRows => None,
        });
        let shown = match editing {
            Some((caret, value)) => {
                let first =
                    editing_first(value, columns, limit, (self.scroll, Some(caret.cursor())));
                body_lines_for_editing(content, columns, limit, first)
            }
            None => body_lines(content, columns, limit, self.scroll),
        };
        let (total, first) = (shown.total(), shown.first());
        let (lines, note) = shown.into_lines();
        let left = self
            .pane
            .origin()
            .x()
            .saturating_add(self.metrics.px(TEXT_LEFT));
        let line = ClientBands::body_line(self.metrics);
        if let Some((caret, value)) = editing.filter(|(caret, _)| caret.has_selection()) {
            let start = place_in_pixels(&self.advances, (value, columns), caret.start());
            let end = place_in_pixels(&self.advances, (value, columns), caret.end());
            self.selection_fills((left, top, line), (&lines, first), (start, end));
        }
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
        if let Some((caret, value)) = editing {
            let place = place_in_pixels(&self.advances, (value, columns), caret.cursor());
            self.caret_bar((left, top, line), place, first);
        }
        if total > limit {
            self.scrollbar(top, line.saturating_mul(room), (first, limit, total));
        }
    }

    /// 選んだ字（折り返した行と左からの画素で `start` から `end` の前まで）の下を選択の色で塗る。見せている行 `lines`
    /// （`first` 行目から）の中だけ。次の行へ続く行は、右に半角 1 つ分も塗る（改行も選んでいる印・GitHub Issue #460）。
    fn selection_fills(
        &mut self,
        (left, top, line): (u32, u32, u32),
        (lines, first): (&[String], usize),
        ((start_row, start_column), (end_row, end_column)): ((usize, u32), (usize, u32)),
    ) {
        let half = self.metrics.half_width();
        for (offset, text) in lines.iter().enumerate() {
            let row = first.saturating_add(offset);
            if !(start_row..=end_row).contains(&row) {
                continue;
            }
            let from = if row == start_row { start_column } else { 0 };
            let to = if row == end_row {
                end_column
            } else {
                self.advances.width(text).saturating_add(half)
            };
            let visible = u32::try_from(offset).unwrap_or(u32::MAX);
            self.fill(
                area(
                    left.saturating_add(from),
                    top.saturating_add(line.saturating_mul(visible)),
                    to.saturating_sub(from),
                    line,
                ),
                self.palette.selection(),
            );
        }
    }

    /// 直している間のカーソルの棒（本文の左上 `left`・`top`・1 行の高さ `line`、折り返した `row` 行目の左から `x` 画素、
    /// 見せている最初の行 `first`・GitHub Issue #458・#462）。
    fn caret_bar(
        &mut self,
        (left, top, line): (u32, u32, u32),
        (row, x): (usize, u32),
        first: usize,
    ) {
        let visible_row = u32::try_from(row.saturating_sub(first)).unwrap_or(u32::MAX);
        let caret_top = top.saturating_add(line.saturating_mul(visible_row));
        let caret_left = left.saturating_add(x);
        self.fill(
            area(caret_left, caret_top, self.metrics.px(CARET), line),
            self.palette.cursor(),
        );
    }

    /// 本文の右の細いスクロールバー（高さ `height` の所に、全部 `total` 行のうち `first` 行目から `shown` 行を見せている）。
    fn scrollbar(&mut self, top: u32, height: u32, (first, shown, total): (usize, usize, usize)) {
        let part = |count: usize| {
            u32::try_from(
                u64::from(height)
                    .saturating_mul(u64::try_from(count).unwrap_or(u64::MAX))
                    .checked_div(u64::try_from(total).unwrap_or(u64::MAX))
                    .unwrap_or(0),
            )
            .unwrap_or(height)
        };
        let thickness = self.metrics.px(PANE_SCROLLBAR);
        let left = self
            .pane
            .origin()
            .x()
            .saturating_add(self.pane.size().width())
            .saturating_sub(self.metrics.px(RIGHT_PADDING) / 2)
            .saturating_sub(thickness / 2);
        self.fill(area(left, top, thickness, height), self.palette.grid());
        let thumb = part(shown)
            .max(self.metrics.px(PANE_SCROLLBAR_MIN))
            .min(height);
        let thumb_top = top
            .saturating_add(part(first))
            .min(top.saturating_add(height).saturating_sub(thumb));
        self.fill(
            area(left, thumb_top, thickness, thumb),
            self.palette.gutter_text(),
        );
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

/// 本文の行（幅 `columns` で折り返し、`scroll` 行目から `limit` 行まで）。`scroll` は最後の画面に収める。
fn body_lines(content: DetailContent<'_>, columns: u32, limit: usize, scroll: usize) -> ShownBody {
    let note_only = ShownBody::note_only;
    let value = match content {
        DetailContent::OutOfView => return note_only("〈カーソルの行は見えている範囲の外〉"),
        DetailContent::Missing => return note_only("〈この行にはこの列のセルが無い〉"),
        DetailContent::NoRows => return note_only("〈表に行が無い（見出しの行だけ）〉"),
        DetailContent::Value("") | DetailContent::Cut { value: "", .. } => {
            return note_only("〈空のセル〉");
        }
        DetailContent::Value(value) | DetailContent::Cut { value, .. } => value,
    };
    // JSON のオブジェクトか配列のセルは整形して見せる（値は変えない・切り詰めた値は整形しない・VIEW-07）。
    let pretty = match content {
        DetailContent::Value(_) => pretty_json(value),
        DetailContent::Cut { .. }
        | DetailContent::OutOfView
        | DetailContent::Missing
        | DetailContent::NoRows => None,
    };
    let all = wrap(pretty.as_deref().unwrap_or(value), columns, usize::MAX);
    let total = all.lines().len();
    let first = scroll.min(total.saturating_sub(limit));
    let rest = total.saturating_sub(first);
    if rest <= limit {
        return ShownBody::new(
            all.lines().get(first..).unwrap_or_default().to_vec(),
            None,
            total,
            first,
        );
    }
    // 下に続きがあれば、最後の 1 行で残りの行の数を言う。
    let shown = limit.saturating_sub(1);
    ShownBody::new(
        all.lines()
            .get(first..first.saturating_add(shown))
            .unwrap_or_default()
            .to_vec(),
        Some(format!(
            "〈あと {} 行〉",
            group_digits(u64::try_from(rest.saturating_sub(shown)).unwrap_or(u64::MAX))
        )),
        total,
        first,
    )
}

/// 直している間の本文の行：「〈あと N 行〉」の断り書きを出さず、`scroll` 行目から `limit` 行を見せる（カーソルの棒の
/// 行が隠れないように・GitHub Issue #458）。空の値も断り書きにせず、空の 1 行にする。
fn body_lines_for_editing(
    content: DetailContent<'_>,
    columns: u32,
    limit: usize,
    scroll: usize,
) -> ShownBody {
    let DetailContent::Value(value) = content else {
        return body_lines(content, columns, limit, scroll);
    };
    let all = wrap(value, columns, usize::MAX);
    let total = all.lines().len();
    let first = scroll.min(total.saturating_sub(limit));
    ShownBody::new(
        all.lines()
            .get(first..first.saturating_add(limit).min(total))
            .unwrap_or_default()
            .to_vec(),
        None,
        total,
        first,
    )
}

/// 本文を幅 `pane`・字の寸法 `metrics` で折り返したときの、全部の行の数と、見せられる行の数（ホイールで動かせる範囲を
/// 決める・GitHub Issue #454）。
pub(crate) fn body_extent(
    pane: PixelRect,
    metrics: GlyphMetrics,
    content: DetailContent<'_>,
) -> (usize, usize) {
    let columns = text_columns(pane, metrics);
    let room = usize::try_from(ClientBands::body_lines(pane.size().height(), metrics))
        .unwrap_or(usize::MAX);
    (body_lines(content, columns, room, 0).total(), room)
}

/// 直している値 `value` を詳細ペイン（矩形 `pane`）に見せているとき、窓の中の `at`（左・上）に近い字の境目が何字目か
/// と、見せている最初の行。本文は `scroll` 行目から、カーソルの棒 `caret`（直していなければ `None`）が見えるように
/// 動かしてある（描くのと同じ規則）。本文より上・下は、見せている行の前・後の行（ドラッグで動かす・GitHub Issue #460）。
pub(crate) fn pane_index_at(
    (pane, metrics, advances): (PixelRect, GlyphMetrics, &CharAdvances),
    value: &str,
    shown: (usize, Option<usize>),
    (x, y): (i64, i64),
) -> (usize, usize) {
    let columns = text_columns(pane, metrics);
    let limit = usize::try_from(ClientBands::body_lines(pane.size().height(), metrics))
        .unwrap_or(usize::MAX);
    let first = editing_first(value, columns, limit, shown);
    let (left, top) = body_origin(pane, metrics);
    let line = i64::from(ClientBands::body_line(metrics).max(1));
    let rows = y.saturating_sub(i64::from(top)).div_euclid(line);
    let row = i64::try_from(first)
        .unwrap_or(i64::MAX)
        .saturating_add(rows)
        .max(0);
    // 本文の左からの画素を、送り幅と同じ細かさで（UI が測った字の幅で近い境目を探す・GitHub Issue #462）。
    let along = x
        .saturating_sub(i64::from(left))
        .max(0)
        .saturating_mul(i64::from(SUBPIXELS));
    let index = index_at(
        value,
        columns,
        (
            usize::try_from(row).unwrap_or(0),
            u32::try_from(along).unwrap_or(u32::MAX),
        ),
        |character| advances.of(character),
    );
    (index, first)
}

/// `value` を幅 `columns` で折り返したときの、`index` 字目の前の位置：折り返した何行目かと、その行の左からの画素
/// （UI が測った字の送り幅で・GitHub Issue #462）。
fn place_in_pixels(
    advances: &CharAdvances,
    (value, columns): (&str, u32),
    index: usize,
) -> (usize, u32) {
    let (row, head) = caret_piece(value, columns, index);
    (row, advances.width(&head))
}

/// 直している間に見せる最初の行：`scroll` 行目から、カーソルの棒 `caret` の行が `limit` 行の中に入るよう動かし、
/// 最後の画面に収める（描くのと押した所を探すのとで同じ・GitHub Issue #458・#460）。
fn editing_first(
    value: &str,
    columns: u32,
    limit: usize,
    (scroll, caret): (usize, Option<usize>),
) -> usize {
    let total = wrap(value, columns, usize::MAX).lines().len();
    let scroll = caret.map_or(scroll, |caret| {
        let (row, _) = caret_place(value, columns, caret);
        scroll
            .min(row)
            .max(row.saturating_add(1).saturating_sub(limit))
    });
    scroll.min(total.saturating_sub(limit))
}

/// 本文の字の左端と、最初の行の上端（窓の中）。
fn body_origin(pane: PixelRect, metrics: GlyphMetrics) -> (u32, u32) {
    (
        pane.origin().x().saturating_add(metrics.px(TEXT_LEFT)),
        pane.origin()
            .y()
            .saturating_add(metrics.px(DETAIL_RULE))
            .saturating_add(TableLayout::row_height_for(metrics))
            .saturating_add(metrics.px(DETAIL_PADDING)),
    )
}

/// 本文の字を書く幅（画素）。
fn text_width(pane: PixelRect, metrics: GlyphMetrics) -> u32 {
    pane.size()
        .width()
        .saturating_sub(metrics.px(TEXT_LEFT))
        .saturating_sub(metrics.px(RIGHT_PADDING))
}

/// 本文を折り返す幅（半角の字数）。
fn text_columns(pane: PixelRect, metrics: GlyphMetrics) -> u32 {
    text_width(pane, metrics) / metrics.half_width().max(1)
}

/// 見出しの行の補助の情報。行（と見出しがあれば列）の番号、値があれば文字数とファイルの文字コード `encoding` での
/// バイト数（Issue #59）。全角の空きで区切る。
fn aux_text(
    place: DetailPlace<'_>,
    content: DetailContent<'_>,
    column: &str,
    encoding: TextEncoding,
) -> String {
    let number = format!("{} 行目", group_digits(place.record().saturating_add(1)));
    // 列の数が見出しと違う行は、そう言う（GitHub Issue #317・専門家ペルソナの 4 巡目）。
    let row = match place.shape() {
        // 見出しより多ければ、消し方を添える（GitHub Issue #413）。
        Some((cells, expected)) if cells > expected => format!(
            "{number}（{cells} 列・見出しは {expected} 列 · Ctrl+Shift+P「余計なセル」で消せる）"
        ),
        Some((cells, expected)) => {
            format!("{number}（列の数が違う：{cells} 列・見出しは {expected} 列）")
        }
        None => number,
    };
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
            // 表の印（□ など）を言葉にする（豆腐に見えて驚かない・GitHub Issue #387）。
            let legend = mark_legend(value);
            let marks = if legend.is_empty() {
                legend
            } else {
                format!("　{legend}")
            };
            // 必ず表せない字があれば、バイト数の代わりにそう言う（保存で止まる・GitHub Issue #395）。
            let bytes = encoding.surely_unrepresentable(value).map_or_else(
                || {
                    format!(
                        "{} で {} バイト",
                        byte_encoding_name(encoding),
                        group_digits(encoding.encoded_length(value))
                    )
                },
                |character| {
                    // 字体に無い字（絵文字）は豆腐になるので、符号の位置も添える。
                    format!(
                        "{} で表せない字「{character}」U+{:04X} あり",
                        byte_encoding_name(encoding),
                        u32::from(character)
                    )
                },
            );
            format!(
                "{position}　{} 文字　{bytes}{json}{marks}",
                group_digits(characters)
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
