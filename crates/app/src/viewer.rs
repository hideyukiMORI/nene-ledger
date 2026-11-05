//! 窓が描く相手。画面の状態と、裏の索引の仕事と、範囲の読み取りを結ぶ。

use nene_ledger_application::clipboard_text::ClipboardText;
use nene_ledger_application::close_state::CloseState;
use nene_ledger_application::command_palette::CommandPalette;
use nene_ledger_application::cut_record::CutRecord;
use nene_ledger_application::document::Document;
use nene_ledger_application::document_effect::DocumentEffect;
use nene_ledger_application::format_summary::FormatSummary;
use nene_ledger_application::index_summary::IndexSummary;
use nene_ledger_application::intent::Intent;
use nene_ledger_application::job_progress::JobProgress;
use nene_ledger_application::job_report::JobReport;
use nene_ledger_application::notice::Notice;
use nene_ledger_application::palette_layout::PaletteLayout;
use nene_ledger_application::palette_outcome::PaletteOutcome;
use nene_ledger_application::pointer_shape::PointerShape;
use nene_ledger_application::reading_choice::ReadingChoice;
use nene_ledger_application::selection_totals::SelectionTotals;
use nene_ledger_application::session::Session;
use nene_ledger_application::settings_failure::SettingsFailure;
use nene_ledger_application::theme_state::ThemeState;
use nene_ledger_application::ticker::Ticker;
use nene_ledger_application::totals_request::TotalsRequest;
use nene_ledger_application::window_view::WindowView;
use nene_ledger_application::zoom_level::ZoomLevel;
use nene_ledger_core::frame::Frame;
use nene_ledger_core::order_run::OrderRun;
use nene_ledger_core::pixel_point::PixelPoint;
use nene_ledger_core::pixel_rect::PixelRect;
use nene_ledger_core::preedit::Preedit;
use nene_ledger_scan::index_job::IndexJob;
use nene_ledger_scan::index_outcome::IndexOutcome;
use nene_ledger_scan::range_job::RangeJob;
use nene_ledger_scan::range_outcome::RangeOutcome;
use nene_ledger_scan::scan_failure::ScanFailure;
use nene_ledger_store::save_job::SaveJob;

use crate::open_palette::OpenPalette;
use crate::preferences::Preferences;
use crate::reader::Reader;
use crate::rows::WINDOW_RECORDS;
use crate::saving::{SavingFailure, describe, problem_of, start_saving};
use crate::totals_fold::TotalsFold;
use crate::warn;
use crate::workers::Workers;

/// 窓が描く相手（`Session` の実装）。状態は application、索引の仕事と読み取りは scan が持ち、ここは結ぶだけ（ARC-006）。
#[derive(Debug)]
pub(crate) struct Viewer {
    document: Document,
    job: IndexJob,
    ticker: Box<dyn Ticker>,
    reader: Reader,
    preferences: Preferences,
    palette: Option<OpenPalette>,
    totals: Option<RangeJob<TotalsFold>>,
    /// 読み方を試している間の、元の画面（Esc で戻す・Issue #52）。
    trial: Option<Document>,
    /// 裏で回っている保存（ADR 0026 の決定 6）。
    saving: Option<SaveJob>,
}

impl Viewer {
    pub(crate) fn new(
        mut document: Document,
        workers: Workers,
        reader: Reader,
        preferences: Preferences,
    ) -> Self {
        document.set_detail(preferences.settings().detail());
        let (job, ticker) = workers.into_parts();
        Self {
            document,
            job,
            ticker,
            reader,
            preferences,
            palette: None,
            totals: None,
            trial: None,
            saving: None,
        }
    }

    /// 書き換えの保存を裏で始める。始められなければ理由を知らせる。
    fn start_saving(&mut self) {
        match start_saving(self.document.changes(), &self.reader) {
            Ok(job) => {
                self.saving = Some(job);
                self.document.begin_saving();
            }
            Err(failure) => {
                warn(&describe(&failure));
                self.document.save_failed(problem_of(&failure));
            }
        }
    }

    /// 裏の保存が終わっていれば、結果を画面に取り込む。成功したら置き換えた後のファイルを開き直す。
    fn finish_saving(&mut self) {
        let Some(outcome) = self.saving.as_mut().and_then(SaveJob::take_result) else {
            return;
        };
        self.saving = None;
        if let Err(failure) = outcome {
            let failure = SavingFailure::Save(failure);
            warn(&describe(&failure));
            self.document.save_failed(problem_of(&failure));
            return;
        }
        // 前の索引の仕事と集計は、置き換える前のファイルのもの。捨ててから開き直す。
        self.totals = None;
        match self.reader.reopen_saved() {
            Ok(job) => {
                self.job = job;
                let reread = self.document.saved();
                self.effect(reread);
            }
            Err(failure) => {
                warn(&format!("保存した後に開き直せなかった: {failure:?}"));
                let _reread = self.document.saved();
                self.document.notify(Notice::ReopenFailed);
            }
        }
    }

