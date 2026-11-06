//! コマンドパレットの状態（AUTO-03・Issue #41）。

use alloc::string::String;
use alloc::vec::Vec;

use nene_ledger_core::dialect_candidate::DialectCandidate;
use nene_ledger_core::draw_command::DrawCommand;
use nene_ledger_core::palette::Palette;

use crate::command_id::CommandId;
use crate::cursor_step::CursorStep;
use crate::detail_pane::DetailPane;
use crate::find_toggle::FindToggle;
use crate::go_to_target::GoToTarget;
use crate::intent::Intent;
use crate::palette_item::PaletteItem;
use crate::palette_layout::PaletteLayout;
use crate::palette_mode::PaletteMode;
use crate::palette_outcome::PaletteOutcome;
use crate::palette_painter::PalettePainter;
use crate::palette_search::{self, commands, key_list, readings, themes};
use crate::reading_aspect::ReadingAspect;
use crate::reading_choice::ReadingChoice;
use crate::window_view::WindowView;
use nene_ledger_core::find_options::FindOptions;

/// `PageUp` / `PageDown` で動く候補の数。
const PAGE: usize = 10;

/// コマンドパレットの状態。段・検索の文字列・選んでいる候補・判別の候補（候補を選ぶ段で見せる）。
/// 開いている間は、どのキーもここへ来る（表には届かない）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPalette {
    mode: PaletteMode,
    query: String,
    selected: usize,
    candidates: Vec<DialectCandidate>,
    find_options: FindOptions,
}

impl CommandPalette {
    /// 飛ぶ先を打つ段で開く（Ctrl+G・Issue #54）。
    #[must_use]
    pub fn open_go_to() -> Self {
        Self {
            mode: PaletteMode::GoTo,
            query: String::new(),
            selected: 0,
            candidates: Vec::new(),
            find_options: FindOptions::standard(),
        }
    }

    /// 探す文字を打つ段で開く（Ctrl+F・ADR 0030 の決定 1）。前に探した文字 `text` と条件 `options` を入れておく。
    #[must_use]
    pub fn open_find(text: &str, options: FindOptions) -> Self {
        Self {
            mode: PaletteMode::Find,
            query: String::from(text),
            selected: 0,
            candidates: Vec::new(),
            find_options: options,
        }
    }

    /// 検索の条件（探す段で切り替える）。
    #[must_use]
    pub const fn find_options(&self) -> FindOptions {
        self.find_options
    }

    /// 名前を付けて保存の保存先を打つ段で開く（X11・ADR 0028 の決定 4）。初めの道筋は `initial`（今のファイルのフォルダ）。
    #[must_use]
    pub fn open_save_as(initial: String) -> Self {
        Self {
            mode: PaletteMode::SaveAs { overwrite: false },
            query: initial,
            selected: 0,
            candidates: Vec::new(),
            find_options: FindOptions::standard(),
        }
    }

    /// 打った道筋のファイルが既にあるので、上書きしてよいかを尋ねる（もう一度 Enter で上書き・ADR 0028 の決定 5）。
    /// 道筋を打ち直せば尋ねるのをやめる。
    pub fn ask_overwrite(&mut self) {
        if let PaletteMode::SaveAs { .. } = self.mode {
            self.mode = PaletteMode::SaveAs { overwrite: true };
        }
    }

    /// 操作の一覧を、何も打たずに開く。
    #[must_use]
    pub fn open() -> Self {
        Self {
            mode: PaletteMode::Commands,
            query: String::new(),
            selected: 0,
            candidates: Vec::new(),
            find_options: FindOptions::standard(),
        }
    }

    /// 段。
    #[must_use]
    pub fn mode(&self) -> PaletteMode {
        self.mode
    }

    /// 検索の文字列。
    #[must_use]
    pub fn query(&self) -> &str {
        &self.query
    }

