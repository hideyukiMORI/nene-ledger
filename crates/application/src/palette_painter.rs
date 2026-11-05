//! コマンドパレットを描画命令に変える（Issue #41・デザインの相談の結果）。

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use nene_ledger_core::color::Color;
use nene_ledger_core::dialect_candidate::DialectCandidate;
use nene_ledger_core::display_width::display_columns;
use nene_ledger_core::draw_command::DrawCommand;
use nene_ledger_core::glyph_metrics::GlyphMetrics;
use nene_ledger_core::palette::Palette;
use nene_ledger_core::pixel_point::PixelPoint;
use nene_ledger_core::pixel_rect::PixelRect;
use nene_ledger_core::pixel_size::PixelSize;
use nene_ledger_core::text_role::TextRole;
use nene_ledger_core::theme_catalog::{COUNT, at};

use crate::command_id::CommandId;
use crate::command_palette::CommandPalette;
use crate::detail_pane::DetailPane;
use crate::go_to_target::GoToTarget;
use crate::palette_item::PaletteItem;
use crate::palette_layout::PaletteLayout;
use crate::palette_mode::PaletteMode;
use crate::palette_search::key_list;
use crate::reading_aspect::ReadingAspect;
use crate::reading_choice::ReadingChoice;

/// 選んでいる候補の左の帯の太さ（詳細ペインの帯と同じ）。
const BAR: u32 = 3;
/// 候補の文字の左の空き（帯の後ろ）。
const TEXT_INSET: u32 = 12;
/// 上辺の強調の線の太さ（影の代わり）。
const ACCENT: u32 = 2;
/// 配色の見本の 1 辺と間。
const SWATCH: u32 = 12;
const SWATCH_GAP: u32 = 2;

/// コマンドパレットの描画命令を、描く順に積む。色はパレットを開いたときの配色に固定する（配色を試しても動かない）。
#[derive(Debug)]
pub(crate) struct PalettePainter<'state> {
    palette: &'state CommandPalette,
    layout: PaletteLayout,
    metrics: GlyphMetrics,
    colors: Palette,
    commands: Vec<DrawCommand>,
}

impl<'state> PalettePainter<'state> {
    pub(crate) fn new(
        palette: &'state CommandPalette,
        layout: PaletteLayout,
        metrics: GlyphMetrics,
        colors: Palette,
    ) -> Self {
        Self {
            palette,
            layout,
            metrics,
            colors,
            commands: Vec::new(),
        }
    }

    /// 面・検索欄・試しの案内・候補の行の命令。`detail` は詳細ペインの開閉（開閉の操作の名前が変わる）。
    pub(crate) fn paint(mut self, detail: DetailPane) -> Vec<DrawCommand> {
        self.frame();
        self.field();
        self.hint();
        let items = self.palette.items();
        if items.is_empty() {
            self.no_match();
        }
        for slot in 0..self.layout.visible() {
            let index = self.layout.first().saturating_add(slot);
            if let Some(&item) = items.get(index) {
                self.row(slot, item, detail);
            }
        }
        self.commands
    }

    /// 表から浮かせる外側の空き（背景）・縁（窓の縁）・面・上辺の強調の線。下の表は暗くしない。
    fn frame(&mut self) {
        let panel = self.layout.panel();
        self.fill(grown(panel, self.px(2)), self.colors.background());
        self.fill(grown(panel, self.px(1)), self.colors.border());
        self.fill(panel, self.colors.gutter());
        let top = PixelRect::new(
            panel.origin(),
            PixelSize::new(panel.size().width(), self.px(ACCENT)),
        );
        self.fill(top, self.colors.cursor());
    }

