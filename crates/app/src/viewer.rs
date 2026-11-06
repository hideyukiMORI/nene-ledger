//! 窓が描く相手。画面の状態と、裏の索引の仕事と、範囲の読み取りを結ぶ。

use nene_ledger_application::clipboard_text::ClipboardText;
use nene_ledger_application::close_state::CloseState;
use nene_ledger_application::command_palette::CommandPalette;
use nene_ledger_application::cut_record::CutRecord;
use nene_ledger_application::document::{Document, MAX_PASTE_CELLS};
use nene_ledger_application::document_effect::DocumentEffect;
use nene_ledger_application::format_summary::FormatSummary;
use nene_ledger_application::index_summary::IndexSummary;
use nene_ledger_application::intent::Intent;
use nene_ledger_application::job_progress::JobProgress;
use nene_ledger_application::job_report::JobReport;
use nene_ledger_application::notice::Notice;
use nene_ledger_application::palette_layout::PaletteLayout;
use nene_ledger_application::palette_mode::PaletteMode;
use nene_ledger_application::palette_outcome::PaletteOutcome;
use nene_ledger_application::paste_request::PasteRequest;
use nene_ledger_application::pointer_shape::PointerShape;
use nene_ledger_application::reading_aspect::ReadingAspect;
use nene_ledger_application::reading_choice::ReadingChoice;
use nene_ledger_application::save_problem::SaveProblem;
use nene_ledger_application::save_target::SaveTarget;
use nene_ledger_application::selection_mode::SelectionMode;
use nene_ledger_application::selection_totals::SelectionTotals;
use nene_ledger_application::session::Session;
use nene_ledger_application::settings_failure::SettingsFailure;
use nene_ledger_application::theme_state::ThemeState;
use nene_ledger_application::ticker::Ticker;
use nene_ledger_application::totals_request::TotalsRequest;
use nene_ledger_application::window_view::WindowView;
use nene_ledger_application::zoom_level::ZoomLevel;
use nene_ledger_core::find_options::FindOptions;
use nene_ledger_core::find_query::FindQuery;
use std::path::PathBuf;

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
use nene_ledger_scan::split_job::SplitJob;
use nene_ledger_store::save_job::SaveJob;