    /// 今の候補（絞り込んだ順）。
    #[must_use]
    pub fn items(&self) -> Vec<PaletteItem> {
        match self.mode {
            PaletteMode::Commands => commands(&self.query),
            // 飛ぶ先は候補ではなく、打った番号そのもの。
            // 飛ぶ先と保存先は候補ではなく、打った文字列そのもの。
            PaletteMode::GoTo | PaletteMode::Find | PaletteMode::SaveAs { .. } => Vec::new(),
            PaletteMode::Keys => key_list(&self.query),
            PaletteMode::Themes { .. } => themes(&self.query),
            PaletteMode::Reading { aspect, .. } => readings(aspect, &self.query),
            PaletteMode::Candidates { .. } => {
                palette_search::candidates(&self.candidates, &self.query)
            }
        }
    }

    /// 判別の候補（候補を選ぶ段で見せる全部・点数の順）。ほかの段では空。
    #[must_use]
    pub fn candidates(&self) -> &[DialectCandidate] {
        &self.candidates
    }

    /// 選んでいる候補の番目（候補の数に収める）。
    #[must_use]
    pub fn selected(&self) -> usize {
        self.selected.min(self.items().len().saturating_sub(1))
    }

    /// 意図を受ける。`current_theme` は今の配色の位置（配色を選ぶ段へ入るとき、元に戻す先として覚える）。
    pub fn handle(&mut self, intent: Intent, current_theme: usize) -> PaletteOutcome {
        match intent {
            Intent::Type(character) => self.edit_query(Some(character)),
            Intent::Erase => self.edit_query(None),
            Intent::Move { step, .. } => self.step(step),
            Intent::Confirm
            | Intent::Advance(CursorStep::Up)
            | Intent::FindAgain { .. }
            | Intent::FindToggle(_)
                if self.mode == PaletteMode::Find =>
            {
                self.find_intent(intent)
            }
            Intent::Confirm if self.mode == PaletteMode::GoTo => GoToTarget::parse(&self.query)
                .map_or(PaletteOutcome::Stay, |target| {
                    PaletteOutcome::Run(Intent::GoTo(target))
                }),
            Intent::Confirm => self.confirmed(current_theme),
            Intent::Cancel | Intent::OpenPalette | Intent::OpenGoTo | Intent::OpenFind => {
                self.cancelled()
            }
            Intent::Scroll(_)
            | Intent::Point { .. }
            | Intent::DoubleClick { .. }
            | Intent::Drag { .. }
            | Intent::Release
            | Intent::SelectAll
            | Intent::SelectRows
            | Intent::SelectColumns
            | Intent::MoveColumns(_)
            | Intent::ResetColumns
            | Intent::ResetWidths
            | Intent::FreezeColumns
            | Intent::UnfreezeColumns
            | Intent::GoTo(_)
            | Intent::Copy
            | Intent::Cut
            | Intent::FindAgain { .. }
            | Intent::FindToggle(_)
            | Intent::ToggleDetail
            | Intent::Theme(_)
            | Intent::Zoom(_)
            | Intent::Edit
            | Intent::Delete
            | Intent::BreakLine
            | Intent::Advance(_)
            | Intent::Save
            | Intent::SaveAs
            | Intent::NewTable
            | Intent::Paste
            | Intent::Undo
            | Intent::Redo
            | Intent::InsertRows(_)
            | Intent::RemoveRows
            | Intent::InsertColumns(_)
            | Intent::RemoveColumns
            | Intent::InsertHere
            | Intent::RemoveHere => PaletteOutcome::Stay,
        }
    }

    /// Enter：保存先の段は道筋で保存する（空なら何もしない）、ほかの段は選んでいる候補を決める。
    fn confirmed(&mut self, current_theme: usize) -> PaletteOutcome {
        match self.mode {
            PaletteMode::SaveAs { .. } if self.query.trim().is_empty() => PaletteOutcome::Stay,
            PaletteMode::SaveAs { overwrite } => PaletteOutcome::SaveAs { overwrite },
            PaletteMode::Commands
            | PaletteMode::GoTo
            | PaletteMode::Find
            | PaletteMode::Keys
            | PaletteMode::Reading { .. }
            | PaletteMode::Candidates { .. }
            | PaletteMode::Themes { .. } => self.confirm(current_theme),
        }
    }