    /// 画面の状態が時計の刻みを要るときだけ刻ませる（溝を押し続けている間）。
    fn sync_ticker(&self) {
        if self.document.wants_ticks() {
            self.ticker.start();
        } else {
            self.ticker.stop();
        }
    }

    /// 変えたことを知らせる。設定を書けなかったら、そちらを知らせる。
    fn notify_saved(&mut self, notice: Notice, saved: Result<(), SettingsFailure>) {
        let shown = match saved {
            Ok(()) => notice,
            Err(SettingsFailure::Unwritable) => Notice::SettingsUnsaved,
        };
        self.document.notify(shown);
    }

    /// 仕事の今の知らせ。終わっていれば結果（作り終えた索引は読み取りに使う）、まだなら進み具合。
    fn report(&mut self) -> JobReport {
        match self.job.take_result() {
            Some(Ok(IndexOutcome::Finished(index))) => {
                let summary =
                    IndexSummary::new(index.records(), index.unclosed_quotes().first().copied());
                self.reader.use_index(index);
                JobReport::Finished(summary)
            }
            Some(Ok(IndexOutcome::Cancelled)) => JobReport::Cancelled,
            Some(Err(_failure)) => JobReport::Failed,
            None => {
                let progress = self.job.progress();
                JobReport::Progress(
                    JobProgress::new(
                        progress.done(),
                        progress.total(),
                        progress.long_quoted_cell(),
                    )
                    .with_records(progress.records()),
                )
            }
        }
    }

    /// 画面の状態が求めたことをする。縦の表示が動くなら、その先の本体の行のレコードから範囲を目印から読んで差し替える
    /// （読めない・行が無ければ縦には動かない）。コピーなら選択の行を読んで文字列にする。
    fn effect(&mut self, effect: DocumentEffect) -> Option<ClipboardText> {
        match effect {
            DocumentEffect::Stay => None,
            DocumentEffect::Read { first_body } => {
                let first = self.document.line_of(first_body);
                if let Ok((cells, cuts)) = self.read_lines(first, WINDOW_RECORDS) {
                    self.document.show(first, &cells);
                    self.document.mark_cut(cuts);
                }
                None
            }
            DocumentEffect::Copy { first_line, count } => {
                let count = usize::try_from(count).ok()?;
                let (rows, cuts) = self.read_lines(first_line, count).ok()?;
                self.document.copy_text(first_line, &rows, &cuts)
            }
            DocumentEffect::StopTotals => {
                if let Some(job) = &self.totals {
                    job.cancel();
                }
                None
            }
            DocumentEffect::Save => {
                self.start_saving();
                None
            }
        }
    }

    /// 行の並びの位置 `first_line` から `count` 行を、区間ごとに原本から読んで位置の順に並べる（ADR 0027）。足した行は
    /// セルの無い行にする（`Document` が表の幅まで空のセルで埋める）。切り詰めたレコードの番号は位置に直す。
    fn read_lines(
        &self,
        first_line: u64,
        count: usize,
    ) -> Result<(Vec<Vec<String>>, Vec<CutRecord>), ScanFailure> {
        let mut cells: Vec<Vec<String>> = Vec::with_capacity(count);
        let mut cuts = Vec::new();
        let wanted = u64::try_from(count).unwrap_or(u64::MAX);
        for run in self.document.runs_from(first_line, wanted) {
            let line = first_line.saturating_add(u64::try_from(cells.len()).unwrap_or(u64::MAX));
            match run {
                OrderRun::Source { first, count: rows } => {
                    let length = rows.map_or(count_left(wanted, &cells), widen_count);
                    let (part, part_cuts) = self.reader.read_count(first, length)?.into_parts();
                    cuts.extend(moved_cuts(part_cuts, first, line));
                    cells.extend(part);
                }
                OrderRun::Added(rows) => cells.extend(rows.iter().map(|_| Vec::new())),
            }
        }
        Ok((cells, cuts))
    }

