//! コマンドパレットを描画命令に変える（Issue #41・デザインの相談の結果）。

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use nene_ledger_core::color::Color;
use nene_ledger_core::dialect_candidate::DialectCandidate;
use nene_ledger_core::display_width::display_columns;
use nene_ledger_core::draw_command::DrawCommand;
use nene_ledger_core::find_options::FindOptions;
use nene_ledger_core::find_query::FindQuery;
use nene_ledger_core::glyph_metrics::GlyphMetrics;
use nene_ledger_core::palette::Palette;
use nene_ledger_core::pixel_point::PixelPoint;
use nene_ledger_core::pixel_rect::PixelRect;
use nene_ledger_core::pixel_size::PixelSize;
use nene_ledger_core::text_encoding::TextEncoding;
use nene_ledger_core::text_role::TextRole;
use nene_ledger_core::theme_catalog::{COUNT, at};

use crate::command_id::CommandId;
use crate::command_palette::CommandPalette;
use crate::detail_pane::DetailPane;
use crate::find_toggle::FindToggle;
use crate::go_to_target::GoToTarget;
/// 操作の行の右の字：キー（最近使った操作は見出しで分かるので印を付けない・GitHub Issue #420）。
fn command_aside(command: CommandId) -> String {
    String::from(command.keys().unwrap_or(""))
}