    /// 探す段の Enter（次）・Shift+Enter（前）・F3・Shift+F3 と、Alt+C・Alt+W の条件の切り替え（ADR 0030 の決定 1・3）。
    fn find_intent(&mut self, intent: Intent) -> PaletteOutcome {
        if let Intent::FindToggle(toggle) = intent {
            self.find_options = match toggle {
                FindToggle::Case => self.find_options.toggled_case(),
                FindToggle::WholeCell => self.find_options.toggled_whole_cell(),
            };
            return PaletteOutcome::Stay;
        }
        let forward = if let Intent::FindAgain { forward } = intent {
            forward
        } else {
            intent == Intent::Confirm
        };
        PaletteOutcome::Find { forward }
    }

    /// 窓 `view` に重ねる描画命令。色 `colors` はパレットを開いたときの配色（試しても動かない）。`detail` は詳細ペインの開閉。
    #[must_use]
    pub fn paint(&self, view: WindowView, colors: Palette, detail: DetailPane) -> Vec<DrawCommand> {
        PalettePainter::new(self, PaletteLayout::new(view, self), view.metrics(), colors)
            .paint(detail)
    }

    /// 読み方の面 `aspect` を選ぶ段へ入る。`current` は今の読み方で、それを選んだ状態にする（Issue #52）。
    pub fn enter_reading(&mut self, aspect: ReadingAspect, current: ReadingChoice) {
        self.mode = PaletteMode::Reading { aspect, current };
        self.query.clear();
        self.selected = aspect
            .options()
            .iter()
            .position(|&choice| choice == current)
            .unwrap_or(0);
    }

    /// 判別の候補 `candidates`（点数の順）を選ぶ段へ入る。`current` は今の読み方で、候補にあればそれを選んだ状態にする（Issue #3）。
    pub fn enter_candidates(&mut self, candidates: Vec<DialectCandidate>, current: ReadingChoice) {
        self.selected = candidates
            .iter()
            .position(|candidate| ReadingChoice::dialect(candidate.dialect()) == current)
            .unwrap_or(0);
        self.candidates = candidates;
        self.mode = PaletteMode::Candidates { current };
        self.query.clear();
    }

    /// 候補の `index` 番目をクリックした（選んで決める）。
    pub fn click(&mut self, index: usize, current_theme: usize) -> PaletteOutcome {
        self.selected = index;
        self.confirm(current_theme)
    }

    /// 選んでいる候補を `step` で動かす。
    fn step(&mut self, step: CursorStep) -> PaletteOutcome {
        let last = self.items().len().saturating_sub(1);
        let current = self.selected();
        let next = match step {
            CursorStep::Up => current.saturating_sub(1),
            CursorStep::Down => current.saturating_add(1).min(last),
            CursorStep::PageUp => current.saturating_sub(PAGE),
            CursorStep::PageDown => current.saturating_add(PAGE).min(last),
            CursorStep::RowStart | CursorStep::Top => 0,
            CursorStep::RowEnd | CursorStep::Bottom => last,
            CursorStep::Left | CursorStep::Right => current,
        };
        self.retarget(next)
    }

    /// `index` 番目を選ぶ。配色の段なら、その配色をその場で試す。
    fn retarget(&mut self, index: usize) -> PaletteOutcome {
        self.selected = index;
        match self.items().get(self.selected()) {
            Some(&PaletteItem::Theme(position)) => PaletteOutcome::Preview(position),
            Some(&PaletteItem::Reading(choice)) => PaletteOutcome::PreviewReading(choice),
            Some(&PaletteItem::Candidate(candidate)) => {
                PaletteOutcome::PreviewReading(ReadingChoice::dialect(candidate.dialect()))
            }
            Some(&PaletteItem::Command(_) | &PaletteItem::Key(_)) | None => PaletteOutcome::Stay,
        }
    }