use crate::find_fold::FindFold;
use crate::open_palette::OpenPalette;
use crate::preferences::Preferences;
use crate::reader::Reader;
use crate::replace_fold::ReplaceFold;
use crate::rows::WINDOW_RECORDS;
#[cfg(not(windows))]
use crate::saving::suggested_entry;
#[cfg(windows)]
use crate::saving::suggested_target;
use crate::saving::{SavingFailure, describe, problem_of, start_saving, start_saving_as};
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
    /// 裏で回っている検索の 1 回分（ADR 0030 の決定 4）。
    finding: Option<SplitJob<FindFold>>,
    /// 裏で回っている全部の置換の数え（ADR 0030 の決定 6）。
    replacing: Option<SplitJob<ReplaceFold>>,
    /// 読み方を試している間の、元の画面（Esc で戻す・Issue #52）。
    trial: Option<Document>,
    /// 裏で回っている保存（ADR 0026 の決定 6）。
    saving: Option<SaveJob>,
    /// 裏で回っている保存が名前を付けて保存なら、その保存先（終われば窓がそこを指す・ADR 0028 の決定 5）。
    saving_as: Option<PathBuf>,
    /// 直前の意図をパレットが受けたか（パレットの項目を押して閉じた直後の 2 度目の押しを、表のダブルクリックにしない）。
    palette_took_last: bool,
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
            finding: None,
            replacing: None,
            trial: None,
            saving: None,
            saving_as: None,
            palette_took_last: false,
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

    /// 新しい表を新しい窓で開く（自分の exe を `--new` で起こす・ADR 0028 の決定 1）。今の窓は残る。
    fn open_new_window(&mut self) {
        #[cfg(windows)]
        let opened = nene_ledger_adapters_windows::new_window::open_new_window();
        #[cfg(not(windows))]
        let opened = nene_ledger_adapters_x11::new_window::open_new_window();
        if let Err(failure) = opened {
            warn(&format!("新しい表の窓を開けなかった: {failure:?}"));
            self.document.notify(Notice::NewWindowFailed);
        }
    }

    /// 名前を付けて保存の保存先を選ばせる（ADR 0028 の決定 4）。Windows は OS の画面（既にあるファイルなら OS の画面が
    /// 上書きを尋ねる）、X11 はパレットの欄（初めは今のファイルのフォルダ）。
    fn choose_save_target(&mut self) {
        #[cfg(windows)]
        {
            let (folder, name) = suggested_target(self.reader.path());
            match nene_ledger_ui_win32::desktop::choose_save_file(folder, &name) {
                Some(path) => self.start_saving_as(path),
                None => self.document.save_as_cancelled(),
            }
        }
        #[cfg(not(windows))]
        {
            let colors = self.preferences.theme().palette();
            let initial = suggested_entry(self.reader.path());
            self.palette = Some(OpenPalette::new(
                CommandPalette::open_save_as(initial),
                colors,
            ));
        }
    }

    /// 読み方の面 `aspect` を選ぶ段へ入る（今の読み方を選んだ状態・Issue #52）。
    fn open_reading(&mut self, aspect: ReadingAspect) {
        let base = self.trial.as_ref().unwrap_or(&self.document);
        let current =
            ReadingChoice::current(aspect, base.format_summary().format(), base.header_record());
        if let Some(open) = self.palette.as_mut() {
            open.state_mut().enter_reading(aspect, current);
        }
    }

    /// パレットを閉じる。名前を付けて保存の欄なら、保存をやめたことにする（保存して閉じるを選んでいれば閉じない）。
    fn close_palette(&mut self) {
        let saving_as = self
            .palette
            .as_ref()
            .is_some_and(|open| matches!(open.state().mode(), PaletteMode::SaveAs { .. }));
        self.palette = None;
        if saving_as {
            self.document.save_as_cancelled();
        }
    }

    /// パレットの欄で打った道筋に名前を付けて保存する。既にあるファイルで、まだ上書きを確かめていなければ尋ねる
    /// （ADR 0028 の決定 5・強制の 3 つ目）。フォルダや読めない道筋なら、欄を開いたまま知らせる。
    fn save_as_entered(&mut self, overwrite: bool) {
        let Some(open) = self.palette.as_mut() else {
            return;
        };
        let path = PathBuf::from(open.state().query().trim());
        match nene_ledger_store::stamping::target_state(&path) {
            Ok(SaveTarget::Existing(_)) if !overwrite => open.state_mut().ask_overwrite(),
            Ok(SaveTarget::Existing(_) | SaveTarget::Absent) => {
                self.palette = None;
                self.start_saving_as(path);
            }
            Err(_kind) => self.document.notify(Notice::SaveFailed(SaveProblem::Other)),
        }
    }

    /// `path` へ名前を付けて保存を裏で始める。始められなければ理由を知らせる。
    fn start_saving_as(&mut self, path: PathBuf) {
        match start_saving_as(self.document.changes(), &self.reader, &path) {
            Ok(job) => {
                self.saving = Some(job);
                self.saving_as = Some(path);
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
        let saved_as = self.saving_as.take();
        if let Err(failure) = outcome {
            let failure = SavingFailure::Save(failure);
            warn(&describe(&failure));
            self.document.save_failed(problem_of(&failure));
            return;
        }
        // 前の索引の仕事と集計は、置き換える前のファイルのもの。捨ててから開き直す。名前を付けて保存したなら、
        // 開けたときだけ窓は保存先を指す（題名も替える・ADR 0028 の決定 5）。
        self.totals = None;
        let title = saved_as.as_deref().map(crate::opening::title_of);
        match self.reader.reopen_saved(saved_as) {
            Ok(job) => {
                self.job = job;
                let reread = match (title, self.reader.line_endings()) {
                    (Some(title), Some(endings)) => self.document.saved_as(title, endings),
                    (Some(title), None) => {
                        let endings = self.document.format_summary().line_endings();
                        self.document.saved_as(title, endings)
                    }
                    (None, _) => self.document.saved(),
                };
                self.effect(reread);
            }
            Err(failure) if title.is_some() => {
                // 保存先には書けたが開けない。窓は前の原本と差分のまま（保存先を指したことにしない）。
                warn(&format!(
                    "名前を付けて保存した後に開けなかった: {failure:?}"
                ));
                self.document.reopen_failed_after_save_as();
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
                    IndexSummary::new(index.records(), index.unclosed_quotes().first().copied())
                        .with_line_endings(index.line_endings());
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
            DocumentEffect::SaveAs => {
                self.choose_save_target();
                None
            }
            DocumentEffect::Find => {
                self.start_find_pass();
                None
            }
            DocumentEffect::Replace => {
                self.start_replace_count();
                None
            }
            DocumentEffect::StopReplacing => {
                if let Some(job) = &self.replacing {
                    job.cancel();
                }
                None
            }
            DocumentEffect::StopFinding => {
                if let Some(job) = &self.finding {
                    job.cancel();
                }
                None
            }
            DocumentEffect::StopSaving => {
                if let Some(job) = &self.saving {
                    job.cancel();
                }
                None
            }
        }
    }

    /// 表に貼る。貼る行を読んでから書く（書く前の値と原本の値を履歴に残す・ADR 0029）。読めなければ貼らない。
    fn paste_into_table(&mut self, text: &str, view: WindowView) {
        let PasteRequest::Read { first_line, count } = self.document.paste(text) else {
            return;
        };
        let lines = usize::try_from(count)
            .ok()
            .and_then(|count| self.read_lines(first_line, count).ok());
        let Some((rows, cuts)) = lines else {
            self.document.paste_unread();
            return;
        };
        let effect = self.document.paste_rows(first_line, (&rows, &cuts), view);
        let _clipboard = self.effect(effect);
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

    /// 検索の欄を開く。前に探した文字と条件を入れておく（ADR 0030 の決定 1・3）。
    fn open_find(&mut self) {
        let (text, options) = self
            .document
            .find_query()
            .map_or((String::new(), FindOptions::standard()), |query| {
                (String::from(query.text()), query.options())
            });
        let colors = self.preferences.theme().palette();
        self.palette = Some(OpenPalette::new(
            CommandPalette::open_find(&text, options),
            colors,
        ));
    }

    /// 検索の欄で Enter（`forward`）か Shift+Enter：打った文字を今の条件で探し始める（欄は開いたまま）。
    fn find_entered(&mut self, forward: bool) -> Option<ClipboardText> {
        let query = self.palette.as_ref().map(|open| {
            let state = open.state();
            FindQuery::new(state.query(), state.find_options())
        })?;
        let effect = self.document.find(query, forward);
        self.effect(effect)
    }

    /// 検索の次の 1 回分を裏で探し始める（前の 1 回分は手放す）。始められなければ止まったと知らせる。
    fn start_find_pass(&mut self) {
        self.finding = None;
        let Some((scope, pass)) = self.document.take_find_pass() else {
            return;
        };
        let fold = FindFold::new(
            scope,
            (self.reader.format(), self.document.header_record()),
            self.document.changes().order().clone(),
            pass.bound(),
        );
        let count = usize::try_from(pass.count()).unwrap_or(usize::MAX);
        match self.reader.spawn_split(pass.first(), count, fold) {
            Ok(job) => self.finding = Some(job),
            Err(_failure) => self.document.find_stopped(true),
        }
    }

    /// 全部の置換の当たりを、原本の全体から裏で数え始める（前の数えは手放す）。始められなければ止まったと知らせる。
    fn start_replace_count(&mut self) {
        self.replacing = None;
        let Some(pass) = self.document.take_replace_pass() else {
            return;
        };
        let limit = usize::try_from(MAX_PASTE_CELLS).unwrap_or(usize::MAX);
        let fold = ReplaceFold::new(
            pass,
            (self.reader.format(), self.document.header_record()),
            self.document.changes().order().clone(),
            limit,
        );
        match self.reader.spawn_split(0, usize::MAX, fold) {
            Ok(job) => self.replacing = Some(job),
            Err(_failure) => self.document.replace_stopped(true),
        }
    }

    /// 裏の数えが終わっていれば書き換えを渡し、まだなら進み具合を渡す。
    fn replacing_report(&mut self) {
        let Some(job) = self.replacing.as_mut() else {
            return;
        };
        match job.take_result() {
            Some(Ok(RangeOutcome::Finished(fold))) => {
                let (counted, too_many) = fold.result();
                self.document.replace_counted(counted, too_many);
            }
            Some(Ok(RangeOutcome::Cancelled)) => self.document.replace_stopped(false),
            Some(Err(_failure)) => self.document.replace_stopped(true),
            None => {
                let progress = job.progress();
                self.document
                    .replace_progress(progress.done(), progress.total());
                return;
            }
        }
        self.replacing = None;
    }

    /// 裏の検索の 1 回分が終わっていれば答えを渡し（続きがあれば次を始める）、まだなら進み具合を渡す。
    fn finding_report(&mut self, view: WindowView) {
        let Some(job) = self.finding.as_mut() else {
            return;
        };
        let effect = match job.take_result() {
            Some(Ok(RangeOutcome::Finished(fold))) => self.document.found(fold.found(), view),
            Some(Ok(RangeOutcome::Cancelled)) => {
                self.document.find_stopped(false);
                DocumentEffect::Stay
            }
            Some(Err(_failure)) => {
                self.document.find_stopped(true);
                DocumentEffect::Stay
            }
            None => {
                let progress = job.progress();
                self.document
                    .find_progress(progress.done(), progress.total());
                return;
            }
        };
        self.finding = None;
        let _clipboard = self.effect(effect);
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
                self.close_palette();
                None
            }
            PaletteOutcome::SaveAs { overwrite } => {
                self.save_as_entered(overwrite);
                None
            }
            PaletteOutcome::Find { forward } => self.find_entered(forward),
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
                self.open_reading(aspect);
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
        if !reading {
            return false;
        }
        // 新しい表の原本は BOM だけなので、読み方を変えると表が化け、新しい表でなくなる（ADR 0028 の決定 3）。
        if self.document.is_untitled() {
            self.palette = None;
            self.document.notify(Notice::ReadingNeedsFile);
            return true;
        }
        if !self.document.has_unsaved() && !self.document.is_saving() {
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
        // 裏の保存の進み具合（見込みは原本の大きさ・列を足せば少し越えるので、割合は 99% で止める・SAVE-02）。
        if let Some(job) = &self.saving {
            self.document
                .save_progress(job.written(), self.reader.source_length());
        }
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
        self.finding_report(view);
        self.replacing_report();
        let ticks = self.ticker.take();
        if ticks > 0 {
            let effect = self.document.tick(ticks, view);
            self.effect(effect);
            self.sync_ticker();
        }
    }

    /// 意図を画面の状態に取り込み、求められたことをして、選んだ範囲の集計を作り直す。
    fn intent(&mut self, intent: Intent, view: WindowView) -> Option<ClipboardText> {
        let palette_took_last = core::mem::replace(&mut self.palette_took_last, false);
        // パレットを押して閉じた直後の 2 度目の押しは、表では 1 度の押し（直し始めない）。
        let intent = if let Intent::DoubleClick { at } = intent
            && (palette_took_last || self.palette.is_some())
        {
            Intent::Point {
                at,
                mode: SelectionMode::Collapse,
            }
        } else {
            intent
        };
        if self.palette.is_some() {
            self.palette_took_last = true;
            return self.palette_intent(intent, view);
        }
        if intent == Intent::OpenFind {
            self.open_find();
            return None;
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
        if intent == Intent::NewTable {
            self.open_new_window();
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

    fn paste(&mut self, text: &str, view: WindowView) {
        if self.palette.is_none() {
            self.paste_into_table(text, view);
            self.refresh_totals();
            return;
        }
        // パレットの欄には 1 行の字として打つ（改行とタブは打たない）。
        for character in text.chars().filter(|character| !character.is_control()) {
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