    /// 検索欄。背景で塗り、下辺は強調の色。打った字の後ろに入力の印。空なら案内文。右端に該当と全部の数。
    fn field(&mut self) {
        let field = self.layout.field();
        self.fill(field, self.colors.background());
        let line_top = bottom(field).saturating_sub(self.px(1));
        self.fill(
            PixelRect::new(
                PixelPoint::new(field.origin().x(), line_top),
                PixelSize::new(field.size().width(), self.px(1)),
            ),
            self.colors.cursor(),
        );
        let left = field.origin().x().saturating_add(self.px(TEXT_INSET));
        let query = self.palette.query();
        if query.is_empty() {
            let placeholder = match self.palette.mode() {
                PaletteMode::Commands => "操作の名前を英字かローマ字で（例: theme, haishoku）",
                PaletteMode::GoTo => "行の番号（例: 1200、列も選ぶなら 1200,3）",
                PaletteMode::Keys => "キーか操作の名前で絞る（例: ctrl, shift, kopi-）",
                PaletteMode::Themes { .. } => "配色の名前を英字で（例: solarized）",
                PaletteMode::Reading {
                    aspect: ReadingAspect::Delimiter,
                    ..
                } => "英字で絞るか、区切りの字を 1 つ打つ（例: tab, #）",
                PaletteMode::Reading { .. } => "英字で絞る（例: sjis, tab）",
                PaletteMode::Candidates { .. } => {
                    "英字で絞る（例: tab, comma）。上から判別の点数の順"
                }
            };
            self.text_at(
                from_left(field, left),
                String::from(placeholder),
                self.colors.gutter_text(),
            );
        } else {
            self.text_at(
                from_left(field, left),
                String::from(query),
                self.colors.text(),
            );
        }
        let caret_x = left.saturating_add(self.width_of(query));
        let caret_top = field.origin().y().saturating_add(
            field
                .size()
                .height()
                .saturating_sub(self.metrics.line_height())
                / 2,
        );
        self.fill(
            PixelRect::new(
                PixelPoint::new(caret_x, caret_top),
                PixelSize::new(self.px(2), self.metrics.line_height()),
            ),
            self.colors.cursor(),
        );
        let Some(total) = self.total() else {
            return;
        };
        let count = format!("{} / {total}", self.palette.items().len());
        self.right_text(field, count);
    }

    /// 段の候補の全部の数（検索欄の右端の「該当 / 全部」）。飛ぶ先の段は数を出さない。
    fn total(&self) -> Option<usize> {
        match self.palette.mode() {
            PaletteMode::GoTo => None,
            PaletteMode::Commands => Some(CommandId::ALL.len()),
            PaletteMode::Keys => Some(key_list("").len()),
            PaletteMode::Themes { .. } => Some(COUNT),
            PaletteMode::Reading { aspect, .. } => Some(aspect.options().len()),
            PaletteMode::Candidates { .. } => Some(self.palette.candidates().len()),
        }
    }

    /// 配色の段の、試しの案内（試している配色の名前と、決める・戻すのキー）。
    fn hint(&mut self) {
        let Some(hint) = self.layout.hint() else {
            return;
        };
        let text = match self.palette.items().get(self.palette.selected()) {
            Some(&PaletteItem::Theme(position)) => {
                format!(
                    "試し中: {}　Enter で決める・Esc で戻す",
                    at(position).name()
                )
            }
            Some(&PaletteItem::Reading(choice)) => {
                format!("試し中: {}　Enter で開き直す・Esc で戻す", choice.name())
            }
            Some(&PaletteItem::Candidate(candidate)) => format!(
                "試し中: {}　Enter で開き直す・Esc で戻す",
                ReadingChoice::dialect(candidate.dialect()).name()
            ),
            Some(&PaletteItem::Command(_) | &PaletteItem::Key(_)) | None => String::new(),
        };
        self.text_at(
            from_left(hint, hint.origin().x()),
            text,
            self.colors.gutter_text(),
        );
    }