#[cfg(test)]
use crate::guide_note::GuideNote;
use crate::note_tone::NoteTone;
use crate::palette_heading::PaletteHeading;
use crate::palette_item::PaletteItem;
use crate::palette_layout::{CHIP_MARK_COLUMNS, CHIP_PAD, FIND_CHIPS, PaletteLayout};
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
    note: Option<&'state str>,
    tone: NoteTone,
    lines: &'state [(String, bool)],
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
            note: None,
            tone: NoteTone::Prompt,
            lines: &[],
            commands: Vec::new(),
        }
    }

    /// 置換の段の 1 行目のキーの案内の代わりに出す、全部の置換の短い 1 行（件数と確かめ・GitHub Issue #189）。
    pub(crate) fn with_note(self, note: Option<&'state str>) -> Self {
        Self { note, ..self }
    }

    /// 知らせの種類（色・GitHub Issue #305）。
    pub(crate) fn with_tone(self, tone: NoteTone) -> Self {
        Self { tone, ..self }
    }

    /// 知らせの種類の色：確かめは強調、済んだは緑、注意は赤。
    fn tone_color(&self, tone: NoteTone) -> Color {
        match tone {
            NoteTone::Prompt => self.colors.cursor(),
            NoteTone::Done => self.colors.success(),
            NoteTone::Problem => self.colors.danger(),
        }
    }

    /// 置換の段の条件の札の代わりに出す、全部の置換の内訳の行（注意は `true`・GitHub Issue #289）。
    pub(crate) fn with_lines(self, lines: &'state [(String, bool)]) -> Self {
        Self { lines, ..self }
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
    /// 置換の段は欄を 2 つ縦に並べ、左に名札（「探す」「置換後」）を付け、打っている欄だけ下辺を強調の色にして入力の印を
    /// 置く（GitHub Issue #287）。全部選んだ探す欄は、字の後ろを選択の色で塗る。
    fn field(&mut self) {
        let mode = self.palette.mode();
        let Some(second) = self.layout.second() else {
            let query = self.palette.field_text();
            self.input(
                self.layout.field(),
                (None, query, placeholder(mode)),
                (true, self.palette.whole_selected()),
            );
            let Some(total) = self.total() else {
                return;
            };
            // 見出しと、見出しの下に重ねて出す操作（最近・よく使う）は数えない（GitHub Issue #420）。
            let shown = if self.palette.mode() == PaletteMode::Commands
                && self.palette.query().trim().is_empty()
            {
                total
            } else {
                self.palette.items().len()
            };
            let count = format!("{shown} / {total}");
            self.right_text(self.layout.field(), count);
            return;
        };
        let with = mode == (PaletteMode::Replace { with: true });
        let label = Some(self.label_width());
        self.input(
            self.layout.field(),
            (
                label.map(|width| ("探す", width)),
                self.palette.query(),
                "探す文字",
            ),
            (!with, self.palette.whole_selected()),
        );
        self.input(
            second,
            (
                label.map(|width| ("置換後", width)),
                self.palette.replacement(),
                "置き換える文字（空なら当たった所を消す）",
            ),
            (with, false),
        );
    }

    /// 名札の列の幅（長いほうの名札と、字との空き）。
    fn label_width(&self) -> u32 {
        self.width_of("置換後")
            .max(self.width_of("探す"))
            .saturating_add(self.px(TEXT_INSET).saturating_mul(2))
    }

    /// 入力の欄 1 つ。`label` は名札とその列の幅、`text` は中身、`hint` は空のときの案内。`active` なら下辺を強調の色に
    /// して入力の印を置き、そうでなければ下辺は縁の色。`selected` なら字の後ろを選択の色で塗る。
    fn input(
        &mut self,
        field: PixelRect,
        (label, text, hint): (Option<(&str, u32)>, &str, &str),
        (active, selected): (bool, bool),
    ) {
        self.fill(field, self.colors.background());
        let line_top = bottom(field).saturating_sub(self.px(1));
        self.fill(
            PixelRect::new(
                PixelPoint::new(field.origin().x(), line_top),
                PixelSize::new(field.size().width(), self.px(1)),
            ),
            if active {
                self.colors.cursor()
            } else {
                self.colors.border()
            },
        );
        let left = self.label(field, label, active);
        if text.is_empty() {
            self.text_at(
                from_left(field, left),
                String::from(hint),
                self.colors.gutter_text(),
            );
        }
        // 欄より長い文字列（深いフォルダの道筋など）は、打っている後ろの側を見せる（頭は … で略す）。
        let room = field
            .size()
            .width()
            .saturating_sub(left.saturating_sub(field.origin().x()))
            .saturating_sub(self.px(TEXT_INSET))
            .saturating_sub(self.px(2));
        let shown = self.tail_within(text, room);
        let line_height = self.metrics.line_height();
        let caret_top = field
            .origin()
            .y()
            .saturating_add(field.size().height().saturating_sub(line_height) / 2);
        if selected && !text.is_empty() {
            self.fill(
                PixelRect::new(
                    PixelPoint::new(left, caret_top),
                    PixelSize::new(self.width_of(&shown), line_height),
                ),
                self.colors.selection(),
            );
        }
        if !text.is_empty() {
            self.text_at(from_left(field, left), shown.clone(), self.colors.text());
        }
        if active {
            let caret_x = left.saturating_add(self.width_of(&shown));
            self.fill(
                PixelRect::new(
                    PixelPoint::new(caret_x, caret_top),
                    PixelSize::new(self.px(2), line_height),
                ),
                self.colors.cursor(),
            );
        }
    }

    /// 欄の左の名札（打っている欄は濃い字）。字を書き始める左端を返す（名札が無ければ内側の余白の後ろ）。
    fn label(&mut self, field: PixelRect, label: Option<(&str, u32)>, active: bool) -> u32 {
        let inset = field.origin().x().saturating_add(self.px(TEXT_INSET));
        let Some((name, width)) = label else {
            return inset;
        };
        let color = if active {
            self.colors.text()
        } else {
            self.colors.gutter_text()
        };
        self.text_at(from_left(field, inset), String::from(name), color);
        field.origin().x().saturating_add(width)
    }

    /// 段の候補の全部の数（検索欄の右端の「該当 / 全部」）。飛ぶ先の段は数を出さない。
    fn total(&self) -> Option<usize> {
        match self.palette.mode() {
            PaletteMode::GoTo
            | PaletteMode::Find
            | PaletteMode::Replace { .. }
            | PaletteMode::About
            | PaletteMode::SaveAs { .. } => None,
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
                    "Enter で決める · Esc で戻す · 試し中: {}",
                    at(position).name()
                )
            }
            Some(&PaletteItem::Reading(choice)) => {
                format!("Enter で開き直す · Esc で戻す · 試し中: {}", choice.name())
            }
            Some(&PaletteItem::Candidate(candidate)) => format!(
                "Enter で開き直す · Esc で戻す · 試し中: {}",
                ReadingChoice::dialect(candidate.dialect()).name()
            ),
            Some(
                &PaletteItem::Command(_)
                | &PaletteItem::Key(_)
                | &PaletteItem::About(_)
                | &PaletteItem::Heading(_),
            )
            | None => String::new(),
        };
        // 候補の行と同じ左の余白（GitHub Issue #312・前は 12 px 左にずれていた）。
        let left = hint.origin().x().saturating_add(self.px(TEXT_INSET));
        self.text_at(from_left(hint, left), text, self.colors.gutter_text());
    }

    /// 候補が無いときの知らせ（Backspace で戻れる）。
    fn no_match(&mut self) {
        if self.palette.mode() == PaletteMode::GoTo {
            self.go_to_line();
            return;
        }
        if let PaletteMode::Find | PaletteMode::Replace { .. } = self.palette.mode() {
            self.find_line();
            return;
        }
        if let PaletteMode::SaveAs { overwrite } = self.palette.mode() {
            self.save_as_line(overwrite);
            return;
        }
        let kind = match self.palette.mode() {
            PaletteMode::GoTo
            | PaletteMode::Find
            | PaletteMode::Replace { .. }
            | PaletteMode::Commands
            | PaletteMode::SaveAs { .. } => "操作",
            PaletteMode::Keys => "キー",
            PaletteMode::About => "行",
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
        // 表の範囲と書き方（打つ前）・範囲の外の番号（注意）は合成ルートが渡す（GitHub Issue #307）。
        if let Some(note) = self.note {
            let color = match self.tone {
                NoteTone::Problem => self.colors.danger(),
                NoteTone::Prompt | NoteTone::Done => self.colors.gutter_text(),
            };
            let row = self.layout.row(0);
            let left = row.origin().x().saturating_add(self.px(TEXT_INSET));
            self.text_at(from_left(row, left), String::from(note), color);
            return;
        }
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

    /// 探す段と置換の段の案内。置換の段は 1 行目に置き換え方のキー（全部の置換の件数と確かめの間はその 1 行・ADR 0030 の
    /// 決定 6・7）。その下に条件の札（[`FIND_CHIPS`]・GitHub Issue #288）。正規表現の式が読めなければ、欄の右端に強調の色で
    /// そう出す（ADR 0031）。行の数は `palette_layout::least_rows` と同じ。
    fn find_line(&mut self) {
        let options = self.palette.find_options();
        if options.regex() && FindQuery::new(self.palette.query(), options).is_broken() {
            self.field_right(String::from("読めない正規表現"), self.colors.cursor());
        }
        // 探す段の知らせ（見つからない・端から続けた）は欄の右端に、その場で（GitHub Issue #291）。列に限って探すなら、
        // 知らせが無い間は欄の右端に名札（GitHub Issue #411）。
        let scope = self.palette.scope().map(String::from);
        if self.palette.mode() == PaletteMode::Find {
            match (self.note, &scope) {
                (Some(note), _) => self.field_right(String::from(note), self.tone_color(self.tone)),
                (None, Some(scope)) => self.field_right(scope.clone(), self.colors.cursor()),
                (None, None) => {}
            }
        }
        if let PaletteMode::Replace { .. } = self.palette.mode() {
            let (text, color) = self.note.map_or_else(
                || match &scope {
                    // 列に限って置き換えるなら、案内の頭に名札を強調の色で（GitHub Issue #411）。
                    Some(scope) => (
                        format!("{scope} · Enter 置き換えて次へ · Alt+A 全部"),
                        self.colors.cursor(),
                    ),
                    None => (
                        String::from("Enter 置き換えて次へ · Alt+A 全部 · Tab 欄を替える"),
                        self.colors.gutter_text(),
                    ),
                },
                |note| (String::from(note), self.tone_color(self.tone)),
            );
            let row = self.layout.row(0);
            let left = row.origin().x().saturating_add(self.px(TEXT_INSET));
            self.text_at(from_left(row, left), text, color);
        }
        // 全部の置換の件数を見せている間は、札の行に内訳を出す（札を替えれば確かめはやめる・GitHub Issue #289）。
        if !self.lines.is_empty() {
            self.breakdown();
            return;
        }
        let Some(chips) = self.layout.chips() else {
            return;
        };
        for (chip, (name, key, toggle)) in chips.into_iter().zip(FIND_CHIPS) {
            self.chip(chip, (name, key), toggle_is_on(options, toggle));
        }
    }

    /// 欄の右端に、余白を空けて右に寄せた文言。
    fn field_right(&mut self, text: String, color: Color) {
        let field = self.layout.field();
        let right = field
            .origin()
            .x()
            .saturating_add(field.size().width())
            .saturating_sub(self.px(TEXT_INSET));
        let left = right.saturating_sub(self.width_of(&text));
        self.text_at(from_left(field, left), text, color);
    }

    /// 全部の置換の内訳の行（2 行目から・注意は強調の色）。1 行目はキーの案内（件数と確かめ）。
    fn breakdown(&mut self) {
        let lines = self.lines;
        let room = self.layout.visible().saturating_sub(1);
        for (slot, (text, strong)) in lines.iter().enumerate().take(room) {
            let row = self.layout.row(slot.saturating_add(1));
            let left = row.origin().x().saturating_add(self.px(TEXT_INSET));
            let color = if *strong {
                self.colors.danger()
            } else {
                self.colors.text()
            };
            self.text_at(from_left(row, left), text.clone(), color);
        }
    }

    /// 条件の札 1 つ。オンは強調の色の縁と選択の色の塗りに ✓ と濃い字、オフは縁の色の枠と薄い字（色だけに頼らず印でも
    /// 見せる・GitHub Issue #288）。キーは札の右に寄せる。
    fn chip(&mut self, chip: PixelRect, (name, key): (&str, &str), on: bool) {
        let (edge, inside, ink) = if on {
            (
                self.colors.cursor(),
                self.colors.selection(),
                self.colors.text(),
            )
        } else {
            (
                self.colors.border(),
                self.colors.background(),
                self.colors.gutter_text(),
            )
        };
        self.fill(chip, edge);
        self.fill(grown_in(chip, self.px(1)), inside);
        let left = chip.origin().x().saturating_add(self.px(CHIP_PAD));
        if on {
            self.text_at(from_left(chip, left), String::from("✓"), ink);
        }
        let name_left =
            left.saturating_add(CHIP_MARK_COLUMNS.saturating_mul(self.metrics.half_width()));
        self.text_at(from_left(chip, name_left), String::from(name), ink);
        let right = chip
            .origin()
            .x()
            .saturating_add(chip.size().width())
            .saturating_sub(self.px(CHIP_PAD));
        let key_left = right.saturating_sub(self.width_of(key));
        self.text_at(
            from_left(chip, key_left),
            String::from(key),
            self.colors.gutter_text(),
        );
    }

    /// 保存先の段の 1 行。上書きを尋ねていればその問い、そうでなければ Enter で保存する案内（ADR 0028 の決定 5）。
    fn save_as_line(&mut self, overwrite: bool) {
        let text = if overwrite {
            String::from("このファイルは既にあります。Enter をもう一度で上書き・Esc でやめる")
        } else {
            String::from("Enter で、この道筋に名前を付けて保存する（方言と文字コードは今のまま）")
        };
        let row = self.layout.row(0);
        let left = row.origin().x().saturating_add(self.px(TEXT_INSET));
        let color = if overwrite {
            self.colors.cursor()
        } else {
            self.colors.gutter_text()
        };
        self.text_at(from_left(row, left), text, color);
    }

    /// 読み方の候補の右の字：今の読み方の印と、文字コードの日本語の注（事務の利用者が選べるように・GitHub Issue #312）。
    fn reading_aside(&self, choice: ReadingChoice) -> String {
        let current = self.is_current(choice).then_some("今の読み方");
        let note = match choice {
            ReadingChoice::Encoding(encoding) => Some(encoding_note(encoding)),
            ReadingChoice::Delimiter(_)
            | ReadingChoice::Quoting { .. }
            | ReadingChoice::Header(_)
            | ReadingChoice::Dialect { .. } => None,
        };
        [current, note]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" · ")
    }

    /// 配色の右の字：明暗と、開いたときの配色の印（GitHub Issue #312）。
    fn theme_aside(&self, position: usize) -> String {
        let appearance = match at(position).appearance() {
            nene_ledger_core::appearance::Appearance::Light => "ライト",
            nene_ledger_core::appearance::Appearance::Dark => "ダーク",
        };
        match self.palette.mode() {
            PaletteMode::Themes { original } if original == position => {
                format!("今の配色 · {appearance}")
            }
            PaletteMode::Themes { .. }
            | PaletteMode::Commands
            | PaletteMode::GoTo
            | PaletteMode::Find
            | PaletteMode::Replace { .. }
            | PaletteMode::Keys
            | PaletteMode::About
            | PaletteMode::Reading { .. }
            | PaletteMode::Candidates { .. }
            | PaletteMode::SaveAs { .. } => String::from(appearance),
        }
    }

    /// 見出しの行：選べないので塗らず、強調の色の字を左の余白をそろえて書く（GitHub Issue #420）。
    fn heading_row(&mut self, row: PixelRect, heading: PaletteHeading) {
        let left = row.origin().x().saturating_add(self.px(TEXT_INSET));
        self.text_at(
            from_left(row, left),
            String::from(heading.name()),
            self.colors.cursor(),
        );
    }

    /// 候補の 1 行。選んでいれば選択の色で塗り、左に強調の色の帯（色だけに頼らない）。
    fn row(&mut self, slot: usize, item: PaletteItem, detail: DetailPane) {
        let row = self.layout.row(slot);
        if let PaletteItem::Heading(heading) = item {
            self.heading_row(row, heading);
            return;
        }
        // このアプリについては読むだけなので、選んだ色を付けない（GitHub Issue #312）。
        if self.layout.first().saturating_add(slot) == self.palette.selected()
            && self.palette.mode() != PaletteMode::About
        {
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
            PaletteItem::Command(command) => {
                (String::from(command.name(detail)), command_aside(command))
            }
            PaletteItem::Reading(choice) => (choice.name(), self.reading_aside(choice)),
            PaletteItem::Candidate(candidate) => {
                let choice = ReadingChoice::dialect(candidate.dialect());
                let aside = candidate_aside(candidate, self.is_current(choice));
                (choice.name(), aside)
            }
            PaletteItem::Key(guide) => (String::from(guide.name()), String::from(guide.keys())),
            PaletteItem::About(line) => (line.text(), String::new()),
            // 上で描いて戻っている。
            PaletteItem::Heading(heading) => (String::from(heading.name()), String::new()),
            PaletteItem::Theme(position) => {
                left = self.swatches(row, left, position);
                (
                    String::from(at(position).name()),
                    self.theme_aside(position),
                )
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
            | PaletteMode::Find
            | PaletteMode::Replace { .. }
            | PaletteMode::Keys
            | PaletteMode::About
            | PaletteMode::SaveAs { .. }
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
    /// `text` が幅 `room` に収まればそのまま、収まらなければ頭を `…` にして後ろの側を収める。
    fn tail_within(&self, text: &str, room: u32) -> String {
        if self.width_of(text) <= room {
            return String::from(text);
        }
        let budget = room.saturating_sub(self.width_of("…"));
        let mut kept: Vec<char> = Vec::new();
        let mut used = 0_u32;
        for character in text.chars().rev() {
            let mut one = [0_u8; 4];
            let next = used.saturating_add(self.width_of(character.encode_utf8(&mut one)));
            if next > budget {
                break;
            }
            used = next;
            kept.push(character);
        }
        core::iter::once('…')
            .chain(kept.into_iter().rev())
            .collect()
    }

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

/// 文字コードの日本語の注（どんなファイルで使われるか・GitHub Issue #312）。
fn encoding_note(encoding: TextEncoding) -> &'static str {
    match encoding {
        TextEncoding::Utf8 => "多くの道具の標準",
        TextEncoding::Utf8WithBom => "Excel で開ける UTF-8",
        TextEncoding::ShiftJis => "Windows・Excel の日本語",
        TextEncoding::EucJp => "古い Unix の日本語",
        TextEncoding::Windows1252 => "欧文の Windows",
        TextEncoding::Utf16LeWithBom
        | TextEncoding::Utf16BeWithBom
        | TextEncoding::Utf16Le
        | TextEncoding::Utf16Be => "Excel の Unicode テキスト",
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
/// 条件 `options` で、札 `toggle` がオンか（「区別する」「セル全体」「正規表現」の側がオン）。
fn toggle_is_on(options: FindOptions, toggle: FindToggle) -> bool {
    match toggle {
        FindToggle::Case => options.match_case(),
        FindToggle::WholeCell => options.whole_cell(),
        FindToggle::Regex => options.regex(),
        FindToggle::Width => options.match_width(),
        FindToggle::Kana => options.match_kana(),
    }
}

/// `rect` を四方から `by` 縮めた矩形。
fn grown_in(rect: PixelRect, by: u32) -> PixelRect {
    PixelRect::new(
        PixelPoint::new(
            rect.origin().x().saturating_add(by),
            rect.origin().y().saturating_add(by),
        ),
        PixelSize::new(
            rect.size().width().saturating_sub(by.saturating_mul(2)),
            rect.size().height().saturating_sub(by.saturating_mul(2)),
        ),
    )
}

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

/// 段 `mode` の欄が空のときの案内。
fn placeholder(mode: PaletteMode) -> &'static str {
    match mode {
        PaletteMode::Commands => "操作の名前で絞る（例: 置換・haishoku・theme）",
        PaletteMode::GoTo => "行の番号（例: 1200、列も選ぶなら 1200,3）",
        PaletteMode::Find => "探す文字（Enter で次・Shift+Enter で前）",
        PaletteMode::Replace { .. } => "探す文字",
        PaletteMode::SaveAs { .. } => "保存先のファイルの道筋（例: /srv/data/表.csv）",
        PaletteMode::Keys => "キーか操作の名前で絞る（例: ctrl, shift, kopi-）",
        PaletteMode::About => "このアプリについて（Esc で閉じる）",
        PaletteMode::Themes { .. } => "配色の名前を英字で（例: solarized）",
        PaletteMode::Reading {
            aspect: ReadingAspect::Delimiter,
            ..
        } => "英字で絞るか、区切りの字を 1 つ打つ（例: tab, #）",
        PaletteMode::Reading {
            aspect: ReadingAspect::Encoding,
            ..
        } => "英字で絞る（例: sjis, utf8）",
        PaletteMode::Reading {
            aspect: ReadingAspect::Quoting,
            ..
        } => "英字で絞る（例: single）",
        PaletteMode::Reading {
            aspect: ReadingAspect::Header,
            ..
        } => "行の番号で絞る（例: 2・2-3）",
        PaletteMode::Candidates { .. } => "英字で絞る（例: tab, comma）。上から判別の点数の順",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::about_line::AboutLine;
    use crate::cursor_step::CursorStep;
    use crate::intent::Intent;
    use crate::palette_outcome::PaletteOutcome;
    use crate::theme_state::ThemeState;
    use crate::window_view::WindowView;
    use alloc::vec;
    use nene_ledger_core::find_options::FindOptions;
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
                DrawCommand::Fill { .. }
                | DrawCommand::Line { .. }
                | DrawCommand::TextSpan { .. } => None,
            })
            .collect()
    }

    fn fills(commands: &[DrawCommand], wanted: Color) -> Vec<PixelRect> {
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
    fn an_empty_palette_shows_the_hint_every_command_and_the_count() {
        let palette = CommandPalette::open();
        let commands = palette.paint(view(), colors(), DetailPane::Open);
        let shown = texts(&commands);
        assert!(shown.contains(&"操作の名前で絞る（例: 置換・haishoku・theme）"));
        assert!(shown.contains(&"47 / 47"));
        // 空の一覧は見出しから：よく使う（開く・保存・探す・置換・元に戻す）と、その下の分類（GitHub Issue #420）。
        assert!(shown.contains(&"よく使う") && shown.contains(&"ファイル"));
        assert!(shown.contains(&"ファイルを開く…") && shown.contains(&"Ctrl+O"));
        assert!(shown.contains(&"変えた所を保存する") && shown.contains(&"Ctrl+S"));
        // 選んでいる最初の行（見出しの下）は選択の色で塗り、左に強調の色の 3px の帯。
        let first_row = PaletteLayout::new(view(), &palette).row(1);
        assert_eq!(fills(&commands, colors().selection()), vec![first_row]);
        assert!(
            fills(&commands, colors().cursor())
                .contains(&PixelRect::new(first_row.origin(), PixelSize::new(3, 28)))
        );
        // 文字はどれも窓の部品の大きさ。
        assert!(commands.iter().all(|command| match command {
            DrawCommand::Text { role, .. } => *role == TextRole::Chrome,
            DrawCommand::Fill { .. } | DrawCommand::Line { .. } | DrawCommand::TextSpan { .. } =>
                true,
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
        assert!(shown.contains(&"0 / 47") && shown.contains(&"xyz"));
        assert!(fills(&commands, colors().selection()).is_empty());
    }

    /// 置換の段は 2 つの欄を縦に並べて名札を付け、その下に置き換え方と条件のキーを出す（ADR 0030 の決定 6・GitHub Issue #287）。
    #[test]
    fn the_replace_step_shows_both_fields_and_its_keys() {
        let palette = CommandPalette::open_replace("tok", "東京", FindOptions::standard());
        let layout = PaletteLayout::new(view(), &palette);
        let (find, second) = (layout.field(), layout.second().unwrap());
        assert!(second.origin().y() > find.origin().y() + find.size().height() - 1);
        let commands = palette.paint(view(), colors(), DetailPane::Closed);
        let shown = texts(&commands);
        assert_eq!(&shown[..4], &["探す", "tok", "置換後", "東京"], "{shown:?}");
        assert!(
            shown.contains(&"大小を区別") && shown.contains(&"Alt+C"),
            "{shown:?}"
        );
    }

    /// 欄の知らせは種類で色を分ける：確かめは強調、済んだは緑、注意は赤（GitHub Issue #305）。
    #[test]
    fn notes_are_colored_by_their_tone() {
        let replace = CommandPalette::open_replace("b", "x", FindOptions::standard());
        for (tone, wanted) in [
            (NoteTone::Prompt, colors().cursor()),
            (NoteTone::Done, colors().success()),
            (NoteTone::Problem, colors().danger()),
        ] {
            let commands = replace.paint_replacing(
                view(),
                (colors(), DetailPane::Closed),
                GuideNote {
                    text: Some("知らせ"),
                    tone,
                    lines: &[],
                },
            );
            assert!(commands.iter().any(|command| matches!(
                command,
                DrawCommand::Text { text, color, .. } if text == "知らせ" && *color == wanted
            )));
        }
    }

    /// 全部の置換の件数を見せている間は、条件の札の行に内訳を出し、注意は強調の色（GitHub Issue #289）。
    #[test]
    fn the_breakdown_takes_the_place_of_the_chips() {
        let palette = CommandPalette::open_replace("980", "990", FindOptions::standard());
        let lines = [
            (String::from("“980”→“990” 3 セル"), false),
            (String::from("一部だけ変わるセルあり"), true),
        ];
        let commands = palette.paint_replacing(
            view(),
            (colors(), DetailPane::Closed),
            GuideNote {
                text: Some("4 セルを置き換えます"),
                tone: NoteTone::Prompt,
                lines: &lines,
            },
        );
        let shown = texts(&commands);
        assert!(shown.contains(&"“980”→“990” 3 セル") && shown.contains(&"一部だけ変わるセルあり"));
        assert!(!shown.contains(&"大小を区別"), "{shown:?}");
        let colored = |wanted: &str| {
            commands.iter().find_map(|command| match command {
                DrawCommand::Text { text, color, .. } if text == wanted => Some(*color),
                DrawCommand::Text { .. }
                | DrawCommand::Fill { .. }
                | DrawCommand::Line { .. }
                | DrawCommand::TextSpan { .. } => None,
            })
        };
        assert_eq!(colored("一部だけ変わるセルあり"), Some(colors().danger()));
        assert_eq!(colored("“980”→“990” 3 セル"), Some(colors().text()));
    }

    /// 打っている欄だけ下辺を強調の色にして入力の印を置き、全部選んだ字の後ろは選択の色。Tab と欄を押すことで替わる
    /// （GitHub Issue #287）。
    #[test]
    fn only_the_typed_field_is_accented() {
        let mut palette = CommandPalette::open_replace("tok", "東京", FindOptions::standard());
        let layout = PaletteLayout::new(view(), &palette);
        let (find, second) = (layout.field(), layout.second().unwrap());
        let accented = |commands: &[DrawCommand], area: PixelRect| {
            fills(commands, colors().cursor()).iter().any(|&rect| {
                rect.origin().y() >= area.origin().y()
                    && rect.origin().y() < area.origin().y() + area.size().height()
            })
        };
        let first = palette.paint(view(), colors(), DetailPane::Closed);
        assert!(accented(&first, find) && !accented(&first, second));
        let selected_in = |commands: &[DrawCommand], area: PixelRect| {
            fills(commands, colors().selection())
                .iter()
                .filter(|&&rect| contains_point(area, rect.origin()))
                .count()
        };
        assert_eq!(selected_in(&first, find), 1);
        palette.handle(Intent::Advance(CursorStep::Right), 0);
        let swapped = palette.paint(view(), colors(), DetailPane::Closed);
        assert!(accented(&swapped, second) && !accented(&swapped, find));
        assert_eq!(selected_in(&swapped, find), 0);
        assert_eq!(layout.field_at(find.origin()), Some(false));
        assert_eq!(layout.field_at(second.origin()), Some(true));
        palette.click_field(false);
        assert_eq!(palette.mode(), PaletteMode::Replace { with: false });
    }

    /// 正規表現の式が読めなければ、欄の右端にそう出す（ADR 0031）。置換の段はキーの行が増える。
    #[test]
    fn the_find_line_shows_the_regex_state() {
        let regex = FindOptions::standard().toggled_regex();
        let broken = CommandPalette::open_find("(b", regex);
        let shown = broken.paint(view(), colors(), DetailPane::Closed);
        assert!(texts(&shown).contains(&"読めない正規表現"));
        let fine = CommandPalette::open_find(r"b\d", regex);
        let ok = fine.paint(view(), colors(), DetailPane::Closed);
        assert!(!texts(&ok).contains(&"読めない正規表現"));
        let replacing = CommandPalette::open_replace("b", "x", FindOptions::standard());
        let both = replacing.paint(view(), colors(), DetailPane::Closed);
        assert!(texts(&both).contains(&"Enter 置き換えて次へ · Alt+A 全部 · Tab 欄を替える"));
    }

    /// パレットの案内の長さの上限（表示の桁・全角 = 2）。Xvfb の画面写し（Noto Sans CJK JP・倍率 100%・2026-11-06）で
    /// 1 桁 ≈ 6.6 px、パレットのいちばん狭い幅（480 px）で字を置ける幅 ≈ 444 px ≈ 67 桁。切れた案内は 74・78・83 桁、
    /// 収まった案内は 64・65 桁だった（GitHub Issue #175）。
    const GUIDE_COLUMNS: u32 = 64;

    /// 条件（大文字小文字・セル全体・正規表現）の全部の組み合わせ。
    fn every_find_option() -> Vec<FindOptions> {
        let mut options = vec![FindOptions::standard()];
        for toggle in [
            FindOptions::toggled_case,
            FindOptions::toggled_whole_cell,
            FindOptions::toggled_regex,
            FindOptions::toggled_width,
            FindOptions::toggled_kana,
        ] {
            let flipped: Vec<FindOptions> = options.iter().map(|&one| toggle(one)).collect();
            options.extend(flipped);
        }
        options
    }

    /// 探す段と置換の段は、5 つの条件の札を出し、オンの札だけに ✓ と選択の色の塗りを付ける。名前とキーは条件に依らず同じ
    /// （GitHub Issue #199・#288）。
    #[test]
    fn the_find_steps_mark_only_the_conditions_that_are_on() {
        for options in every_find_option() {
            for palette in [
                CommandPalette::open_find("b", options),
                CommandPalette::open_replace("b", "", options),
            ] {
                let commands = palette.paint(view(), colors(), DetailPane::Closed);
                assert_marks(&palette, &commands, options);
            }
        }
    }

    /// 札ごとに、名前とキーがあり、オンの札だけ ✓ と選択の色の塗りがある。
    fn assert_marks(palette: &CommandPalette, commands: &[DrawCommand], options: FindOptions) {
        let chips = PaletteLayout::new(view(), palette).chips().unwrap();
        for (chip, (name, key, toggle)) in chips.into_iter().zip(FIND_CHIPS) {
            let inside = texts_in(commands, chip);
            assert!(inside.contains(&name) && inside.contains(&key));
            let on = toggle_is_on(options, toggle);
            assert_eq!(inside.contains(&"✓"), on, "{name} {options:?}");
            let filled = fills(commands, colors().selection())
                .into_iter()
                .any(|rect| contains_point(chip, rect.origin()));
            assert_eq!(filled, on, "{name} {options:?}");
        }
    }

    /// 矩形 `area` の中に書いた字。
    fn texts_in(commands: &[DrawCommand], area: PixelRect) -> Vec<&str> {
        commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Text { text, baseline, .. } if contains_point(area, *baseline) => {
                    Some(text.as_str())
                }
                DrawCommand::Text { .. }
                | DrawCommand::Fill { .. }
                | DrawCommand::Line { .. }
                | DrawCommand::TextSpan { .. } => None,
            })
            .collect()
    }

    fn contains_point(area: PixelRect, at: PixelPoint) -> bool {
        (area.origin().x()..area.origin().x() + area.size().width()).contains(&at.x())
            && (area.origin().y()..area.origin().y() + area.size().height()).contains(&at.y())
    }

    /// 探す段と置換の段の案内は、条件のどの組み合わせでもパレットの幅に収まる長さ（打った文字は除く・Issue #175）。
    #[test]
    fn find_and_replace_guides_fit_the_narrowest_palette() {
        let palettes: Vec<CommandPalette> = every_find_option()
            .into_iter()
            .flat_map(|one| {
                ["", "b", "(b"].into_iter().flat_map(move |text| {
                    [
                        CommandPalette::open_find(text, one),
                        CommandPalette::open_replace(text, "", one),
                        CommandPalette::open_replace("", text, one),
                    ]
                })
            })
            .collect();
        assert_eq!(palettes.len(), 32 * 3 * 3);
        for palette in palettes {
            let commands = palette.paint(view(), colors(), DetailPane::Closed);
            let guides = texts(&commands).into_iter().filter(|&text| {
                text != palette.query()
                    && text != palette.replacement()
                    && !text.starts_with("探す: ")
                    && !text.starts_with("置き換える: ")
            });
            for text in guides {
                assert!(
                    display_columns(text) <= GUIDE_COLUMNS,
                    "{} 桁: {text:?}（{:?}）",
                    display_columns(text),
                    palette.mode()
                );
            }
        }
    }

    /// 全部の置換の件数と確かめは、置換の段の 1 行目のキーの案内の代わりに強調の色で出し、いちばん長い数でも幅に収まる。
    /// 探す段には出さない（GitHub Issue #189）。
    #[test]
    fn the_replace_step_shows_the_count_and_the_confirmation_in_place_of_its_keys() {
        let notes = [
            "1,000,000 セルを置き換えます · Enter 置き換える · Esc やめる",
            "置き換えるセルを数えています… 99% · Esc やめる",
            "1,000,000 セルを置き換えました · 表で Ctrl+Z で戻す",
        ];
        let replace = CommandPalette::open_replace("b", "", FindOptions::standard());
        for note in notes {
            let commands = replace.paint_noted(view(), (colors(), DetailPane::Closed), Some(note));
            let shown = texts(&commands);
            assert!(shown.contains(&note), "{shown:?}");
            assert!(
                !shown
                    .iter()
                    .any(|text| text.starts_with("Enter 置き換えて次へ"))
            );
            assert!(display_columns(note) <= GUIDE_COLUMNS, "{note:?}");
            assert!(commands.iter().any(|command| matches!(
                command,
                DrawCommand::Text { text, color, .. } if text == note && *color == colors().cursor()
            )));
        }
        // 探す段の知らせは、欄の右端に強調の色で（GitHub Issue #291）。
        let find = CommandPalette::open_find("b", FindOptions::standard());
        let note = "見つかりません（オン: セル全体）";
        let commands = find.paint_noted(view(), (colors(), DetailPane::Closed), Some(note));
        let field = PaletteLayout::new(view(), &find).field();
        assert!(texts_in(&commands, field).contains(&note));
        assert!(display_columns(note) <= GUIDE_COLUMNS);
    }

    /// 「このアプリについて…」で版・著作権・ライセンスの行を出す。どの行もパレットの幅に収まる（ADR 0032・Issue #179）。
    #[test]
    fn the_about_step_shows_the_version_and_the_license() {
        let mut palette = CommandPalette::open();
        for character in "about".chars() {
            palette.handle(Intent::Type(character), 0);
        }
        assert_eq!(palette.handle(Intent::Confirm, 0), PaletteOutcome::Stay);
        assert_eq!(palette.mode(), PaletteMode::About);
        let commands = palette.paint(view(), colors(), DetailPane::Closed);
        let shown = texts(&commands);
        for line in AboutLine::ALL {
            let text = line.text();
            assert!(shown.contains(&text.as_str()), "{text:?} が無い: {shown:?}");
            assert!(display_columns(&text) <= GUIDE_COLUMNS, "{text:?}");
        }
        assert!(shown.contains(&format!("NeNe Ledger {}", env!("CARGO_PKG_VERSION")).as_str()));
        assert_eq!(palette.handle(Intent::Cancel, 0), PaletteOutcome::Close);
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
            &"Enter で開き直す · Esc で戻す · 試し中: カンマ区切り・\"\" で引用符（RFC 4180）"
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
                DrawCommand::Text { .. }
                | DrawCommand::Fill { .. }
                | DrawCommand::Line { .. }
                | DrawCommand::TextSpan { .. } => None,
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
        assert!(shown.contains(&"Enter で決める · Esc で戻す · 試し中: ubuntu-aubergine"));
        assert!(shown.contains(&"ubuntu-aubergine") && shown.contains(&"ダーク"));
        assert!(shown.contains(&"neutral-light") && shown.contains(&"ライト"));
        // 見本は配色ごとに 3 色。面はパレットを開いたときの色（渡した色）で塗る。
        let seed = at(0);
        assert!(!fills(&commands, seed.accent()).is_empty());
        let panel = PaletteLayout::new(view(), &palette).panel();
        assert!(fills(&commands, frozen.gutter()).contains(&panel));
    }

    /// 欄より長い道筋は、頭を … で略して後ろ（ファイルの名前）を見せる（ADR 0028 の決定 4・X11 の欄）。
    #[test]
    fn a_long_save_as_path_shows_its_tail() {
        let deep = format!("/srv/data/{}/表.csv", "folder/".repeat(30));
        let palette = CommandPalette::open_save_as(deep.clone());
        let commands = palette.paint(view(), colors(), DetailPane::Closed);
        let shown = texts(&commands);
        let field = shown
            .iter()
            .find(|text| text.ends_with("表.csv"))
            .expect("the file name is shown");
        assert!(field.starts_with('…'));
        assert!(field.chars().count() < deep.chars().count());
        assert!(
            shown.contains(
                &"Enter で、この道筋に名前を付けて保存する（方言と文字コードは今のまま）"
            )
        );
        let short = CommandPalette::open_save_as(String::from("/tmp/a.csv"));
        assert!(texts(&short.paint(view(), colors(), DetailPane::Closed)).contains(&"/tmp/a.csv"));
    }

    /// 2 段目の文言と印：文字コードの日本語の注・今の配色の印・見出しの段の案内・このアプリについては選ぶ色を付けない
    /// （GitHub Issue #312）。
    #[test]
    fn second_steps_say_what_each_choice_is() {
        let current = ReadingChoice::Encoding(TextEncoding::Utf8);
        let mut encodings = CommandPalette::open();
        encodings.enter_reading(ReadingAspect::Encoding, current);
        let shown_encodings = encodings.paint(view(), colors(), DetailPane::Closed);
        let texts_now = texts(&shown_encodings);
        assert!(
            texts_now.contains(&"今の読み方 · 多くの道具の標準"),
            "{texts_now:?}"
        );
        assert!(texts_now.contains(&"英字で絞る（例: sjis, utf8）"));
        let mut headers = CommandPalette::open();
        headers.enter_reading(ReadingAspect::Header, ReadingChoice::Header(None));
        let shown_headers = headers.paint(view(), colors(), DetailPane::Closed);
        assert!(texts(&shown_headers).contains(&"行の番号で絞る（例: 2・2-3）"));
        let mut themes = CommandPalette::open();
        typed_into(&mut themes, "haishoku");
        let _entered = themes.handle(Intent::Confirm, 1);
        let shown_themes = themes.paint(view(), colors(), DetailPane::Closed);
        assert!(
            texts(&shown_themes)
                .iter()
                .any(|text| text.starts_with("今の配色 · "))
        );
        let mut about = CommandPalette::open();
        typed_into(&mut about, "about");
        let _opened = about.handle(Intent::Confirm, 0);
        let shown_about = about.paint(view(), colors(), DetailPane::Closed);
        let first_row = PaletteLayout::new(view(), &about).row(0);
        assert!(!fills(&shown_about, colors().selection()).contains(&first_row));
    }

    fn typed_into(palette: &mut CommandPalette, text: &str) {
        for character in text.chars() {
            palette.handle(Intent::Type(character), 0);
        }
    }
}