    /// 欄に字 `typed` を足す（`None` は 1 字消す）。上書きを尋ねていればやめ、候補を頭から選び直す。
    fn edit_query(&mut self, typed: Option<char>) -> PaletteOutcome {
        match typed {
            Some(character) => self.query.push(character),
            None => drop(self.query.pop()),
        }
        self.stop_asking();
        self.retarget(0)
    }

    /// Esc（とパレットを開くキー）。試していれば元に戻して閉じる。
    fn cancelled(&self) -> PaletteOutcome {
        match self.mode {
            PaletteMode::Commands
            | PaletteMode::GoTo
            | PaletteMode::Find
            | PaletteMode::Keys
            | PaletteMode::SaveAs { .. } => PaletteOutcome::Close,
            PaletteMode::Themes { original } => PaletteOutcome::Revert(original),
            PaletteMode::Reading { .. } | PaletteMode::Candidates { .. } => {
                PaletteOutcome::RevertReading
            }
        }
    }

    /// 道筋を打ち直したので、上書きを尋ねるのをやめる。
    fn stop_asking(&mut self) {
        if let PaletteMode::SaveAs { overwrite: true } = self.mode {
            self.mode = PaletteMode::SaveAs { overwrite: false };
        }
    }

    /// 選んでいる候補で決める。候補が無ければ何もしない。配色を選ぶは 2 段目へ入り、今の配色を選んだ状態にする。
    fn confirm(&mut self, current_theme: usize) -> PaletteOutcome {
        match self.items().get(self.selected()) {
            // パレットに無いキーの行は読むだけ。
            None | Some(&PaletteItem::Key(_)) => PaletteOutcome::Stay,
            Some(&PaletteItem::Theme(position)) => PaletteOutcome::Commit(position),
            Some(&PaletteItem::Reading(choice)) => PaletteOutcome::CommitReading(choice),
            Some(&PaletteItem::Candidate(candidate)) => {
                PaletteOutcome::CommitReading(ReadingChoice::dialect(candidate.dialect()))
            }
            Some(&PaletteItem::Command(command)) => self.choose(command, current_theme),
        }
    }