    /// 選んだ範囲が変わっていれば集計を作り直す（前の裏の計算は捨てる）。見えている行の外なら選んだ行を読み
    /// （読めなければ出さない）、大きければ裏で数え始める（始められなければ読めなかったと知らせる）。
    fn refresh_totals(&mut self) {
        let (first_line, count, background) = match self.document.refresh_totals() {
            TotalsRequest::Unchanged => return,
            TotalsRequest::Settled => {
                self.totals = None;
                return;
            }
            TotalsRequest::Read { first_line, count } => (first_line, count, false),
            TotalsRequest::Background { first_line, count } => (first_line, count, true),
        };
        // 前の仕事を手放す（中止して合流する）。
        self.totals = None;
        let Ok(count) = usize::try_from(count) else {
            self.document.totals_report(SelectionTotals::Failed);
            return;
        };
        if background {
            let (bands, header) = self.document.totals_scope();
            let fold = TotalsFold::new(bands, header, self.reader.format());
            // 裏の集計は行を足し消ししていないときだけ来る（位置は原本のレコードの番号と同じ）。
            match self.reader.spawn_fold(first_line, count, fold) {
                Ok(job) => self.totals = Some(job),
                Err(_failure) => self.document.totals_report(SelectionTotals::Failed),
            }
        } else if let Ok((rows, _cuts)) = self.read_lines(first_line, count) {
            self.document.count_totals(first_line, &rows);
        }
    }

    /// 裏の集計の今の知らせ。終わっていれば結果、まだなら進み具合。仕事が無ければ `None`。
    fn totals_report(&mut self) -> Option<SelectionTotals> {
        let job = self.totals.as_mut()?;
        let report = match job.take_result() {
            Some(Ok(RangeOutcome::Finished(fold))) => SelectionTotals::Counted(fold.stats()),
            Some(Ok(RangeOutcome::Cancelled)) => SelectionTotals::Stopped,
            Some(Err(_failure)) => SelectionTotals::Failed,
            None => {
                let progress = job.progress();
                return Some(SelectionTotals::Counting(JobProgress::new(
                    progress.done(),
                    progress.total(),
                    None,
                )));
            }
        };
        self.totals = None;
        Some(report)
    }

    /// パレットが開いている間の意図。どのキーもパレットへ渡し、表には届けない（Issue #41）。
    fn palette_intent(&mut self, intent: Intent, view: WindowView) -> Option<ClipboardText> {
        let current = self.preferences.theme().position();
        let outcome = {
            let open = self.palette.as_mut()?;
            let state = open.state_mut();
            if let Intent::Point { at, .. } = intent {
                let layout = PaletteLayout::new(view, state);
                match layout.item_at(at) {
                    Some(index) => state.click(index, current),
                    None if layout.contains(at) => PaletteOutcome::Stay,
                    // パレットの外のクリックは Esc と同じ（試した配色も戻す）。
                    None => state.handle(Intent::Cancel, current),
                }
            } else {
                state.handle(intent, current)
            }
        };
        self.after_palette(outcome, view)
    }

    /// パレットの結果に応じて、閉じる・実行する・配色を試す・決める・戻す。
    fn after_palette(
        &mut self,
        outcome: PaletteOutcome,
        view: WindowView,
    ) -> Option<ClipboardText> {
        if self.reading_blocked(outcome) {
            return None;
        }
        match outcome {
            PaletteOutcome::Stay => None,
            PaletteOutcome::Close => {
                self.palette = None;
                None
            }
            PaletteOutcome::Run(intent) => {
                self.palette = None;
                self.intent(intent, view)
            }
            PaletteOutcome::Preview(position) => {
                self.preferences.preview_theme(ThemeState::at(position));
                None
            }
            PaletteOutcome::Commit(position) => {
                self.palette = None;
                self.commit_theme(position);
                None
            }
            PaletteOutcome::Revert(position) => {
                self.palette = None;
                self.preferences.preview_theme(ThemeState::at(position));
                None
            }
            PaletteOutcome::OpenReading(aspect) => {
                let base = self.trial.as_ref().unwrap_or(&self.document);
                let current = ReadingChoice::current(
                    aspect,
                    base.format_summary().format(),
                    base.header_record(),
                );
                if let Some(open) = self.palette.as_mut() {
                    open.state_mut().enter_reading(aspect, current);
                }
                None
            }
            PaletteOutcome::PreviewReading(choice) => {
                self.try_reading(choice);
                None
            }
            PaletteOutcome::CommitReading(choice) => {
                self.palette = None;
                self.commit_reading(choice);
                None
            }
            PaletteOutcome::RevertReading => {
                self.palette = None;
                if let Some(original) = self.trial.take() {
                    self.document = original;
                }
                None
            }
            PaletteOutcome::OpenCandidates => {
                self.open_candidates();
                None
            }
        }
    }