    /// 候補が無いときの知らせ（Backspace で戻れる）。
    fn no_match(&mut self) {
        if self.palette.mode() == PaletteMode::GoTo {
            self.go_to_line();
            return;
        }
        let kind = match self.palette.mode() {
            PaletteMode::GoTo | PaletteMode::Commands => "操作",
            PaletteMode::Keys => "キー",
            PaletteMode::Themes { .. } => "配色",
            PaletteMode::Reading { aspect, .. } => aspect.noun(),
            PaletteMode::Candidates { .. } => "候補",
        };
        let text = format!("“{}” に合う{kind}は無い", self.palette.query());
        let row = self.layout.row(0);
        let left = row.origin().x().saturating_add(self.px(TEXT_INSET));
        self.text_at(from_left(row, left), text, self.colors.gutter_text());
    }

    /// 飛ぶ先の段の 1 行。打った番号が読めれば飛ぶ先、読めなければ書き方の案内（空なら何も出さない）。
    fn go_to_line(&mut self) {
        let query = self.palette.query();
        if query.is_empty() {
            return;
        }
        let text = GoToTarget::parse(query).map_or_else(
            || String::from("数字で行の番号を（列も選ぶなら 1200,3）"),
            |target| format!("Enter で {} へ", target.describe()),
        );
        let row = self.layout.row(0);
        let left = row.origin().x().saturating_add(self.px(TEXT_INSET));
        self.text_at(from_left(row, left), text, self.colors.gutter_text());
    }

    /// 候補の 1 行。選んでいれば選択の色で塗り、左に強調の色の帯（色だけに頼らない）。
    fn row(&mut self, slot: usize, item: PaletteItem, detail: DetailPane) {
        let row = self.layout.row(slot);
        if self.layout.first().saturating_add(slot) == self.palette.selected() {
            self.fill(row, self.colors.selection());
            self.fill(
                PixelRect::new(
                    row.origin(),
                    PixelSize::new(self.px(BAR), row.size().height()),
                ),
                self.colors.cursor(),
            );
        }
        let mut left = row.origin().x().saturating_add(self.px(TEXT_INSET));
        let (name, aside) = match item {
            PaletteItem::Command(command) => (
                String::from(command.name(detail)),
                String::from(command.keys().unwrap_or("")),
            ),
            PaletteItem::Reading(choice) => (
                choice.name(),
                String::from(if self.is_current(choice) {
                    "今の読み方"
                } else {
                    ""
                }),
            ),
            PaletteItem::Candidate(candidate) => {
                let choice = ReadingChoice::dialect(candidate.dialect());
                let aside = candidate_aside(candidate, self.is_current(choice));
                (choice.name(), aside)
            }
            PaletteItem::Key(guide) => (String::from(guide.name()), String::from(guide.keys())),
            PaletteItem::Theme(position) => {
                left = self.swatches(row, left, position);
                let seed = at(position);
                let appearance = match seed.appearance() {
                    nene_ledger_core::appearance::Appearance::Light => "ライト",
                    nene_ledger_core::appearance::Appearance::Dark => "ダーク",
                };
                (String::from(seed.name()), String::from(appearance))
            }
        };
        // 名前は右の字（キー・印・点数）に重ねない。長ければ手前で切る。
        let reserved = self
            .width_of(&aside)
            .saturating_add(self.px(TEXT_INSET).saturating_mul(2));
        let name_area = from_left(row, left);
        self.text_at(
            PixelRect::new(
                name_area.origin(),
                PixelSize::new(
                    name_area.size().width().saturating_sub(reserved),
                    name_area.size().height(),
                ),
            ),
            name,
            self.colors.text(),
        );
        self.right_text(row, aside);
    }

    /// 読み方の候補 `choice` が、今の読み方（読み方を選ぶ段・判別の候補を選ぶ段で印を付ける）か。
    fn is_current(&self, choice: ReadingChoice) -> bool {
        match self.palette.mode() {
            PaletteMode::Reading { current, .. } | PaletteMode::Candidates { current } => {
                current == choice
            }
            PaletteMode::Commands
            | PaletteMode::GoTo
            | PaletteMode::Keys
            | PaletteMode::Themes { .. } => false,
        }
    }