    /// 操作 `command` で決める。意図があれば実行し、2 段目のある操作はその段へ入る（読み方と判別の候補は合成ルートが候補を渡す）。
    fn choose(&mut self, command: CommandId, current_theme: usize) -> PaletteOutcome {
        if let Some(intent) = command.intent() {
            return PaletteOutcome::Run(intent);
        }
        if let Some(aspect) = command.reading_aspect() {
            return PaletteOutcome::OpenReading(aspect);
        }
        if command == CommandId::ChooseDialect {
            return PaletteOutcome::OpenCandidates;
        }
        self.query.clear();
        if command == CommandId::ShowKeys {
            self.mode = PaletteMode::Keys;
            self.selected = 0;
            return PaletteOutcome::Stay;
        }
        self.mode = PaletteMode::Themes {
            original: current_theme,
        };
        self.selected = current_theme;
        PaletteOutcome::Stay
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selection_mode::SelectionMode;
    use crate::zoom_step::ZoomStep;

    fn down() -> Intent {
        Intent::Move {
            step: CursorStep::Down,
            mode: SelectionMode::Collapse,
        }
    }

    fn typed(palette: &mut CommandPalette, text: &str) {
        for character in text.chars() {
            palette.handle(Intent::Type(character), 0);
        }
    }

    /// 探す段：Enter で次・Shift+Enter と Shift+F3 で前・Alt+C と Alt+W で条件を切り替え、Esc で閉じる（ADR 0030 の決定 1・3）。
    #[test]
    fn the_find_step_searches_both_ways_and_toggles_its_options() {
        let mut palette = CommandPalette::open_find("tok", FindOptions::standard());
        assert_eq!(
            (palette.mode(), palette.query()),
            (PaletteMode::Find, "tok")
        );
        typed(&mut palette, "yo");
        assert_eq!(palette.query(), "tokyo");
        assert_eq!(
            palette.handle(Intent::Confirm, 0),
            PaletteOutcome::Find { forward: true }
        );
        assert_eq!(
            palette.handle(Intent::Advance(CursorStep::Up), 0),
            PaletteOutcome::Find { forward: false }
        );
        assert_eq!(
            palette.handle(Intent::FindAgain { forward: false }, 0),
            PaletteOutcome::Find { forward: false }
        );
        assert_eq!(
            palette.handle(Intent::FindToggle(FindToggle::Case), 0),
            PaletteOutcome::Stay
        );
        palette.handle(Intent::FindToggle(FindToggle::WholeCell), 0);
        assert!(palette.find_options().match_case() && palette.find_options().whole_cell());
        assert!(palette.items().is_empty());
        assert_eq!(palette.handle(Intent::Cancel, 0), PaletteOutcome::Close);
    }

    #[test]
    fn typing_filters_and_enter_runs_the_selected_command() {
        let mut palette = CommandPalette::open();
        typed(&mut palette, "zoom");
        assert_eq!(palette.query(), "zoom");
        assert_eq!(palette.items().len(), 3);
        assert_eq!(palette.handle(down(), 0), PaletteOutcome::Stay);
        assert_eq!(
            palette.handle(Intent::Confirm, 0),
            PaletteOutcome::Run(Intent::Zoom(ZoomStep::Out))
        );
        palette.handle(Intent::Erase, 0);
        assert_eq!(palette.query(), "zoo");
        assert_eq!(palette.selected(), 0);
    }

    #[test]
    fn nothing_matching_makes_enter_do_nothing_and_escape_closes() {
        let mut palette = CommandPalette::open();
        typed(&mut palette, "xyz");
        assert!(palette.items().is_empty());
        assert_eq!(palette.handle(Intent::Confirm, 0), PaletteOutcome::Stay);
        assert_eq!(palette.handle(Intent::Cancel, 0), PaletteOutcome::Close);
        assert_eq!(
            palette.handle(Intent::OpenPalette, 0),
            PaletteOutcome::Close
        );
        // 開いている間の他のキーは表に届かない。
        assert_eq!(palette.handle(Intent::Copy, 0), PaletteOutcome::Stay);
    }

    #[test]
    fn the_key_list_runs_commands_and_only_shows_the_other_keys() {
        use crate::key_guide::KeyGuide;
        let mut palette = CommandPalette::open();
        typed(&mut palette, "keys");
        assert_eq!(palette.items(), [PaletteItem::Command(CommandId::ShowKeys)]);
        assert_eq!(palette.handle(Intent::Confirm, 0), PaletteOutcome::Stay);
        assert_eq!(palette.mode(), PaletteMode::Keys);
        assert_eq!(palette.query(), "");
        // キーのある操作が先で、Enter で実行する。
        assert_eq!(
            palette.items().first(),
            Some(&PaletteItem::Command(CommandId::Top))
        );
        assert_eq!(
            palette.handle(Intent::Confirm, 0),
            PaletteOutcome::Run(CommandId::Top.intent().unwrap())
        );
        // パレットに無いキーの行は読むだけ（閉じない）。Esc で閉じる。
        palette.handle(
            Intent::Move {
                step: CursorStep::Bottom,
                mode: SelectionMode::Collapse,
            },
            0,
        );
        assert_eq!(
            palette.items().get(palette.selected()),
            Some(&PaletteItem::Key(KeyGuide::Advance))
        );
        assert_eq!(palette.handle(Intent::Confirm, 0), PaletteOutcome::Stay);
        assert_eq!(palette.handle(Intent::Cancel, 0), PaletteOutcome::Close);
    }

    #[test]
    fn typing_one_symbol_offers_it_as_the_delimiter_and_letters_still_search() {
        let mut palette = CommandPalette::open();
        palette.enter_reading(ReadingAspect::Delimiter, ReadingChoice::Delimiter(b','));
        // 1 字の記号はその字で区切る候補になり、その場で試す。Enter で開き直す。
        assert_eq!(
            palette.handle(Intent::Type('#'), 0),
            PaletteOutcome::PreviewReading(ReadingChoice::Delimiter(b'#'))
        );
        assert_eq!(
            palette.items(),
            alloc::vec![PaletteItem::Reading(ReadingChoice::Delimiter(b'#'))]
        );
        assert_eq!(
            palette.handle(Intent::Confirm, 0),
            PaletteOutcome::CommitReading(ReadingChoice::Delimiter(b'#'))
        );
        // 2 字目からは検索。英字 1 字も検索（`t` はタブの検索語）。
        palette.handle(Intent::Type('#'), 0);
        assert_eq!(
            palette.items().len(),
            ReadingAspect::Delimiter.options().len()
        );
        palette.handle(Intent::Erase, 0);
        palette.handle(Intent::Erase, 0);
        palette.handle(Intent::Type('t'), 0);
        assert!(
            palette
                .items()
                .contains(&PaletteItem::Reading(ReadingChoice::Delimiter(b'\t')))
        );
        // 空白 1 つは空白区切り。ほかの面では字を区切りにしない。
        palette.handle(Intent::Erase, 0);
        palette.handle(Intent::Type(' '), 0);
        assert_eq!(
            palette.items(),
            alloc::vec![PaletteItem::Reading(ReadingChoice::Delimiter(b' '))]
        );
        palette.enter_reading(
            ReadingAspect::Encoding,
            ReadingChoice::Encoding(nene_ledger_core::text_encoding::TextEncoding::Utf8),
        );
        palette.handle(Intent::Type('#'), 0);
        assert_eq!(
            palette.items().len(),
            ReadingAspect::Encoding.options().len()
        );
    }

    #[test]
    fn page_and_end_keys_move_through_the_list() {
        let mut palette = CommandPalette::open();
        let moving = |step| Intent::Move {
            step,
            mode: SelectionMode::Collapse,
        };
        palette.handle(moving(CursorStep::Bottom), 0);
        assert_eq!(palette.selected(), CommandId::ALL.len() - 1);
        // 1 回の PageUp は 10 件。操作は 34 なので 4 回で先頭（最後は 0 で止まる）。
        for page in 1..=3 {
            palette.handle(moving(CursorStep::PageUp), 0);
            assert_eq!(palette.selected(), CommandId::ALL.len() - 1 - 10 * page);
        }
        palette.handle(moving(CursorStep::PageUp), 0);
        assert_eq!(palette.selected(), 0);
        for page in 1..=3 {
            palette.handle(moving(CursorStep::PageDown), 0);
            assert_eq!(palette.selected(), 10 * page);
        }
        palette.handle(moving(CursorStep::PageDown), 0);
        assert_eq!(palette.selected(), CommandId::ALL.len() - 1);
        palette.handle(moving(CursorStep::RowStart), 0);
        palette.handle(moving(CursorStep::Left), 0);
        assert_eq!(palette.selected(), 0);
    }

    #[test]
    fn choosing_a_theme_previews_commits_and_reverts() {
        let mut palette = CommandPalette::open();
        typed(&mut palette, "theme");
        // 「配色を選ぶ…」で 2 段目へ。今の配色（5 番目）を選んだ状態で、試しはまだしない。
        assert_eq!(palette.handle(Intent::Confirm, 5), PaletteOutcome::Stay);
        assert_eq!(palette.mode(), PaletteMode::Themes { original: 5 });
        assert_eq!(palette.query(), "");
        assert_eq!(palette.selected(), 5);
        assert_eq!(palette.handle(down(), 5), PaletteOutcome::Preview(6));
        assert_eq!(
            palette.handle(Intent::Confirm, 6),
            PaletteOutcome::Commit(6)
        );
        assert_eq!(palette.handle(Intent::Cancel, 6), PaletteOutcome::Revert(5));
        // 名前で絞ると、先頭の候補を試す。
        let solarized = nene_ledger_core::theme_catalog::position_of("solarized-dark").unwrap();
        palette.handle(Intent::Erase, 5);
        typed(&mut palette, "solarized");
        assert_eq!(
            palette.items().first(),
            Some(&PaletteItem::Theme(solarized))
        );
        assert_eq!(palette.click(0, 5), PaletteOutcome::Commit(solarized));
    }

    #[test]
    fn choosing_a_reading_opens_its_list_previews_and_commits() {
        let mut palette = CommandPalette::open();
        typed(&mut palette, "delimiter");
        assert_eq!(
            palette.handle(Intent::Confirm, 0),
            PaletteOutcome::OpenReading(ReadingAspect::Delimiter)
        );
        // 合成ルートが今の読み方（カンマ）を渡して段に入る。カンマを選んだ状態。
        palette.enter_reading(ReadingAspect::Delimiter, ReadingChoice::Delimiter(b','));
        assert_eq!(palette.selected(), 0);
        assert_eq!(
            palette.handle(down(), 0),
            PaletteOutcome::PreviewReading(ReadingChoice::Delimiter(b'\t'))
        );
        assert_eq!(
            palette.handle(Intent::Confirm, 0),
            PaletteOutcome::CommitReading(ReadingChoice::Delimiter(b'\t'))
        );
        assert_eq!(
            palette.handle(Intent::Cancel, 0),
            PaletteOutcome::RevertReading
        );
        // 英字で絞る。
        typed(&mut palette, "pipe");
        assert_eq!(
            palette.items(),
            alloc::vec![PaletteItem::Reading(ReadingChoice::Delimiter(b'|'))]
        );
    }

    #[test]
    fn the_candidate_list_marks_the_current_dialect_previews_and_commits() {
        use nene_ledger_core::delimiter::Delimiter;
        use nene_ledger_core::dialect::Dialect;
        use nene_ledger_core::quoting::Quoting;
        let mut palette = CommandPalette::open();
        typed(&mut palette, "candidates");
        assert_eq!(
            palette.items(),
            alloc::vec![PaletteItem::Command(CommandId::ChooseDialect)]
        );
        assert_eq!(
            palette.handle(Intent::Confirm, 0),
            PaletteOutcome::OpenCandidates
        );
        let comma = DialectCandidate::new(Dialect::rfc4180(), 900, 5);
        let tab = DialectCandidate::new(
            Dialect::new(Delimiter::TAB, Quoting::Doubled, b'"').unwrap(),
            400,
            2,
        );
        // 合成ルートが候補と今の読み方（タブ）を渡す。2 番目を選んだ状態で、試しはまだしない。
        let current = ReadingChoice::dialect(tab.dialect());
        palette.enter_candidates(alloc::vec![comma, tab], current);
        assert_eq!(palette.mode(), PaletteMode::Candidates { current });
        assert_eq!(palette.query(), "");
        assert_eq!(palette.selected(), 1);
        assert_eq!(palette.candidates(), [comma, tab]);
        let up = Intent::Move {
            step: CursorStep::Up,
            mode: SelectionMode::Collapse,
        };
        assert_eq!(
            palette.handle(up, 0),
            PaletteOutcome::PreviewReading(ReadingChoice::dialect(comma.dialect()))
        );
        assert_eq!(
            palette.handle(Intent::Confirm, 0),
            PaletteOutcome::CommitReading(ReadingChoice::dialect(comma.dialect()))
        );
        assert_eq!(
            palette.handle(Intent::Cancel, 0),
            PaletteOutcome::RevertReading
        );
    }

    #[test]
    fn a_current_dialect_outside_the_candidates_selects_the_first_and_typing_filters() {
        use nene_ledger_core::delimiter::Delimiter;
        use nene_ledger_core::dialect::Dialect;
        use nene_ledger_core::quoting::Quoting;
        let mut palette = CommandPalette::open();
        let comma = DialectCandidate::new(Dialect::rfc4180(), 900, 5);
        let tab = DialectCandidate::new(
            Dialect::new(Delimiter::TAB, Quoting::Doubled, b'"').unwrap(),
            400,
            2,
        );
        palette.enter_candidates(alloc::vec![comma, tab], ReadingChoice::Delimiter(b'#'));
        assert_eq!(palette.selected(), 0);
        // 英字で絞ると先頭の候補を試し、クリックで開き直す。
        assert_eq!(
            palette.handle(Intent::Type('t'), 0),
            PaletteOutcome::PreviewReading(ReadingChoice::dialect(tab.dialect()))
        );
        typed(&mut palette, "ab");
        assert_eq!(palette.items(), alloc::vec![PaletteItem::Candidate(tab)]);
        assert_eq!(
            palette.click(0, 0),
            PaletteOutcome::CommitReading(ReadingChoice::dialect(tab.dialect()))
        );
    }

    #[test]
    fn the_go_to_stage_runs_a_jump_for_a_readable_number() {
        let mut palette = CommandPalette::open_go_to();
        assert!(palette.items().is_empty());
        typed(&mut palette, "x");
        assert_eq!(palette.handle(Intent::Confirm, 0), PaletteOutcome::Stay);
        palette.handle(Intent::Erase, 0);
        typed(&mut palette, "12,2");
        assert_eq!(
            palette.handle(Intent::Confirm, 0),
            PaletteOutcome::Run(Intent::GoTo(GoToTarget::parse("12,2").unwrap()))
        );
        assert_eq!(palette.handle(Intent::Cancel, 0), PaletteOutcome::Close);
    }

    /// 名前を付けて保存の欄は、既にあるファイルなら一度尋ね、もう一度 Enter を受けるまで上書きの結果を出さない。
    /// 道筋を打ち直せば尋ねるのをやめる（ADR 0028 の決定 5・強制の 3 つ目）。
    #[test]
    fn saving_as_needs_a_second_enter_to_overwrite() {
        let mut palette = CommandPalette::open_save_as(String::from("/tmp/"));
        assert_eq!(palette.query(), "/tmp/");
        for character in "a.csv".chars() {
            palette.handle(Intent::Type(character), 0);
        }
        assert_eq!(
            palette.handle(Intent::Confirm, 0),
            PaletteOutcome::SaveAs { overwrite: false }
        );
        // 合成ルートがファイルを見つけて尋ねる。
        palette.ask_overwrite();
        assert_eq!(palette.mode(), PaletteMode::SaveAs { overwrite: true });
        assert_eq!(
            palette.handle(Intent::Confirm, 0),
            PaletteOutcome::SaveAs { overwrite: true }
        );
        // 打ち直すと、もう一度確かめる。
        palette.handle(Intent::Erase, 0);
        assert_eq!(palette.mode(), PaletteMode::SaveAs { overwrite: false });
        assert_eq!(
            palette.handle(Intent::Confirm, 0),
            PaletteOutcome::SaveAs { overwrite: false }
        );
        assert_eq!(palette.handle(Intent::Cancel, 0), PaletteOutcome::Close);
    }

    /// 空の欄では何もしない。ほかの段で尋ねる印は付かない。
    #[test]
    fn saving_as_ignores_an_empty_path_and_other_modes_never_ask() {
        let mut empty = CommandPalette::open_save_as(String::from("  "));
        assert_eq!(empty.handle(Intent::Confirm, 0), PaletteOutcome::Stay);
        let mut commands = CommandPalette::open();
        commands.ask_overwrite();
        assert_eq!(commands.mode(), PaletteMode::Commands);
    }
}