    /// 書き換えがある間（保存の途中も）に読み方を変えようとしたら、パレットを閉じて知らせ、`true`。
    /// 読み方を変えると原本のレコードの番号の意味が変わり、書き換えが別のセルに付く（ADR 0024）。
    fn reading_blocked(&mut self, outcome: PaletteOutcome) -> bool {
        let reading = matches!(
            outcome,
            PaletteOutcome::OpenReading(_) | PaletteOutcome::OpenCandidates
        );
        if !reading || (self.document.edits().is_empty() && !self.document.is_saving()) {
            return false;
        }
        self.palette = None;
        self.document.notify(Notice::ReadingNeedsSave);
        true
    }

    /// 位置 `position` の配色に決めて設定に残し、名前を知らせる（書けなければそちらを知らせる）。
    fn commit_theme(&mut self, position: usize) {
        let saved = self.preferences.commit_theme(ThemeState::at(position));
        let theme = self.preferences.theme();
        let notice = Notice::Theme {
            name: theme.name(),
            position: theme.position(),
        };
        self.notify_saved(notice, saved);
    }

    /// 判別の候補を選ぶ段へ入る。今の文字コードで標本を読み直して候補を作り、今の方言を選んだ状態にする（Issue #3）。
    /// 標本を読めなければ候補は無い。
    fn open_candidates(&mut self) {
        let base = self.trial.as_ref().unwrap_or(&self.document);
        let format = base.format_summary().format();
        let candidates = self
            .reader
            .candidates(format.encoding())
            .unwrap_or_default();
        if let Some(open) = self.palette.as_mut() {
            open.state_mut()
                .enter_candidates(candidates, ReadingChoice::dialect(format.dialect()));
        }
    }

    /// 読み方の候補 `choice` で先頭の行を読み直し、その画面に替えて試す（元の画面は戻せるように残す・Issue #52）。
    /// 切れない組か読めなければ、画面は替えずに `false`。
    fn try_reading(&mut self, choice: ReadingChoice) -> bool {
        let base = self.trial.as_ref().unwrap_or(&self.document);
        let Some((format, header)) =
            choice.apply(base.format_summary().format(), base.header_record())
        else {
            return false;
        };
        let Ok(rows) = self.reader.read_trial(format) else {
            return false;
        };
        let summary = FormatSummary::new(format, base.format_summary().line_endings());
        let mut trial = Document::open(base.title().to_owned(), summary, header, rows.cells());
        trial.mark_cut(rows.cuts().to_vec());
        trial.set_detail(base.detail());
        let original = core::mem::replace(&mut self.document, trial);
        if self.trial.is_none() {
            self.trial = Some(original);
        }
        true
    }

    /// 読み方の候補 `choice` で開き直す。試した画面を残し、索引を作り直して、裏の集計は捨てる。開き直せなければ元に戻す。
    fn commit_reading(&mut self, choice: ReadingChoice) {
        if !self.try_reading(choice) {
            if let Some(original) = self.trial.take() {
                self.document = original;
            }
            return;
        }
        match self.reader.reopen(self.document.format_summary().format()) {
            Ok(job) => {
                self.job = job;
                self.trial = None;
                self.totals = None;
            }
            Err(_failure) => {
                if let Some(original) = self.trial.take() {
                    self.document = original;
                }
            }
        }
    }
}

impl Session for Viewer {
    fn close(&mut self) -> bool {
        // 尋ねる画面はパレットより前に出す。
        self.palette = None;
        self.document.ask_close()
    }

    fn finished(&self) -> bool {
        self.document.close_state() == CloseState::Finished
    }

    fn frame(&self, view: WindowView) -> Frame {
        let frame = self
            .document
            .frame(view, self.preferences.theme().palette());
        let Some(open) = &self.palette else {
            return frame;
        };
        let mut commands = frame.into_commands();
        commands.extend(
            open.state()
                .paint(view, open.colors(), self.document.detail()),
        );
        Frame::new(view.size(), commands)
    }

    fn zoom(&self) -> ZoomLevel {
        self.preferences.settings().zoom()
    }