    /// 配色の 3 色（背景・前景・強調）の見本を並べ、その右の位置を返す。行の中では色を変えない。
    fn swatches(&mut self, row: PixelRect, left: u32, position: usize) -> u32 {
        let seed = at(position);
        let (swatch, inner) = (self.px(SWATCH), self.px(1));
        let top = row
            .origin()
            .y()
            .saturating_add(row.size().height().saturating_sub(swatch) / 2);
        let mut x = left;
        for color in [seed.background(), seed.foreground(), seed.accent()] {
            self.fill(
                PixelRect::new(PixelPoint::new(x, top), PixelSize::new(swatch, swatch)),
                self.colors.border(),
            );
            self.fill(
                PixelRect::new(
                    PixelPoint::new(x.saturating_add(inner), top.saturating_add(inner)),
                    PixelSize::new(
                        swatch.saturating_sub(inner.saturating_mul(2)),
                        swatch.saturating_sub(inner.saturating_mul(2)),
                    ),
                ),
                color,
            );
            x = x.saturating_add(swatch).saturating_add(self.px(SWATCH_GAP));
        }
        x.saturating_add(self.px(TEXT_INSET) / 2)
    }

    /// 行 `row` の右端に寄せて補助の字で書く（幅は近似）。
    fn right_text(&mut self, row: PixelRect, text: String) {
        let right = row
            .origin()
            .x()
            .saturating_add(row.size().width())
            .saturating_sub(self.px(TEXT_INSET));
        let left = right.saturating_sub(self.width_of(&text));
        self.text_at(from_left(row, left), text, self.colors.gutter_text());
    }

    /// 固定の画素 `pixels` に画面の倍率を掛ける（ADR 0020 の 5）。
    fn px(&self, pixels: u32) -> u32 {
        self.metrics.px(pixels)
    }

    /// `text` の幅の近似（半角 1・全角 2 の字数 × 半角の幅）。
    fn width_of(&self, text: &str) -> u32 {
        display_columns(text).saturating_mul(self.metrics.half_width())
    }

    /// 矩形 `area` の左端から縦の真ん中で書く。矩形の外は切る。
    fn text_at(&mut self, area: PixelRect, text: String, color: Color) {
        let spare = area
            .size()
            .height()
            .saturating_sub(self.metrics.line_height());
        self.commands.push(DrawCommand::Text {
            baseline: PixelPoint::new(
                area.origin().x(),
                area.origin()
                    .y()
                    .saturating_add(spare / 2)
                    .saturating_add(self.metrics.ascent()),
            ),
            text,
            color,
            clip: area,
            role: TextRole::Chrome,
        });
    }

    fn fill(&mut self, rect: PixelRect, color: Color) {
        self.commands.push(DrawCommand::Fill { rect, color });
    }
}

/// 判別の候補の右に添える字。今の読み方の印・点数（千分率を百分率に）・いちばん多い列の数（OPEN-02）。
fn candidate_aside(candidate: DialectCandidate, current: bool) -> String {
    let score = format!("{}% · {} 列", candidate.score() / 10, candidate.columns());
    if current {
        format!("今の読み方 · {score}")
    } else {
        score
    }
}

/// 行 `row` のうち、左端 `left` から行の右端までの矩形。
fn from_left(row: PixelRect, left: u32) -> PixelRect {
    let right = row.origin().x().saturating_add(row.size().width());
    PixelRect::new(
        PixelPoint::new(left, row.origin().y()),
        PixelSize::new(right.saturating_sub(left), row.size().height()),
    )
}

/// `rect` を四方に `by` 広げた矩形。
fn grown(rect: PixelRect, by: u32) -> PixelRect {
    PixelRect::new(
        PixelPoint::new(
            rect.origin().x().saturating_sub(by),
            rect.origin().y().saturating_sub(by),
        ),
        PixelSize::new(
            rect.size().width().saturating_add(by.saturating_mul(2)),
            rect.size().height().saturating_add(by.saturating_mul(2)),
        ),
    )
}