    fn refresh(&mut self, view: WindowView) {
        self.finish_saving();
        let report = self.report();
        // 読み方を試している間の索引の知らせは、元の読み方の画面のもの。
        // 索引を作り終えたら今の表示の行を読み直す（試している読み方の画面は、元の読み方の索引では読めないので読み直さない）。
        let reread = match self.trial.as_mut() {
            Some(original) => {
                original.report(report);
                DocumentEffect::Stay
            }
            None => self.document.report(report),
        };
        self.effect(reread);
        // 索引を作り終えたら、ファイルの終わりまでの選択を数え直す（変わっていなければ何もしない）。
        self.refresh_totals();
        if let Some(totals) = self.totals_report() {
            self.document.totals_report(totals);
        }
        let ticks = self.ticker.take();
        if ticks > 0 {
            let effect = self.document.tick(ticks, view);
            self.effect(effect);
            self.sync_ticker();
        }
    }

    /// 意図を画面の状態に取り込み、求められたことをして、選んだ範囲の集計を作り直す。
    fn intent(&mut self, intent: Intent, view: WindowView) -> Option<ClipboardText> {
        if self.palette.is_some() {
            return self.palette_intent(intent, view);
        }
        if intent == Intent::OpenGoTo {
            let colors = self.preferences.theme().palette();
            self.palette = Some(OpenPalette::new(CommandPalette::open_go_to(), colors));
            return None;
        }
        if intent == Intent::OpenPalette {
            // 開いたときの配色でパレットを描く（配色を試しても、パレットの色は動かない）。
            let colors = self.preferences.theme().palette();
            self.palette = Some(OpenPalette::new(CommandPalette::open(), colors));
            return None;
        }
        if let Intent::Zoom(step) = intent {
            // 文字の大きさも配色と同じく、ファイルの画面の状態ではないので設定として持つ（ADR 0020）。
            let zoom = self.zoom().stepped(step);
            let saved = self.preferences.set_zoom(zoom);
            self.notify_saved(Notice::Zoom(zoom), saved);
            return None;
        }
        if let Intent::Theme(step) = intent {
            // 配色はファイルの画面の状態ではないので設定として持つ。替えたら名前を知らせる。
            let saved = self.preferences.step_theme(step);
            let theme = self.preferences.theme();
            let notice = Notice::Theme {
                name: theme.name(),
                position: theme.position(),
            };
            self.notify_saved(notice, saved);
            return None;
        }
        let effect = self.document.apply(intent, view);
        if intent == Intent::ToggleDetail
            && self.preferences.set_detail(self.document.detail()).is_err()
        {
            self.document.notify(Notice::SettingsUnsaved);
        }
        let copied = self.effect(effect);
        self.refresh_totals();
        self.sync_ticker();
        copied
    }

    fn compose(&mut self, preedit: Option<Preedit>, _view: WindowView) {
        // パレットの欄は英字とローマ字で絞るので、確定前の文字列は描かない（確定は 1 字ずつ渡す）。
        if self.palette.is_none() {
            self.document.compose(preedit);
        }
    }

    fn commit_text(&mut self, text: &str, view: WindowView) {
        if self.palette.is_none() {
            self.document.commit_text(text);
            return;
        }
        for character in text.chars() {
            // 字を打ってもクリップボードに置くものは出ない。
            let _clipboard = self.intent(Intent::Type(character), view);
        }
    }

    fn composition_area(&self, view: WindowView) -> Option<PixelRect> {
        self.palette
            .is_none()
            .then(|| self.document.composition_area(view))
            .flatten()
    }

    fn hover(&mut self, at: Option<PixelPoint>, view: WindowView) -> bool {
        // パレットが開いている間は、下の表の見た目を変えない。
        self.palette.is_none() && self.document.hover(at, view)
    }

    fn pointer_shape(&self) -> PointerShape {
        // パレットが開いている間は、下の表の境目に載っても矢印のまま。
        match self.palette {
            Some(_) => PointerShape::Arrow,
            None => self.document.pointer_shape(),
        }
    }
}

/// `wanted` 行のうち、まだ読んでいない行の数。
fn count_left(wanted: u64, read: &[Vec<String>]) -> usize {
    let left = wanted.saturating_sub(u64::try_from(read.len()).unwrap_or(u64::MAX));
    usize::try_from(left).unwrap_or(usize::MAX)
}

/// 行の数を `usize` にする（収まらなければ最大）。
fn widen_count(rows: u64) -> usize {
    usize::try_from(rows).unwrap_or(usize::MAX)
}

/// 原本のレコード `first` から読んだ区間の、切り詰めたレコードの番号を、位置 `line` からの位置に直す。
fn moved_cuts(cuts: Vec<CutRecord>, first: u64, line: u64) -> impl Iterator<Item = CutRecord> {
    cuts.into_iter().map(move |cut| {
        let moved = line.saturating_add(cut.record().saturating_sub(first));
        CutRecord::new(moved, cut.total_bytes())
    })
}