fn bottom(rect: PixelRect) -> u32 {
    rect.origin().y().saturating_add(rect.size().height())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intent::Intent;
    use crate::theme_state::ThemeState;
    use crate::window_view::WindowView;
    use alloc::vec;
    use nene_ledger_core::window_placement::WindowPlacement;

    fn view() -> WindowView {
        WindowView::new(
            PixelSize::new(1024, 640),
            GlyphMetrics::new(8, 20, 15),
            WindowPlacement::Normal,
            None,
        )
    }

    fn colors() -> Palette {
        ThemeState::at(1).palette()
    }

    fn texts(commands: &[DrawCommand]) -> Vec<&str> {
        commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Text { text, .. } => Some(text.as_str()),
                DrawCommand::Fill { .. } | DrawCommand::Line { .. } => None,
            })
            .collect()
    }

    fn fills(commands: &[DrawCommand], wanted: Color) -> Vec<PixelRect> {
        commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Fill { rect, color } if *color == wanted => Some(*rect),
                DrawCommand::Fill { .. } | DrawCommand::Text { .. } | DrawCommand::Line { .. } => {
                    None
                }
            })
            .collect()
    }

    #[test]
    fn an_empty_palette_shows_the_hint_every_command_and_the_count() {
        let palette = CommandPalette::open();
        let commands = palette.paint(view(), colors(), DetailPane::Open);
        let shown = texts(&commands);
        assert!(shown.contains(&"操作の名前を英字かローマ字で（例: theme, haishoku）"));
        assert!(shown.contains(&"31 / 31"));
        assert!(shown.contains(&"詳細ペインを閉じる") && shown.contains(&"Ctrl+Shift+U"));
        assert!(shown.contains(&"文字を大きくする") && shown.contains(&"Ctrl+Plus"));
        // 選んでいる最初の行は選択の色で塗り、左に強調の色の 3px の帯。
        let first_row = PaletteLayout::new(view(), &palette).row(0);
        assert_eq!(fills(&commands, colors().selection()), vec![first_row]);
        assert!(
            fills(&commands, colors().cursor())
                .contains(&PixelRect::new(first_row.origin(), PixelSize::new(3, 28)))
        );
        // 文字はどれも窓の部品の大きさ。
        assert!(commands.iter().all(|command| match command {
            DrawCommand::Text { role, .. } => *role == TextRole::Chrome,
            DrawCommand::Fill { .. } | DrawCommand::Line { .. } => true,
        }));
    }

    #[test]
    fn the_key_list_shows_names_and_keys_of_both_kinds() {
        let mut palette = CommandPalette::open();
        for character in "keys".chars() {
            palette.handle(Intent::Type(character), 0);
        }
        palette.handle(Intent::Confirm, 0);
        let total = crate::palette_search::key_list("").len();
        let everything = palette.paint(view(), colors(), DetailPane::Closed);
        let listed = texts(&everything);
        assert!(listed.contains(&format!("{total} / {total}").as_str()));
        assert!(listed.contains(&"表の先頭に移る") && listed.contains(&"Ctrl+Home"));
        for character in "esc".chars() {
            palette.handle(Intent::Type(character), 0);
        }
        let narrowed = palette.paint(view(), colors(), DetailPane::Closed);
        let found = texts(&narrowed);
        assert!(found.contains(&"やめる・元に戻す") && found.contains(&"Esc"));
        for character in "zz".chars() {
            palette.handle(Intent::Type(character), 0);
        }
        let nothing = palette.paint(view(), colors(), DetailPane::Closed);
        assert!(texts(&nothing).contains(&"“esczz” に合うキーは無い"));
    }

    #[test]
    fn a_query_without_matches_says_so() {
        let mut palette = CommandPalette::open();
        for character in "xyz".chars() {
            palette.handle(Intent::Type(character), 0);
        }
        let commands = palette.paint(view(), colors(), DetailPane::Closed);
        let shown = texts(&commands);
        assert!(shown.contains(&"“xyz” に合う操作は無い"));
        assert!(shown.contains(&"0 / 31") && shown.contains(&"xyz"));
        assert!(fills(&commands, colors().selection()).is_empty());
    }

    /// 判別の候補の段（カンマ 87%・6 列が今の読み方、タブ・引用なし 12%・1 列）。
    fn candidate_palette() -> CommandPalette {
        use nene_ledger_core::delimiter::Delimiter;
        use nene_ledger_core::dialect::Dialect;
        use nene_ledger_core::quoting::Quoting;
        let mut palette = CommandPalette::open();
        let comma = DialectCandidate::new(Dialect::rfc4180(), 874, 6);
        let tab = DialectCandidate::new(
            Dialect::new(Delimiter::TAB, Quoting::Plain, b'"').unwrap(),
            120,
            1,
        );
        palette.enter_candidates(vec![comma, tab], ReadingChoice::dialect(comma.dialect()));
        palette
    }

    #[test]
    fn the_candidate_list_shows_scores_columns_and_the_current_mark() {
        let mut palette = candidate_palette();
        let commands = palette.paint(view(), colors(), DetailPane::Open);
        let shown = texts(&commands);
        assert!(shown.contains(&"英字で絞る（例: tab, comma）。上から判別の点数の順"));
        assert!(shown.contains(&"2 / 2"));
        assert!(shown.contains(&"カンマ区切り・\"\" で引用符（RFC 4180）"));
        assert!(shown.contains(&"今の読み方 · 87% · 6 列"));
        assert!(shown.contains(&"タブ区切り・引用なし"));
        assert!(shown.contains(&"12% · 1 列"));
        assert!(shown.contains(
            &"試し中: カンマ区切り・\"\" で引用符（RFC 4180）　Enter で開き直す・Esc で戻す"
        ));
        for character in "zz".chars() {
            palette.handle(Intent::Type(character), 0);
        }
        let nothing = palette.paint(view(), colors(), DetailPane::Open);
        assert!(texts(&nothing).contains(&"“zz” に合う候補は無い"));
    }

    #[test]
    fn a_long_name_is_clipped_before_the_aside() {
        let palette = candidate_palette();
        let commands = palette.paint(view(), colors(), DetailPane::Open);
        // 名前の描く矩形は、右の字（今の読み方の印と点数）の手前で終わる（重ねない）。
        let first_row = PaletteLayout::new(view(), &palette).row(0);
        let name_clip = commands
            .iter()
            .find_map(|command| match command {
                DrawCommand::Text { text, clip, .. } if text.starts_with("カンマ区切り") => {
                    Some(*clip)
                }
                DrawCommand::Text { .. } | DrawCommand::Fill { .. } | DrawCommand::Line { .. } => {
                    None
                }
            })
            .unwrap();
        let aside_width = display_columns("今の読み方 · 87% · 6 列") * 8;
        assert!(
            name_clip.origin().x() + name_clip.size().width() + aside_width
                < first_row.origin().x() + first_row.size().width()
        );
    }

    #[test]
    fn the_theme_list_shows_swatches_and_the_trial_hint_in_the_frozen_colors() {
        let mut palette = CommandPalette::open();
        for character in "theme".chars() {
            palette.handle(Intent::Type(character), 0);
        }
        palette.handle(Intent::Confirm, 0);
        let frozen = ThemeState::at(40).palette();
        let commands = palette.paint(view(), frozen, DetailPane::Open);
        let shown = texts(&commands);
        assert!(shown.contains(&"試し中: ubuntu-aubergine　Enter で決める・Esc で戻す"));
        assert!(shown.contains(&"ubuntu-aubergine") && shown.contains(&"ダーク"));
        assert!(shown.contains(&"neutral-light") && shown.contains(&"ライト"));
        // 見本は配色ごとに 3 色。面はパレットを開いたときの色（渡した色）で塗る。
        let seed = at(0);
        assert!(!fills(&commands, seed.accent()).is_empty());
        let panel = PaletteLayout::new(view(), &palette).panel();
        assert!(fills(&commands, frozen.gutter()).contains(&panel));
    }
}
