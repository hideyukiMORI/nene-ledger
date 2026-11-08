//! 開いたファイルの画面の状態。

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::ops::{Range, RangeInclusive};

use crate::access::Access;
use crate::added_row_shape::AddedRowShape;
use crate::bar_hit::BarHit;
use crate::bar_marks::BarMarks;
use crate::body_mapping::{body_count, body_of, record_of};
use crate::cell_display::single_line;
use crate::cell_editor::CellEditor;
use crate::cell_position::CellPosition;
use crate::cell_restore::CellRestore;
use crate::cell_row::CellRow;
use crate::chrome_painter::ChromePainter;
use crate::client_bands::ClientBands;
use crate::clipboard_text::ClipboardText;
use crate::close_choice::CloseChoice;
use crate::close_prompt::ClosePrompt;
use crate::close_state::CloseState;
use crate::column_change::ColumnChange;
use crate::column_place::ColumnPlace;
use crate::column_shift::ColumnShift;
use crate::columns_notice::ColumnsNotice;
use crate::cursor_moves::step_cursor;
use crate::cursor_step::CursorStep;
use crate::cut_mark::CutMark;
use crate::cut_record::CutRecord;
use crate::detail_content::DetailContent;
use crate::detail_painter::DetailPainter;
use crate::detail_pane::DetailPane;
use crate::detail_place::DetailPlace;
use crate::document_effect::DocumentEffect;
use crate::edit_commit::EditCommit;
use crate::edit_history::EditHistory;
use crate::edit_mode::EditMode;
use crate::edit_step::EditStep;
use crate::editing_cell::EditingCell;
use crate::find_bound::FindBound;
use crate::find_notice::FindNotice;
use crate::find_pass::FindPass;
use crate::find_run::FindRun;
use crate::find_scope::FindScope;
use crate::fixed_parts::FixedParts;
use crate::format_summary::FormatSummary;
use crate::frame_painter::FramePainter;
use crate::go_to_target::GoToTarget;
use crate::index_summary::IndexSummary;
use crate::intent::Intent;
use crate::job_progress::JobProgress;
use crate::job_report::JobReport;
use crate::job_state::JobState;
use crate::line_hit::LineHit;
use crate::notice::Notice;
use crate::paste_notice::PasteNotice;
use crate::paste_request::PasteRequest;
use crate::pending_paste::PendingPaste;
use crate::pointer_hold::PointerHold;
use crate::pointer_shape::PointerShape;
use crate::press_kind::PressKind;
use crate::ranges::Ranges;
use crate::read_only_notice::ReadOnlyNotice;
use crate::replace_failure::ReplaceFailure;
use crate::replace_notice::ReplaceNotice;
use crate::replace_run::ReplaceRun;
use crate::row_band::RowBand;
use crate::row_change::RowChange;
use crate::row_place::RowPlace;
use crate::rows_notice::RowsNotice;
use crate::save_problem::SaveProblem;
use crate::scroll::Scroll;
use crate::scroll_bar::ScrollBar;
use crate::scroll_orientation::ScrollOrientation;
use crate::scrollbar_painter::ScrollbarPainter;
use crate::scrollbars::Scrollbars;
use crate::scrolling::{clamp_first, reveal_row, reveal_row_below};
use crate::selection::Selection;
use crate::selection_mode::SelectionMode;
use crate::selection_stats::SelectionStats;
use crate::selection_totals::SelectionTotals;
use crate::status_line::{
    cursor_status, format_status, group_digits, index_status, notice_status, skipped_note,
    totals_status,
};
use crate::table_changes::TableChanges;
use crate::table_extent::TableExtent;
use crate::table_view::TableView;
use crate::thumb_state::ThumbState;
use crate::totals_request::TotalsRequest;
use crate::track_side::TrackSide;
use crate::tsv::{parse_tsv, tsv};
use crate::view_row::ViewRow;
use crate::window_view::WindowView;
use nene_ledger_core::cell_change::CellChange;
use nene_ledger_core::cell_edits::CellEdits;
use nene_ledger_core::cell_value::CellValue;
use nene_ledger_core::column_id::ColumnId;
use nene_ledger_core::column_order::ColumnOrder;
use nene_ledger_core::column_widths::{ColumnWidths, MAX_RESIZED_COLUMNS, MIN_COLUMN_COLUMNS};
use nene_ledger_core::display_width::display_columns;
use nene_ledger_core::draw_command::DrawCommand;
use nene_ledger_core::find_query::FindQuery;
use nene_ledger_core::frame::Frame;
use nene_ledger_core::glyph_metrics::GlyphMetrics;
use nene_ledger_core::header_rows::HeaderRows;
use nene_ledger_core::line_ending::LineEnding;
use nene_ledger_core::line_ending_counts::LineEndingCounts;
use nene_ledger_core::order_run::OrderRun;
use nene_ledger_core::palette::Palette;
use nene_ledger_core::pixel_point::PixelPoint;
use nene_ledger_core::pixel_rect::PixelRect;
use nene_ledger_core::pixel_size::PixelSize;
use nene_ledger_core::pointer_point::PointerPoint;
use nene_ledger_core::preedit::Preedit;
use nene_ledger_core::row_id::RowId;
use nene_ledger_core::row_order::RowOrder;
use nene_ledger_core::scroll_axis::ScrollAxis;
use nene_ledger_core::table_layout::{GRID_LINE, TableLayout};
use nene_ledger_core::text_role::TextRole;

/// 開いたファイルの画面の状態。題名・固定した見出しの行・列の幅・本体の見えている行・索引の一括処理の状態。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    title: String,
    format: FormatSummary,
    /// 上に固定した見出しの行（続いた 0〜3 行・ADR 0034）。
    header: Vec<ViewRow>,
    widths: ColumnWidths,
    view: TableView,
    first_column: usize,
    /// 左に固定した列の数（表示の位置で数える・VIEW-01）。横の表示はこの次の列から。
    frozen: usize,
    selection: Selection,
    detail: DetailPane,
    notice: Option<Notice>,
    /// 見えている行のうち、上限で切り詰めたレコード（GitHub Issue #36）。
    cuts: Vec<CutRecord>,
    totals: SelectionTotals,
    others: Vec<Selection>,
    order: Vec<usize>,
    counted: Option<Vec<Selection>>,
    hold: Option<PointerHold>,
    hovered: Option<ScrollOrientation>,
    index: JobState,
    /// 利用者がドラッグで変えた幅（元の列 → 半角の字数・Issue #64）。
    custom: BTreeMap<usize, u32>,
    /// 指が見出しの列の境目の上にあるか（カーソルの形）。
    grip: bool,
    /// 行の並び（ADR 0024 の層 1・ADR 0027）。
    rows: RowOrder,
    /// 列の並び（ADR 0024 の層 1・ADR 0027）。表の列は、この並びの順（`t`）で数える。
    columns: ColumnOrder,
    /// 読んだ行の中で最も多い原本のセルの数（表の列の数は、ここから列の並びで数える）。
    source_width: usize,
    /// 見出しの各行の、原本の順のセル（列を足し消ししたら並べ直す）。
    header_source: Vec<Vec<String>>,
    /// 書き換えたセル（ADR 0024 の層 2）。画面の表は原本 ＋ これから作る派生値。
    edits: CellEdits,
    /// 編集中のセル（ADR 0026）。
    editor: Option<CellEditor>,
    /// Ctrl+Enter で書いている間の欄（書けなければ戻す・EDIT-02・GitHub Issue #203）。
    filling: Option<CellEditor>,
    /// 裏で保存している間（直すことと保存を受けない）。
    saving: bool,
    /// 閉じる手順（ADR 0026 の決定 7）。
    closing: CloseState,
    /// 元に戻す・やり直すの履歴（UNDO-01）。保存の後も残す。
    history: EditHistory,
    /// 開いた直後（保存した直後）の行と列の並び。ふだんは原本のまま、新しい表は足した 1 行と 3 列
    /// （未保存かの基準・ADR 0028 の決定 2）。
    baseline: (Vec<OrderRun>, Vec<OrderRun>),
    /// 原本のファイルが無い新しい表か（Ctrl+S が名前を付けて保存になる・ADR 0028 の決定 4）。
    untitled: bool,
    /// 裏の保存の進み具合（書いたバイト数・書く見込みのバイト数・SAVE-02）。
    save_progress: Option<(u64, u64)>,
    /// 原本の行を読むのを待っている貼り付け（ADR 0029・`paste_rows` で書く）。
    pasting: Option<PendingPaste>,
    /// 切り取りの枠（ADR 0029 の決定 5）。表を変える操作・Esc・編集で消える。
    cut: Option<CutMark>,
    /// 前に探した問い（F3 で続けて探す・ADR 0030 の決定 1）。
    find_query: Option<FindQuery>,
    /// 進んでいる検索。
    finding: Option<FindRun>,
    /// 見つけたセルが下に入ったら困る、表に重なった面（窓の座標・開いている検索と置換のパレット・GitHub Issue #286）。
    cover: Option<PixelRect>,
    /// 進んでいる全部の置換（数えている間と、件数を見せている間・ADR 0030 の決定 6）。
    replacing: Option<ReplaceRun>,
    /// 求めたコピーが切り取りなら、その範囲（`copy_text` で枠を作る）。
    cutting: Option<Selection>,
    /// 読み取り専用か（表の値・行・列を変える操作を止める・EDIT-13・GitHub Issue #238）。
    access: Access,
    /// 最後に意図を取り込んだときの窓（窓の行が持つ列を決める・ADR 0033 の決定 1）。まだ無ければ `None`。
    window: Option<WindowView>,
}

/// 窓を知る前（開いた直後）に、窓の行が持つ元の列の数（左から・高い解像度の画面に下限の幅の列を並べても足りる数・
/// ADR 0033 の決定 1）。
pub const FIRST_HELD_COLUMNS: usize = 1_024;

/// 状態の行の右端に出すパレットの案内。
const PALETTE_HINT: &str = "Ctrl+Shift+P でコマンド";

/// 入力の状態の案内（ADR 0026 の決定 2）。
const EDITING_HINT_ENTERING: &str = "入力中 · Enter 確定 · Esc やめる · F2 編集へ";

/// 編集の状態の案内。
const EDITING_HINT_EDITING: &str = "編集中 · Enter 確定 · Esc やめる · F2 入力へ";

/// 一度にコピーする行の上限。
pub const MAX_COPY_ROWS: u64 = 100_000;
/// 見出しの列の境目として押せる、境目の左右の画素（100% のとき・Issue #64）。
const COLUMN_GRIP: u32 = 4;
/// 列を移すときの、落とす位置の線の太さ（画素・100% のとき・ほかの帯と同じ 3）。
const DROP_LINE: u32 = 3;

/// 列全体・すべての選択の行の終わり（ファイルの終わりまで・数えるときは本体の最後の行で切る）。
pub const TO_END: u64 = u64::MAX;

/// セルの上で押してからドラッグとみなすまでの指の動き（画素・100% のとき・アクセシビリティ）。
const DRAG_THRESHOLD: u32 = 6;

/// 溝を押し続けたとき、繰り返し始めるまでの時計の刻み（50ms × 10 = 500ms）。
pub const REPEAT_DELAY_TICKS: u64 = 10;

/// その場で集計する行の上限。超えれば裏で集計する（Issue #43）。
pub const MAX_TOTAL_ROWS: u64 = 100_000;
/// 一度にコピーする文字列の上限（バイト）。X11 の 1 つの要求に収める（INCR はまだ）。
pub const MAX_COPY_BYTES: usize = 4 * 1024 * 1024;

/// 1 回の貼り付けで書くセルの上限（ADR 0029 の決定 4）。
pub const MAX_PASTE_CELLS: u128 = 1_000_000;
/// 1 回の貼り付けで書く行の上限（貼る行は全部の列を原本から読むので、コピーと同じ上限・GitHub Issue #153）。
pub const MAX_PASTE_ROWS: u64 = MAX_COPY_ROWS;
/// 1 回の貼り付けで書く列の上限（Excel の列の数と同じ・GitHub Issue #153）。
pub const MAX_PASTE_COLUMNS: usize = 16_384;
/// 貼るクリップボードの文字列の上限（バイト・ADR 0029 の決定 4）。
pub const MAX_PASTE_BYTES: usize = 64 * 1024 * 1024;

/// 足した列の幅（半角の字数・ADR 0027）。
const NEW_COLUMN_WIDTH: u32 = 8;

/// 新しい表の行の数（ADR 0028 の決定 2）。
const BLANK_ROWS: u64 = 1;

/// 新しい表の列の数（ADR 0028 の決定 2）。
const BLANK_COLUMNS: usize = 3;

impl Document {
    /// 題名（タイトルバーに書く）・読み方の要約（状態の行に書く）・見出しのレコードの番号（無ければ `None`）・
    /// 先頭から読んだ行 `rows` から始める。
    /// 列の幅は、見出しと先頭の行から決めて固定する（VIEW-02）。カーソルは左上のセル。詳細ペインは開いておく（初めて使う人が
    /// 開閉のキーに気づけるよう、見出しの行に案内を出す）。索引はまだ知らせが無い。
    #[must_use]
    pub fn open(
        title: String,
        format: FormatSummary,
        header: Option<HeaderRows>,
        rows: Vec<CellRow>,
    ) -> Self {
        let header = header_rows_in(header, &rows);
        let source_width = rows.iter().map(CellRow::len).max().unwrap_or(0);
        let view = TableView::from_records(0, rows, lines_of(&header));
        let header_source = header.iter().map(|row| row.values().to_strings()).collect();
        let mut widths = ColumnWidths::fit(&[]);
        let shown: Vec<&ViewRow> = header.iter().chain(view.rows()).collect();
        widen_to(&mut widths, &shown);
        let first_data = header.first().map_or(0, ViewRow::line);
        Self {
            title,
            format,
            widths,
            header,
            view,
            first_column: 0,
            frozen: 0,
            // 前書きの行があれば、カーソルはその後ろのデータの 1 行目（本体の「前書きの行の数」番目・OPEN-11・GitHub
            // Issue #273）。表示の先頭は動かさない（前書きも見える）。
            selection: Selection::at(CellPosition::new(first_data, 0)),
            detail: DetailPane::Open,
            notice: None,
            cuts: Vec::new(),
            totals: SelectionTotals::Hidden,
            others: Vec::new(),
            order: Vec::new(),
            counted: None,
            hold: None,
            hovered: None,
            index: JobState::Waiting,
            custom: BTreeMap::new(),
            grip: false,
            rows: RowOrder::whole(),
            columns: ColumnOrder::whole(),
            source_width,
            header_source,
            edits: CellEdits::empty(),
            editor: None,
            filling: None,
            saving: false,
            closing: CloseState::Open,
            history: EditHistory::empty(),
            baseline: (
                RowOrder::whole().runs().to_vec(),
                ColumnOrder::whole().runs().to_vec(),
            ),
            untitled: false,
            save_progress: None,
            pasting: None,
            cut: None,
            find_query: None,
            finding: None,
            cover: None,
            replacing: None,
            cutting: None,
            access: Access::Writable,
            window: None,
        }
    }

    /// 原本の無い新しい表（ADR 0028 の決定 2）。原本は空で、行の並びに足した 1 行、列の並びに足した 3 列。見出しの行は無い。
    /// この形を開いた直後とする（何も打たなければ未保存でない・Undo で消えない）。
    #[must_use]
    pub fn blank(title: String, format: FormatSummary) -> Self {
        let mut document = Self::open(title, format, None, Vec::new());
        let _row = document.rows.insert(0, BLANK_ROWS);
        let _columns = document.columns.insert(0, BLANK_COLUMNS);
        document.columns_inserted(0, &[NEW_COLUMN_WIDTH; BLANK_COLUMNS]);
        document.baseline = (
            document.rows.runs().to_vec(),
            document.columns.runs().to_vec(),
        );
        document.untitled = true;
        document
    }

    /// 書き換えたセル（保存で store に渡す・ADR 0025）。
    #[must_use]
    pub const fn edits(&self) -> &CellEdits {
        &self.edits
    }

    /// 保存に渡す差分の全部（行の並び・セルの書き換え・足した行の書き方・ADR 0027 の決定 5）。
    #[must_use]
    pub fn changes(&self) -> TableChanges<'_> {
        let ending = self
            .format
            .line_endings()
            .dominant()
            .unwrap_or(LineEnding::CrLf);
        let shape = AddedRowShape::new(self.widths.columns().len(), ending);
        TableChanges::new((&self.rows, &self.columns), &self.edits, shape)
    }

    /// 保存していない変更があるか（セルの書き換え・行の並び・列の並び）。未保存の印・保存・閉じるときの確認・
    /// 読み方を変えてよいかの判断は、すべてこれで決める（GitHub Issue #107）。編集中のセルは確定するまで数えない。
    #[must_use]
    pub fn has_unsaved(&self) -> bool {
        !self.edits.is_empty()
            || self.rows.runs() != self.baseline.0.as_slice()
            || self.columns.runs() != self.baseline.1.as_slice()
    }

    /// 編集中のセル。
    #[must_use]
    pub const fn editor(&self) -> Option<&CellEditor> {
        self.editor.as_ref()
    }

    /// 裏で保存しているか。
    #[must_use]
    pub const fn is_saving(&self) -> bool {
        self.saving
    }

    /// 閉じる手順の状態。`Finished` なら UI が窓を閉じる。
    #[must_use]
    pub const fn close_state(&self) -> CloseState {
        self.closing
    }

    /// 閉じるよう求められた（× のボタン・Alt+F4・窓の管理者）。編集中なら確定する。
    /// 未保存の書き換えが無く、保存もしていなければ `true`（すぐ閉じてよい）。あれば尋ねる画面を出して `false`。
    pub fn ask_close(&mut self) -> bool {
        if self.closing == CloseState::Finished {
            return true;
        }
        self.commit_edit();
        if !self.has_unsaved() && !self.saving {
            self.closing = CloseState::Finished;
            return true;
        }
        if self.closing == CloseState::Open {
            self.closing = CloseState::Asking;
        }
        false
    }

    /// 尋ねる画面を出している間の意図。尋ねていなければ `None`（ふだんの意図として続ける）。
    fn close_intent(&mut self, intent: Intent, window: WindowView) -> Option<DocumentEffect> {
        match self.closing {
            CloseState::Open => return None,
            // 保存が終わるまで待つ（ほかの操作は受けない）。
            CloseState::AfterSave | CloseState::Finished => return Some(DocumentEffect::Stay),
            CloseState::Asking => {}
        }
        let choice = match intent {
            Intent::Type('s' | 'S') | Intent::Confirm => Some(CloseChoice::Save),
            Intent::Type('d' | 'D') => Some(CloseChoice::Discard),
            Intent::Cancel => Some(CloseChoice::Cancel),
            Intent::Point { at, .. } | Intent::DoubleClick { at } => {
                ClosePrompt::new(window).choice_at(at)
            }
            Intent::Scroll(_)
            | Intent::Move { .. }
            | Intent::Drag { .. }
            | Intent::Release
            | Intent::GoTo(_)
            | Intent::OpenGoTo
            | Intent::SelectAll
            | Intent::SelectRows
            | Intent::SelectColumns
            | Intent::MoveColumns(_)
            | Intent::ResetColumns
            | Intent::ResetWidths
            | Intent::FreezeColumns
            | Intent::UnfreezeColumns
            | Intent::Copy
            | Intent::Cut
            | Intent::OpenFind { .. }
            | Intent::FindAgain { .. }
            | Intent::FindToggle(_)
            | Intent::ReplaceAll
            | Intent::ToggleDetail
            | Intent::ToggleReadOnly
            | Intent::Theme(_)
            | Intent::Zoom(_)
            | Intent::OpenPalette
            | Intent::Type(_)
            | Intent::Erase
            | Intent::Edit
            | Intent::Delete
            | Intent::BreakLine
            | Intent::FillSelection
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
            | Intent::RemoveHere => None,
        };
        Some(choice.map_or(DocumentEffect::Stay, |choice| self.choose_close(choice)))
    }

    /// 尋ねる画面で `choice` を選んだ。
    fn choose_close(&mut self, choice: CloseChoice) -> DocumentEffect {
        match choice {
            CloseChoice::Cancel => {
                self.closing = CloseState::Open;
                DocumentEffect::Stay
            }
            CloseChoice::Discard => {
                self.closing = CloseState::Finished;
                DocumentEffect::Stay
            }
            CloseChoice::Save => {
                self.closing = CloseState::AfterSave;
                if self.saving {
                    // 今の保存が終われば閉じる。
                    DocumentEffect::Stay
                } else if self.untitled {
                    DocumentEffect::SaveAs
                } else {
                    DocumentEffect::Save
                }
            }
        }
    }

    /// 裏の保存を始めた（`DocumentEffect::Save` を受けて合成ルートが呼ぶ）。
    pub fn begin_saving(&mut self) {
        self.saving = true;
        self.save_progress = None;
    }

    /// 裏の保存が `written` バイトを書いた（見込みは `total` バイト・SAVE-02）。状態の行に割合を出す。
    pub fn save_progress(&mut self, written: u64, total: u64) {
        if self.saving {
            self.save_progress = Some((written, total));
        }
    }

    /// 保存して、置き換えた後のファイルを同じ読み方で開き直した。書き換えを空にし、索引は作り直しを待つ。
    /// カーソルと表示の位置は保ち、今の位置から読み直させる。
    pub fn saved(&mut self) -> DocumentEffect {
        self.saving = false;
        // 履歴は残し、触ったセルのファイルの値を保存した値に進める（保存の後も元に戻せる）。
        // 行を足し消しした手があれば、保存の後は原本のレコードの番号が変わるので、履歴を残せない（ADR 0027）。
        if self.history.has_row_steps() || !self.rows.is_whole() || !self.columns.is_whole() {
            self.history = EditHistory::empty();
        } else {
            self.history.saved(&self.edits);
        }
        self.edits = CellEdits::empty();
        // 保存したファイルは、今の行と列の並びで書かれている。見出しは新しい位置と列の順に置き直す。
        if !self.columns.is_whole() {
            self.header_source = self
                .header_source
                .iter()
                .map(|cells| self.arranged(&CellRow::new(cells)))
                .collect();
            self.source_width = self.columns.len(self.source_width);
        }
        if let Some(lines) = self.header_line() {
            self.header = core::mem::take(&mut self.header)
                .into_iter()
                .zip(lines.lines())
                .map(|(header, line)| {
                    ViewRow::from_values(line, header.values().clone())
                        .with_row(RowId::Source(line))
                })
                .collect();
        }
        self.rows = RowOrder::whole();
        self.columns = ColumnOrder::whole();
        self.baseline = (self.rows.runs().to_vec(), self.columns.runs().to_vec());
        self.editor = None;
        self.index = JobState::Waiting;
        self.notice = Some(Notice::Saved);
        if self.closing == CloseState::AfterSave {
            self.closing = CloseState::Finished;
        }
        DocumentEffect::Read {
            first_body: self.view.first_body(),
        }
    }

    /// 名前を付けて保存して、保存先のファイルを同じ読み方で開き直した。題名を `title` に替え、窓は保存先を指す
    /// （ADR 0028 の決定 5）。後は [`Self::saved`] と同じ。
    /// 改行の内訳は保存先の先頭から数え直した `line_endings`（新しい表は、書くまで改行が無い）。
    pub fn saved_as(&mut self, title: String, line_endings: LineEndingCounts) -> DocumentEffect {
        self.title = title;
        self.format = FormatSummary::new(self.format.format(), line_endings);
        self.untitled = false;
        self.saved()
    }

    /// 名前を付けて保存して書けたが、保存先を開けなかった。窓は前の原本と差分のまま（題名・書き換え・新しい表かは
    /// 変えない）。保存して閉じると選んでいれば、書けたので閉じる。
    pub fn reopen_failed_after_save_as(&mut self) {
        self.saving = false;
        if self.closing == CloseState::AfterSave {
            self.closing = CloseState::Finished;
        }
        self.notice = Some(Notice::ReopenFailed);
    }

    /// 名前を付けて保存の保存先を選ばずにやめた。保存して閉じると選んでいたなら、閉じずに戻る。
    pub fn save_as_cancelled(&mut self) {
        if self.closing == CloseState::AfterSave && !self.saving {
            self.closing = CloseState::Open;
        }
    }

    /// 原本のファイルが無い新しい表か。
    #[must_use]
    pub const fn is_untitled(&self) -> bool {
        self.untitled
    }

    /// 保存できなかった。書き換えは残す。
    pub fn save_failed(&mut self, problem: SaveProblem) {
        self.saving = false;
        // 保存して閉じると選んでいても、保存できなければ閉じない（書き換えを失わない）。
        if self.closing == CloseState::AfterSave {
            self.closing = CloseState::Open;
        }
        self.notice = Some(Notice::SaveFailed(problem));
    }

    /// Ctrl+S。編集中なら確定してから、書き換えがあれば保存させる。
    fn save_request(&mut self) -> DocumentEffect {
        self.commit_edit();
        if self.untitled {
            return DocumentEffect::SaveAs;
        }
        if !self.has_unsaved() {
            self.notice = Some(Notice::NothingToSave);
            return DocumentEffect::Stay;
        }
        DocumentEffect::Save
    }

    /// 題名（タイトルバーに書く）。
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// 窓の名前（自前のタイトルバーと OS の窓の名前）。未保存の書き換えがあれば頭に印を付ける（WIN-01・ADR 0026 の決定 7）。
    #[must_use]
    pub fn window_name(&self) -> String {
        // 読み取り専用の印はファイル名の直後に付ける（「名前 — NeNe Ledger」ならアプリ名の前・EDIT-13）。
        let title = match (self.is_read_only(), self.title.split_once(" — ")) {
            (false, _) => self.title.clone(),
            (true, Some((name, app))) => format!("{name}（読み取り専用） — {app}"),
            (true, None) => format!("{}（読み取り専用）", self.title),
        };
        if self.has_unsaved() {
            format!("● {title}")
        } else {
            title
        }
    }

    /// 読み方の要約（文字コード・方言・改行の内訳）。
    #[must_use]
    pub fn format_summary(&self) -> FormatSummary {
        self.format
    }

    /// 固定した見出しの行（上から・無ければ空・ADR 0034）。
    #[must_use]
    pub fn header(&self) -> &[ViewRow] {
        &self.header
    }

    /// 列 `column`（表示だけの入れ替えの前の列）の名前。見出しの各行のその列の値を上から「 / 」でつなぐ（空の値は飛ばす・ADR 0034 の決定 3）。
    /// 見出しが無いか、どの行にも値が無ければ `None`。
    #[must_use]
    pub fn column_name(&self, column: usize) -> Option<String> {
        let names: Vec<&str> = self
            .header
            .iter()
            .filter_map(|row| row.cells().get(column))
            .filter(|name| !name.is_empty())
            .collect();
        (!names.is_empty()).then(|| names.join(" / "))
    }

    /// 本体の見えている行。
    #[must_use]
    pub fn view(&self) -> &TableView {
        &self.view
    }

    /// 列の幅（字数）。
    #[must_use]
    pub fn widths(&self) -> &ColumnWidths {
        &self.widths
    }

    /// 索引の状態。
    #[must_use]
    pub fn index(&self) -> JobState {
        self.index
    }

    /// 読んだ原本の行 `rows` のセルの数を覚える（表の列の数を数える元）。
    fn see_source(&mut self, rows: &[CellRow]) {
        let widest = rows.iter().map(CellRow::len).max().unwrap_or(0);
        self.source_width = self.source_width.max(widest);
    }

    /// 表の列の同一性（表の列の順）。
    fn table_ids(&self) -> Vec<ColumnId> {
        self.columns.columns(self.columns.len(self.source_width))
    }

    /// 表の `column` 列目の同一性（書き換えの差分を引く鍵）。
    fn column_id(&self, column: usize) -> ColumnId {
        self.columns
            .column_at(column)
            .unwrap_or(ColumnId::Source(column))
    }

    /// 原本の順のセル `cells` を、表の列の順に並べた行（列を足し消ししていなければそのまま）。足した列は空のセル。
    /// 原本に無いセルは、後ろに書くセルが無ければ書かない（「セルが無い」のまま）。セルごとに `String` を作らず 1 本の
    /// 文字列で作る（GitHub Issue #22）。
    fn arranged_row(&self, cells: &CellRow) -> CellRow {
        if self.columns.is_whole() {
            return cells.clone();
        }
        let ids = self.table_ids();
        let last = ids.iter().rposition(|id| match id {
            ColumnId::Source(source) => *source < cells.len(),
            ColumnId::Added(_) => true,
        });
        CellRow::collect(
            ids.iter()
                .take(last.map_or(0, |last| last.saturating_add(1)))
                .map(|id| match id {
                    ColumnId::Source(source) => cells.get(*source).unwrap_or(""),
                    ColumnId::Added(_) => "",
                }),
        )
    }

    /// [`Self::arranged_row`] と同じ並びを、読んだ行 `cells` を持ち主ごと受け取って作る（列を足し消ししていなければ写さずに
    /// そのまま使う・窓の行を 2 組持たない・GitHub Issue #22）。
    fn arranged_owned(&self, cells: CellRow) -> CellRow {
        if self.columns.is_whole() {
            cells
        } else {
            self.arranged_row(&cells)
        }
    }

    /// [`Self::arranged_row`] と同じ並びを、セルごとの `String` で（書き換えを重ねるコピーと集計の写し）。
    fn arranged(&self, cells: &CellRow) -> Vec<String> {
        if self.columns.is_whole() {
            return cells.to_strings();
        }
        self.arranged_row(cells).to_strings()
    }

    /// 本体の `body` 番目の行の、行の並びの位置（読み始める所・前書きと見出しも数える）。
    #[must_use]
    pub fn line_of(&self, body: u64) -> u64 {
        record_of(body, self.header_line())
    }

    /// 行の並びの位置 `first_line` から `count` 行の区間（合成ルートが区間ごとに原本を読み、足した行は空の行にして
    /// 位置の順に `show`・`copy_text`・`count_totals` に渡す・ADR 0027）。
    #[must_use]
    pub fn runs_from(&self, first_line: u64, count: u64) -> Vec<OrderRun> {
        self.rows.slice(first_line, count)
    }

    /// 位置 `first_line` から読んだ行 `rows` に差し替える（動いた先）。見出しは除き、列が増えていればその列の幅だけを足す。
    /// 足した行は空のセルで表の幅まで埋める。見出しを除いて行が残らなければ差し替えない。
    pub fn show(&mut self, first_line: u64, rows: Vec<CellRow>) {
        self.see_source(&rows);
        let columns = self.columns.len(self.source_width);
        let shown: Vec<ViewRow> = rows
            .into_iter()
            .zip(first_line..)
            .map(|(cells, line)| {
                let row = self.rows.row_at(line).unwrap_or(RowId::Source(line));
                match row {
                    RowId::Source(_) => ViewRow::from_values(line, self.arranged_owned(cells)),
                    RowId::Added(_) => ViewRow::new(line, &vec![String::new(); columns]),
                }
                .with_row(row)
            })
            .collect();
        let view = TableView::from_rows(first_line, shown, self.header_line());
        // 行を全部消した表は、空のまま見せる（前の行を残さない）。
        let emptied = self
            .records()
            .is_some_and(|records| self.body_lines(records) == 0);
        if view.rows().is_empty() && !emptied {
            return;
        }
        // 見出しも渡す（まだ幅を決めていない列の仮の幅と、初めて窓に入った列の幅に見出しの名前を含める・ADR 0033 の決定 3）。
        let fresh: Vec<&ViewRow> = self.header.iter().chain(view.rows()).collect();
        widen_to(&mut self.widths, &fresh);
        self.view = view;
        self.cuts.clear();
    }

    /// 窓 `view` の本体に全部入る行の数（タイトルバー・縁・見出しの行・詳細ペイン・下の状態の行を除く・少なくとも 1）。1 画面の移動の量。
    #[must_use]
    pub fn page_rows(&self, view: WindowView) -> u64 {
        let row = TableLayout::row_height_for(view.content_metrics()).max(1);
        // 上に固定した行（見出しか、見出しが無ければ列の番号）の分を除く。
        let body = self
            .bars(view)
            .body()
            .size()
            .height()
            .saturating_sub(row.saturating_mul(self.fixed_rows()));
        u64::from((body / row).max(1))
    }

    /// 詳細ペインの開閉。
    #[must_use]
    pub fn detail(&self) -> DetailPane {
        self.detail
    }

    /// 詳細ペインの開閉を、残した設定のとおりにする（開いた直後）。
    pub fn set_detail(&mut self, detail: DetailPane) {
        self.detail = detail;
    }

    /// 選択。
    #[must_use]
    pub fn selection(&self) -> Selection {
        self.selection
    }

    /// 表示の最初の列（横に動く列の最初。固定した列はその左に残る）。
    #[must_use]
    pub fn first_column(&self) -> usize {
        self.first_column
    }

    /// 左に固定した列の数（表示の位置で数える・VIEW-01）。
    #[must_use]
    pub fn frozen_columns(&self) -> usize {
        self.frozen
    }

    /// 窓 `window` での利用者の意図を取り込み、合成ルートがすることを返す。カーソル・選択・横の表示はすぐ変える。
    /// 縦の表示の先頭（本体の何番目か）が変わるなら、そこから読ませる。末尾は索引を作り終えて行の数が分かってから。
    /// コピーなら選択の行を読ませる（大きすぎれば知らせるだけ）。前の知らせは消す。
    pub fn apply(&mut self, intent: Intent, window: WindowView) -> DocumentEffect {
        self.window = Some(window);
        let effect = self.apply_intent(intent, window);
        // 横に動いて、見えている列を窓の行が持っていなければ、今の位置から読み直させる（ADR 0033 の決定 1）。
        if effect == DocumentEffect::Stay && !self.holds_shown_columns(window) {
            return DocumentEffect::Read {
                first_body: self.view.first_body(),
            };
        }
        effect
    }

    /// 窓 `window` の大きさに合わせて、表の右の端が窓に収まるのに左へ隠れた列があれば、最初の列を、最後の列が収まる
    /// いちばん左まで戻す（狭い窓で右へ動いた後に広げると、間の列が出なくなった・GitHub Issue #265）。
    pub fn fit_columns(&mut self, window: WindowView) {
        self.window = Some(window);
        let Some(last) = self.widths.columns().len().checked_sub(1) else {
            return;
        };
        if self.first_column <= self.frozen {
            return;
        }
        let width = self.bars(window).body().size().width();
        let fitting = self
            .layout(self.first_column, window.content_metrics())
            .first_column_showing(last, self.frozen, width);
        if fitting < self.first_column {
            self.first_column = fitting;
        }
    }

    /// 窓の行を読んだ後でも、見えている列を窓の行が持っていなければ、読み直す本体の位置（今の表示の先頭）。合成ルートは
    /// `Read` を受けて読んだ後にこれを見る（行の数が分かる前に表の終わりより下を読むと、読んだ行が無く前の窓の行が残る・
    /// ADR 0033 の決定 1）。足りていれば `None`。
    #[must_use]
    pub fn columns_reread(&self) -> Option<u64> {
        let window = self.window?;
        (!self.holds_shown_columns(window)).then(|| self.view.first_body())
    }

    /// 窓の行が読むべき元の列の範囲（見えている列の左右に、見えている列の数だけの余白・左に固定した列・カーソルの列・
    /// ADR 0033 の決定 1）。合成ルートが窓の行を読むときに使う。列を足し消ししていれば `None`（全部の列）。窓をまだ
    /// 知らなければ、左から [`FIRST_HELD_COLUMNS`] 列。
    #[must_use]
    pub fn columns_to_read(&self) -> Option<Vec<Range<usize>>> {
        match self.window {
            Some(window) => self.held_columns(window, true),
            None => self
                .columns
                .is_whole()
                .then(|| alloc::vec![0..FIRST_HELD_COLUMNS]),
        }
    }

    /// 窓の行が持つべき元の列の範囲。`margin` なら左右に余白を足す（読むとき）。偽なら見えている列だけ（足りているかを
    /// 確かめるとき）。列を足し消ししていれば `None`（全部の列・表の列と元の列の対応を窓の行で引くため）。
    fn held_columns(&self, window: WindowView, margin: bool) -> Option<Vec<Range<usize>>> {
        if !self.columns.is_whole() {
            return None;
        }
        let width = self.bars(window).body().size().width();
        let layout = self.layout(self.first_column, window.content_metrics());
        let visible = (self.first_column..layout.column_count())
            .take_while(|&column| layout.column_left(column).is_some_and(|left| left < width))
            .count()
            .max(1);
        let (start, end) = if margin {
            (
                self.first_column.saturating_sub(visible),
                self.first_column.saturating_add(visible.saturating_mul(2)),
            )
        } else {
            (self.first_column, self.first_column.saturating_add(visible))
        };
        let cursor = self.selection.cursor().column();
        let mut sources: Vec<usize> = (0..self.frozen)
            .chain(start..end)
            .chain(core::iter::once(cursor))
            .map(|display| self.source_column(display))
            .collect();
        sources.sort_unstable();
        sources.dedup();
        Some(spans_of(&sources))
    }

    /// 見えている列（固定した列・カーソルの列を含む）を、窓の行がどれも持っているか。列を足し消ししていれば、全部の列を
    /// 持つ行だけが足りる。
    fn holds_shown_columns(&self, window: WindowView) -> bool {
        let rows = self.view.rows();
        match self.held_columns(window, false) {
            None => rows.iter().all(|row| row.values().is_whole()),
            Some(spans) => rows.iter().all(|row| {
                spans
                    .iter()
                    .all(|span| span.clone().all(|column| row.values().holds(column)))
            }),
        }
    }

    /// 意図を取り込む（[`Self::apply`] の中身）。
    fn apply_intent(&mut self, intent: Intent, window: WindowView) -> DocumentEffect {
        if let Some(effect) = self.close_intent(intent, window) {
            return effect;
        }
        self.notice = None;
        if !keeps_cut(intent) {
            self.cut = None;
        }
        if let Some(effect) = self.replace_intent(intent) {
            return effect;
        }
        // 探している間の Esc は、探すのをやめる（ADR 0030 の決定 4）。
        if self.finding.is_some() && intent == Intent::Cancel && self.editor.is_none() {
            return DocumentEffect::StopFinding;
        }
        if let Intent::FindAgain { forward } = intent {
            return self.find_again(forward);
        }
        // 保存の途中の Esc は、保存をやめる（SAVE-02・ADR 0025 の施主の判断 4）。
        if self.saving && intent == Intent::Cancel && self.editor.is_none() {
            return DocumentEffect::StopSaving;
        }
        if self.saving && changes_the_table(intent) {
            self.notice = Some(Notice::StillSaving);
            return DocumentEffect::Stay;
        }
        if changes_when_writable(intent) && self.refuses_change() {
            return DocumentEffect::Stay;
        }
        if intent == Intent::Save {
            return self.save_request();
        }
        if let Intent::DoubleClick { at } = intent {
            return self.double_click(at, window);
        }
        if intent == Intent::SaveAs {
            self.commit_edit();
            return DocumentEffect::SaveAs;
        }
        if let Intent::Undo | Intent::Redo = intent {
            return self.undo_or_redo(intent == Intent::Redo, window);
        }
        if intent == Intent::FillSelection {
            return self.fill_selection();
        }
        match self.edit_intent(intent) {
            Some(intent) => self.apply_to_table(intent, window),
            None => DocumentEffect::Stay,
        }
    }

    /// 編集の欄が受けなかった意図を、表に取り込む。
    fn apply_to_table(&mut self, intent: Intent, window: WindowView) -> DocumentEffect {
        if let Some(effect) = self.early_effect(intent, window) {
            return effect;
        }
        let page = self.page_rows(window);
        let current = self.view.first_body();
        let target = match intent {
            Intent::Copy => return self.copy_request(),
            Intent::Cut => return self.cut_request(),
            // 配色・文字の大きさ・パレット・字の入力・検索の欄は合成ルートが、続けて探すのは `apply` が受ける。
            Intent::OpenFind { .. }
            | Intent::FindAgain { .. }
            | Intent::FindToggle(_)
            | Intent::ReplaceAll
            | Intent::Theme(_)
            | Intent::Zoom(_)
            | Intent::OpenPalette
            | Intent::Type(_)
            | Intent::Erase
            | Intent::Confirm
            | Intent::Cancel
            | Intent::Drag { .. }
            | Intent::DoubleClick { .. }
            | Intent::Release
            | Intent::OpenGoTo
            | Intent::Edit
            | Intent::Delete
            | Intent::BreakLine
            | Intent::FillSelection
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
            | Intent::RemoveHere => current,
            Intent::SelectAll
            | Intent::SelectRows
            | Intent::SelectColumns
            | Intent::MoveColumns(_)
            | Intent::ResetColumns
            | Intent::ResetWidths
            | Intent::FreezeColumns
            | Intent::UnfreezeColumns => self.arrange(intent, window),
            Intent::GoTo(target) => self.jump(target, page, window),
            Intent::ToggleDetail => self.toggle_detail(current, page),
            Intent::ToggleReadOnly => self.toggle_read_only(current),
            Intent::Scroll(scroll) => self.wheel(scroll, current, window),
            Intent::Move { step, mode } => self.move_cursor((step, mode), page, window),
            Intent::Point { at, mode } => {
                self.press_cell(at, mode, window);
                current
            }
        };
        self.go_to(target, page)
    }

    /// 詳細ペインを開閉する。ペインの分だけ表が狭まるので、カーソルが見えるように縦の表示を動かす（`current` は今の最初の
    /// 本体の行・`page` は 1 画面の行の数）。動いた先の最初の行を返す。
    /// 読み取り専用にする / 外す。直している途中なら、先に確定する（打った字は失わない）。
    /// 縦の表示の先頭 `current` は動かさない。
    fn toggle_read_only(&mut self, current: u64) -> u64 {
        self.commit_edit();
        self.access = self.access.toggled();
        self.notice = Some(Notice::ReadOnly(match self.access {
            Access::ReadOnly => ReadOnlyNotice::On,
            Access::Writable => ReadOnlyNotice::Off,
        }));
        current
    }

    /// 開いた原本を保存で置き換えられない理由 `blocked` があれば、読み取り専用で始めて理由を知らせる（FILE-04・
    /// GitHub Issue #240）。外せば直せる（保存はもう一度確かめる）。
    pub fn start_read_only_if_blocked(&mut self, blocked: Option<ReplaceFailure>) {
        let notice = match blocked {
            None => return,
            Some(ReplaceFailure::Locked) => ReadOnlyNotice::OpenedLocked,
            Some(ReplaceFailure::Denied | ReplaceFailure::Interrupted | ReplaceFailure::Other) => {
                ReadOnlyNotice::OpenedDenied
            }
        };
        self.access = Access::ReadOnly;
        self.notice = Some(Notice::ReadOnly(notice));
    }

    /// 読み取り専用か（EDIT-13）。
    #[must_use]
    pub const fn is_read_only(&self) -> bool {
        matches!(self.access, Access::ReadOnly)
    }

    /// 読み取り専用なら、表を変えようとしたことを知らせて `true`（呼び手は何もしない）。
    fn refuses_change(&mut self) -> bool {
        let read_only = self.is_read_only();
        if read_only {
            self.notice = Some(Notice::ReadOnly(ReadOnlyNotice::Refused));
        }
        read_only
    }

    fn toggle_detail(&mut self, current: u64, page: u64) -> u64 {
        self.detail = self.detail.toggled();
        reveal_row(current, self.selection.cursor().body(), page)
    }

    /// Esc：ドラッグの途中ならやめ、選んだ範囲を数えていれば止める。どちらでもなければ `None`。
    fn cancel_effect(&mut self, window: WindowView) -> Option<DocumentEffect> {
        if let Some(effect) = self.cancel_drag(window) {
            return Some(effect);
        }
        if let SelectionTotals::Counting(_) = self.totals {
            return Some(DocumentEffect::StopTotals);
        }
        None
    }

    /// 表の意図のうち、先に受けるもの（ドラッグと裏の集計をやめる Esc・スクロールバーの上の指・行と列の足し消し）。
    fn early_effect(&mut self, intent: Intent, window: WindowView) -> Option<DocumentEffect> {
        if intent == Intent::Cancel
            && let Some(effect) = self.cancel_effect(window)
        {
            return Some(effect);
        }
        match intent {
            Intent::InsertRows(place) => Some(self.insert_rows(place, window)),
            Intent::InsertColumns(place) => Some(self.insert_columns(place, window)),
            Intent::RemoveColumns => Some(self.remove_columns(window)),
            Intent::RemoveHere if self.columns_chosen() => Some(self.remove_columns(window)),
            Intent::RemoveRows | Intent::RemoveHere => Some(self.remove_rows(window)),
            Intent::InsertHere if self.columns_chosen() => {
                Some(self.insert_columns(ColumnPlace::Left, window))
            }
            Intent::InsertHere => Some(self.insert_rows(RowPlace::Above, window)),
            Intent::Scroll(_)
            | Intent::Move { .. }
            | Intent::Point { .. }
            | Intent::DoubleClick { .. }
            | Intent::Drag { .. }
            | Intent::Release
            | Intent::GoTo(_)
            | Intent::OpenGoTo
            | Intent::SelectAll
            | Intent::SelectRows
            | Intent::SelectColumns
            | Intent::MoveColumns(_)
            | Intent::ResetColumns
            | Intent::ResetWidths
            | Intent::FreezeColumns
            | Intent::UnfreezeColumns
            | Intent::Copy
            | Intent::Cut
            | Intent::OpenFind { .. }
            | Intent::FindAgain { .. }
            | Intent::FindToggle(_)
            | Intent::ReplaceAll
            | Intent::ToggleDetail
            | Intent::ToggleReadOnly
            | Intent::Theme(_)
            | Intent::Zoom(_)
            | Intent::OpenPalette
            | Intent::Type(_)
            | Intent::Erase
            | Intent::Confirm
            | Intent::Cancel
            | Intent::Edit
            | Intent::Delete
            | Intent::BreakLine
            | Intent::FillSelection
            | Intent::Advance(_)
            | Intent::Save
            | Intent::SaveAs
            | Intent::NewTable
            | Intent::Paste
            | Intent::Undo
            | Intent::Redo => self.pointer_intent(intent, window),
        }
    }

    /// 編集中の矢印。欄の中で動ければ `None`、欄の端なら確定して表の動きの意図を返す。
    fn arrow_in_field(&mut self, step: CursorStep, mode: SelectionMode) -> Option<Intent> {
        if let Some(editor) = self.editor.as_mut()
            && editor.arrow(step, mode == SelectionMode::Extend)
        {
            return None;
        }
        self.commit_edit();
        Some(Intent::Move { step, mode })
    }

    /// 編集の意図（ADR 0026 の決定 1・2）。欄で受けたら `None`。表の意図として続けるなら、その意図を返す
    /// （確定して動くときは、確定してから動きの意図を返す）。
    fn edit_intent(&mut self, intent: Intent) -> Option<Intent> {
        let Some(editor) = self.editor.as_mut() else {
            return self.start_edit(intent);
        };
        match intent {
            Intent::Type(character) => editor.type_character(character),
            Intent::Erase => editor.erase_before(),
            Intent::Delete => editor.erase_after(),
            Intent::BreakLine => editor.break_line(),
            Intent::Edit => editor.toggle_mode(),
            Intent::SelectAll => editor.select_all(),
            Intent::Cancel => self.editor = None,
            // 🔴 欄の中の字のコピーと切り取りはまだ（OS のクリップボードへ渡す経路を、別の Issue で足す）。
            Intent::Copy | Intent::Cut => {}
            Intent::Move { step, mode } => return self.arrow_in_field(step, mode),
            Intent::Confirm => return Some(self.committed_move(CursorStep::Down)),
            Intent::Advance(step) => return Some(self.committed_move(step)),
            // 別のセル・別の行への移動と、表の並びを変える操作と、続けて探すのは、確定してから（保存と元に戻すは `apply` が先に受ける）。
            Intent::FindAgain { .. }
            | Intent::FillSelection
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
            | Intent::RemoveHere
            | Intent::Point { .. }
            | Intent::DoubleClick { .. }
            | Intent::GoTo(_)
            | Intent::OpenGoTo
            | Intent::OpenPalette
            | Intent::SelectRows
            | Intent::SelectColumns
            | Intent::MoveColumns(_)
            | Intent::ResetColumns
            | Intent::ResetWidths
            | Intent::FreezeColumns
            | Intent::UnfreezeColumns => {
                self.commit_edit();
                return Some(intent);
            }
            // 表示だけを動かすものは、編集を続けたまま。
            Intent::OpenFind { .. }
            | Intent::FindToggle(_)
            | Intent::ReplaceAll
            | Intent::Scroll(_)
            | Intent::Drag { .. }
            | Intent::Release
            | Intent::ToggleDetail
            | Intent::ToggleReadOnly
            | Intent::Theme(_)
            | Intent::Zoom(_) => return Some(intent),
        }
        None
    }

    /// 編集を確定し、`step` の向きへ動く意図を返す（Enter・Tab・Shift+Tab・Shift+Enter）。
    fn committed_move(&mut self, step: CursorStep) -> Intent {
        self.commit_edit();
        Intent::Move {
            step,
            mode: SelectionMode::Collapse,
        }
    }

    /// 編集していないときの編集の意図。打ち始め・F2 で編集に入り、Delete でカーソルのセルを空にする。
    /// Enter・Tab などは動くだけ。
    /// 2 度目の押し（ADR 0026 の決定 1）。1 度押したのと同じに扱い、押した所がカーソルの本体のセルなら、押して始めた
    /// ドラッグを終えて、そのセルを編集の状態（元の値・カーソルは末尾）で直し始める。保存の途中は直し始めない。
    fn double_click(&mut self, at: PixelPoint, window: WindowView) -> DocumentEffect {
        let effect = self.apply(
            Intent::Point {
                at,
                mode: SelectionMode::Collapse,
            },
            window,
        );
        let on_cursor = self.cell_at(at, window) == Some(self.selection.cursor());
        if on_cursor && self.editor.is_none() && !self.saving && self.closing == CloseState::Open {
            self.release(window);
            self.begin_edit(EditMode::Editing);
        }
        effect
    }

    /// 打った字 `character` で入力を始める（セルの値を打った字で置き換える）。
    fn type_into_new_edit(&mut self, character: char) {
        if self.begin_edit(EditMode::Entering)
            && let Some(editor) = self.editor.as_mut()
        {
            editor.type_character(character);
        }
    }

    fn start_edit(&mut self, intent: Intent) -> Option<Intent> {
        match intent {
            Intent::Type(character) => {
                self.type_into_new_edit(character);
                None
            }
            Intent::Edit => {
                self.begin_edit(EditMode::Editing);
                None
            }
            Intent::Delete => {
                self.empty_cursor_cell();
                None
            }
            Intent::BreakLine | Intent::FillSelection => None,
            Intent::Confirm => Some(plain_move(CursorStep::Down)),
            Intent::Advance(step) => Some(plain_move(step)),
            Intent::Scroll(_)
            | Intent::Move { .. }
            | Intent::Point { .. }
            | Intent::DoubleClick { .. }
            | Intent::Drag { .. }
            | Intent::Release
            | Intent::GoTo(_)
            | Intent::OpenGoTo
            | Intent::SelectAll
            | Intent::SelectRows
            | Intent::SelectColumns
            | Intent::MoveColumns(_)
            | Intent::ResetColumns
            | Intent::ResetWidths
            | Intent::FreezeColumns
            | Intent::UnfreezeColumns
            | Intent::Copy
            | Intent::Cut
            | Intent::OpenFind { .. }
            | Intent::FindAgain { .. }
            | Intent::FindToggle(_)
            | Intent::ReplaceAll
            | Intent::ToggleDetail
            | Intent::ToggleReadOnly
            | Intent::Theme(_)
            | Intent::Zoom(_)
            | Intent::OpenPalette
            | Intent::Erase
            | Intent::Cancel
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
            | Intent::RemoveHere => Some(intent),
        }
    }

    /// Delete：カーソルのセルを空にする（1 手）。
    /// 🔴 選んだ範囲の全部を空にするのは一括の入力（EDIT-02）の Issue で。今はカーソルのセルだけ。
    fn empty_cursor_cell(&mut self) {
        if self.begin_edit(EditMode::Entering) {
            self.commit_edit();
        }
    }

    /// カーソルのセルの編集を始める。見えていない行・切り詰めたレコード・セルの無い所なら始めない（知らせる）。
    /// 始めたら `true`。
    fn begin_edit(&mut self, mode: EditMode) -> bool {
        // 読み取り専用なら直し始めない（打つ・F2・ダブルクリックのどれでも・EDIT-13）。
        if self.refuses_change() {
            return false;
        }
        // セルを直し始めたら、切り取りの枠は消える（ADR 0029 の決定 5）。
        self.cut = None;
        let cursor = self.selection.cursor();
        let column = self.source_column(cursor.column());
        // 窓の行がカーソルの列を持っていなければ（読み直す前）、始めない（「セルが無い」と取り違えない・ADR 0033 の決定 1）。
        let Some((line, row, source)) = self
            .view_row(cursor.body())
            .filter(|row| row.values().holds(column))
            .map(|row| {
                (
                    row.line(),
                    row.row(),
                    row.values().get(column).map(String::from),
                )
            })
        else {
            return false;
        };
        if self.cuts.iter().any(|cut| cut.record() == line) {
            self.notice = Some(Notice::EditCut);
            return false;
        }
        let cell = (row, self.column_id(column));
        // 原本の行に無いセル（列の数が足りない行）は空として入る。保存では間の列を空のセルで埋めて足す
        // （ADR 0024 の施主の判断 4・5）。差分で消したセル（`Absent`）は直さない。
        let shown = match (self.edits.get(cell.0, cell.1), &source) {
            (Some(CellValue::Text(value)), _) | (None, Some(value)) => value.clone(),
            (None, None) => String::new(),
            (Some(CellValue::Absent), _) => {
                self.notice = Some(Notice::EditMissingCell);
                return false;
            }
        };
        let editor = match mode {
            EditMode::Entering => CellEditor::entering(cell, &shown, source.as_deref()),
            EditMode::Editing => CellEditor::editing(cell, &shown, source.as_deref()),
        };
        self.editor = Some(editor);
        true
    }

    /// IME の確定前の文字列を欄に出す（`None` で消す・ADR 0026 の決定 4）。編集していなければ、
    /// 空でない文字列が始まったときに入力の状態で編集に入る。尋ねる画面と保存中は受けない（確定のほうで知らせる）。
    pub fn compose(&mut self, preedit: Option<Preedit>) {
        if self.closing != CloseState::Open || self.saving {
            return;
        }
        let starting = preedit
            .as_ref()
            .is_some_and(|preedit| !preedit.text().is_empty());
        if self.editor.is_none() && !(starting && self.begin_edit(EditMode::Entering)) {
            return;
        }
        if let Some(editor) = self.editor.as_mut() {
            editor.compose(preedit);
        }
    }

    /// クリップボードの文字列 `text` を貼る（CLIP-01・ADR 0029）。編集中は、何であれ字のまま欄のカーソルに挿す（最後の
    /// 改行は除く・引用は解かない）。編集していなければ、タブ区切り（Excel の形）として読み、選んだ最初の範囲の左上から
    /// 束のまま書く（束が 1 つのセルで範囲が広ければ範囲の全部）。書く前の値と原本の値を履歴に残すため、貼る行を合成ルート
    /// に読ませる（`PasteRequest::Read` を `paste_rows` に渡す）。値は変えない（型を推し量らない・KEEP-01）。
    pub fn paste(&mut self, text: &str) -> PasteRequest {
        if self.closing != CloseState::Open || text.is_empty() {
            return PasteRequest::Done;
        }
        if self.saving {
            self.notice = Some(Notice::StillSaving);
            return PasteRequest::Done;
        }
        if self.editor.is_none() && self.refuses_change() {
            return PasteRequest::Done;
        }
        self.notice = None;
        self.pasting = None;
        // この窓の切り取りで置いた文字列で、その後に表が変わっていなければ、貼った後に元を空にする（ADR 0029 の決定 5）。
        // 断った貼り付けでは枠を残す（GitHub Issue #153）。
        let revision = self.history.revision();
        let moving = self
            .cut
            .as_ref()
            .is_some_and(|mark| mark.is_current(revision) && mark.placed(text));
        if let Some(editor) = self.editor.as_mut() {
            // 欄には字のまま挿す（引用を解かない・Excel のセルの中の貼り付けと同じ）。最後の改行だけ除く。
            let raw = text
                .strip_suffix("\r\n")
                .or_else(|| text.strip_suffix('\n'))
                .unwrap_or(text);
            editor.insert(raw);
            return PasteRequest::Done;
        }
        // 分ける前に、区切りの数で大きさを見積もる（タブだけの大きな文字列を分けてから断らない・GitHub Issue #153）。
        let separators = text
            .bytes()
            .filter(|&byte| byte == b'\t' || byte == b'\n')
            .count();
        if text.len() > MAX_PASTE_BYTES
            || u128::try_from(separators).unwrap_or(u128::MAX) > MAX_PASTE_CELLS.saturating_mul(2)
        {
            self.notice = Some(Notice::Paste(PasteNotice::TooLarge));
            return PasteRequest::Done;
        }
        let Some(mut plan) = self.paste_plan(parse_tsv(text)) else {
            return PasteRequest::Done;
        };
        let cut = self.cut.take();
        if moving && let Some(mark) = cut {
            plan = plan.moving(mark.into_clears());
        }
        self.request_rows(plan)
    }

    /// Ctrl+Enter（EDIT-02・GitHub Issue #203）：編集中の欄の文字を、今の範囲（カーソルのある範囲）のすべてのセルに書く。
    /// 書く道は貼り付けの「1 つの値は範囲を埋める」（ADR 0029 の決定 1）と同じで、1 手で戻り、上限も同じ。欄の字は引用を
    /// 解かずにそのまま 1 つの値にする（改行やタブを含んでも分けない）。範囲が 1 つのセルなら、動かずに確定するだけ。編集して
    /// いなければ何もしない。欄は書けたときに閉じ、書けなければ（上限・索引を待つ・切り詰めた行・セルが無い・読めない）
    /// 戻して、打った字を失わない。保存の途中は `apply` が先に断る。
    fn fill_selection(&mut self) -> DocumentEffect {
        let Some(editor) = self.editor.as_ref() else {
            return DocumentEffect::Stay;
        };
        let range = self.ranges().active();
        if range.top_left() == range.bottom_right() {
            self.commit_edit();
            return DocumentEffect::Stay;
        }
        let value = String::from(editor.field().text());
        self.pasting = None;
        let Some(plan) = self.paste_plan_in(range, alloc::vec![alloc::vec![value]]) else {
            return DocumentEffect::Stay;
        };
        self.filling = self.editor.take();
        match self.request_rows(plan) {
            PasteRequest::Read { first_line, count } => {
                DocumentEffect::ReadPasted { first_line, count }
            }
            PasteRequest::Done => {
                self.editor = self.filling.take();
                DocumentEffect::Stay
            }
        }
    }

    /// 待たせる貼り付け `plan` の行を読ませる（表の終わりより下は足す行なので読まない）。
    fn request_rows(&mut self, plan: PendingPaste) -> PasteRequest {
        let first_line = self.line_of(plan.top());
        // 表の終わりより下は足す行なので読まない。
        let (known, _) = self.known_body_rows();
        let count = if plan.top() < known {
            let last = plan.bottom().min(known.saturating_sub(1));
            self.line_of(last)
                .saturating_sub(first_line)
                .saturating_add(1)
        } else {
            0
        };
        self.pasting = Some(plan);
        PasteRequest::Read { first_line, count }
    }

    /// 束 `cells` を、選んだ最初の範囲のどこにどの大きさで貼るか。上限を超える・索引を待つなら知らせて `None`。
    fn paste_plan(&mut self, cells: Vec<Vec<String>>) -> Option<PendingPaste> {
        let range = self.ranges().all().next().unwrap_or(self.selection);
        self.paste_plan_in(range, cells)
    }

    /// 束 `cells` を、範囲 `range` のどこにどの大きさで貼るか。上限を超える・索引を待つなら知らせて `None`。
    fn paste_plan_in(&mut self, range: Selection, cells: Vec<Vec<String>>) -> Option<PendingPaste> {
        let (top_left, bottom_right) = (range.top_left(), range.bottom_right());
        let corner = (top_left.body(), top_left.column());
        let single = match cells.as_slice() {
            [row] => match row.as_slice() {
                [value] => Some(value.clone()),
                _ => None,
            },
            _ => None,
        };
        let plan = match single {
            Some(value) if top_left != bottom_right => {
                if bottom_right.body() == TO_END && self.records().is_none() {
                    self.notice = Some(Notice::Paste(PasteNotice::AwaitIndex));
                    return None;
                }
                let bottom = bottom_right.body().min(self.last_body());
                PendingPaste::filling(
                    value,
                    corner,
                    (
                        bottom.saturating_sub(corner.0).saturating_add(1),
                        bottom_right
                            .column()
                            .saturating_sub(corner.1)
                            .saturating_add(1),
                    ),
                )
            }
            Some(_) | None => PendingPaste::new(cells, corner),
        };
        if plan.cell_count() > MAX_PASTE_CELLS
            || plan.rows() > MAX_PASTE_ROWS
            || plan.columns() > MAX_PASTE_COLUMNS
        {
            self.notice = Some(Notice::Paste(PasteNotice::TooLarge));
            return None;
        }
        if plan.rows() == 0 || plan.columns() == 0 {
            return None;
        }
        let (known, exact) = self.known_body_rows();
        if !exact && plan.bottom() >= known {
            self.notice = Some(Notice::Paste(PasteNotice::AwaitIndex));
            return None;
        }
        Some(plan)
    }

    /// 本体の行の数と、それが表の全部か（索引を作り終えていれば全部・そうでなければ今読める所までで、その先にもあり得る）。
    fn known_body_rows(&self) -> (u64, bool) {
        if let Some(records) = self.records() {
            return (self.body_lines(records), true);
        }
        let seen = self
            .view
            .first_body()
            .saturating_add(u64::try_from(self.view.rows().len()).unwrap_or(u64::MAX));
        (self.reachable_rows().unwrap_or(0).max(seen), false)
    }

    /// `paste` が読ませた行（位置 `first_record` から読んだ `rows`・切り詰めたレコード `cuts`）に、待っている束を書く。
    /// 表の端を越えれば足りない行と列を足し、足した列・足した行・書き換えを 1 手の束として積む（ADR 0029 の決定 2・3）。
    /// 切り詰めた行か「セルが無い」にしたセルにかかれば、何も書かずに知らせる。貼った範囲を選び、カーソルは左上。
    pub fn paste_rows(
        &mut self,
        first_record: u64,
        (rows, cuts): (&[CellRow], &[CutRecord]),
        window: WindowView,
    ) -> DocumentEffect {
        let before = self.history.revision();
        let effect = self.write_pasted(first_record, (rows, cuts), window);
        self.end_filling(self.history.revision() != before);
        effect
    }

    /// Ctrl+Enter の欄を、書けたなら閉じ、書けなかったなら戻す（打った字を失わない・EDIT-02）。
    fn end_filling(&mut self, written: bool) {
        let filling = self.filling.take();
        if !written {
            self.editor = self.editor.take().or(filling);
        }
    }

    /// `paste_rows` の中身：待っている束を、読ませた行に書く。
    fn write_pasted(
        &mut self,
        first_record: u64,
        (rows, cuts): (&[CellRow], &[CutRecord]),
        window: WindowView,
    ) -> DocumentEffect {
        let Some(mut plan) = self.pasting.take() else {
            return DocumentEffect::Stay;
        };
        if self.saving || self.editor.is_some() || self.closing != CloseState::Open {
            return DocumentEffect::Stay;
        }
        let read = first_record
            ..first_record.saturating_add(u64::try_from(rows.len()).unwrap_or(u64::MAX));
        if cuts.iter().any(|cut| read.contains(&cut.record())) {
            self.notice = Some(Notice::EditCut);
            return DocumentEffect::Stay;
        }
        let mut targets_rows = self.pasted_rows(&plan, first_record, rows);
        let width = self.widths.columns().len();
        let targets = self.paste_targets(&plan, width);
        if self.hits_missing_cell(&plan, &targets_rows, (&targets, width)) {
            self.notice = Some(Notice::EditMissingCell);
            return DocumentEffect::Stay;
        }
        let mut steps = Vec::new();
        let added_columns = plan
            .left()
            .saturating_add(plan.columns())
            .saturating_sub(width);
        if added_columns > 0 {
            steps.push(EditStep::Columns(self.add_columns(width, added_columns)));
        }
        let (known, exact) = self.known_body_rows();
        let written = u64::try_from(targets_rows.len()).unwrap_or(u64::MAX);
        let added_rows = if exact {
            plan.rows().saturating_sub(written)
        } else {
            0
        };
        if added_rows > 0 {
            let (ids, change) = self.add_rows(self.line_of(known), added_rows);
            targets_rows.extend(ids.into_iter().map(|id| (id, Vec::new())));
            steps.push(EditStep::Rows(change));
        }
        let mut cells = self.paste_changes(&plan, &targets_rows, &targets);
        let moved = plan.is_move();
        cells.extend(self.uncovered_clears(&mut plan, &targets_rows, &targets));
        self.history.record_batch(steps, cells, &mut self.edits);
        if added_columns > 0 {
            self.end_columns_added((width, added_columns), plan.left(), window);
        }
        self.notice = Some(Notice::Paste(PasteNotice::Pasted {
            rows: u64::try_from(targets_rows.len()).unwrap_or(u64::MAX),
            columns: plan.columns(),
            added_rows,
            added_columns,
            moved,
        }));
        self.select_pasted(&plan, targets_rows.len(), window)
    }

    /// `paste` が求めた行を合成ルートが読めなかった。待っている束を捨てて知らせる（GitHub Issue #153）。
    pub fn paste_unread(&mut self) {
        if self.pasting.take().is_some() {
            self.notice = Some(Notice::Paste(PasteNotice::Unread));
        }
        self.end_filling(false);
    }

    /// 貼り付けで表の右の端（`width` 列目）に `count` 列を足した後。表示だけの列の入れ替えはそのまま残して後ろに並べ
    /// （入れ替えを外すと、貼った値の並びが見えていた順と変わる・GitHub Issue #153）、表示の `left` 列目へ。
    fn end_columns_added(
        &mut self,
        (width, count): (usize, usize),
        left: usize,
        window: WindowView,
    ) {
        let shown = core::mem::take(&mut self.order);
        let _read = self.after_columns_changed(left, window);
        if !shown.is_empty() {
            self.order = shown
                .into_iter()
                .chain(width..width.saturating_add(count))
                .collect();
        }
    }

    /// 位置 `first_record` から読んだ行 `rows` のうち、束 `plan` を書く本体の行（同一性と、表の列の順に並べた原本のセル）。
    fn pasted_rows(
        &self,
        plan: &PendingPaste,
        first_record: u64,
        rows: &[CellRow],
    ) -> Vec<(RowId, Vec<String>)> {
        self.body_cells(first_record, rows)
            .into_iter()
            .zip(plan.top()..)
            .take(usize::try_from(plan.rows()).unwrap_or(usize::MAX))
            .map(|(cells, body)| {
                let line = self.line_of(body);
                let row = self.rows.row_at(line).unwrap_or(RowId::Source(line));
                (row, self.arranged(cells))
            })
            .collect()
    }

    /// 束 `plan` の各列を書く表の列（表の幅 `width` より右は、これから右の端に足す列）。
    fn paste_targets(&self, plan: &PendingPaste, width: usize) -> Vec<usize> {
        (0..plan.columns())
            .map(|offset| {
                let display = plan.left().saturating_add(offset);
                if display < width {
                    self.source_column(display)
                } else {
                    display
                }
            })
            .collect()
    }

    /// 束 `plan` が、差分で「セルが無い」にしたセルにかかるか（編集と同じく直さない・書く前に確かめて途中で止めない）。
    fn hits_missing_cell(
        &self,
        plan: &PendingPaste,
        rows: &[(RowId, Vec<String>)],
        (targets, width): (&[usize], usize),
    ) -> bool {
        let cells = rows.iter().zip(0_u64..).flat_map(|((row, _), offset)| {
            targets
                .iter()
                .enumerate()
                .map(move |(column, &table)| (*row, offset, column, table))
        });
        cells
            .filter(|&(_, offset, column, table)| {
                table < width && plan.value(offset, column).is_some()
            })
            .any(|(row, _, _, table)| {
                matches!(
                    self.edits.get(row, self.column_id(table)),
                    Some(CellValue::Absent)
                )
            })
    }

    /// 行の並びの位置 `at` に、空の行を `count` 行足す。足した行と、足した手を返す（履歴には呼ぶ側が積む）。
    fn add_rows(&mut self, at: u64, count: u64) -> (Vec<RowId>, RowChange) {
        let added = self.rows.insert(at, count);
        let ids = added
            .iter()
            .filter_map(|row| match row {
                RowId::Added(id) => Some(*id),
                RowId::Source(_) => None,
            })
            .collect();
        (
            added,
            RowChange::Inserted {
                at,
                runs: vec![OrderRun::Added(ids)],
            },
        )
    }

    /// 表の `at` 列目の前に、空の列を `count` 列足す（幅は足した列の幅）。足した手を返す（履歴には呼ぶ側が積む）。
    fn add_columns(&mut self, at: usize, count: usize) -> ColumnChange {
        let ids = self
            .columns
            .insert(at, count)
            .into_iter()
            .filter_map(|column| match column {
                ColumnId::Added(id) => Some(id),
                ColumnId::Source(_) => None,
            })
            .collect();
        let widths = vec![NEW_COLUMN_WIDTH; count];
        self.columns_inserted(at, &widths);
        ColumnChange::Inserted {
            at,
            runs: vec![OrderRun::Added(ids)],
            widths,
        }
    }

    /// 束 `plan` を行 `rows`（同一性と、表の列の順に並べた原本のセル）と表の列 `targets` に書く書き換えと、書いたセルの
    /// ファイルの今の値。見えている値と同じセルは書かない。束の短い行の先のセルも書かない。
    fn paste_changes(
        &self,
        plan: &PendingPaste,
        rows: &[(RowId, Vec<String>)],
        targets: &[usize],
    ) -> Vec<(CellChange, String)> {
        rows.iter()
            .zip(0_u64..)
            .flat_map(|((row, cells), offset)| {
                targets
                    .iter()
                    .enumerate()
                    .filter_map(move |(column, &table)| {
                        let value = plan.value(offset, column)?;
                        self.pasted_cell((*row, table), cells, value)
                    })
            })
            .collect()
    }

    /// 切り取りを貼るとき、元を空にする書き換えのうち、貼った所に重ならないもの（重なった所は貼った値・ADR 0029 の決定 5）。
    fn uncovered_clears(
        &self,
        plan: &mut PendingPaste,
        rows: &[(RowId, Vec<String>)],
        targets: &[usize],
    ) -> Vec<(CellChange, String)> {
        if !plan.is_move() {
            return Vec::new();
        }
        let pasted: &PendingPaste = plan;
        let covered: BTreeSet<(RowId, ColumnId)> = rows
            .iter()
            .zip(0_u64..)
            .flat_map(|((row, _), offset)| {
                targets
                    .iter()
                    .enumerate()
                    .filter(move |&(column, _)| pasted.value(offset, column).is_some())
                    .map(move |(_, &table)| (*row, self.column_id(table)))
                    .collect::<Vec<_>>()
            })
            .collect();
        plan.take_clears()
            .into_iter()
            .filter(|(change, _)| !covered.contains(&(change.row(), change.column())))
            .collect()
    }

    /// 行 `row` の表の `table` 列目（原本のセル `cells`）を `value` にする書き換えと、そのセルのファイルの今の値。
    /// 見えている値と同じなら `None`。
    fn pasted_cell(
        &self,
        (row, table): (RowId, usize),
        cells: &[String],
        value: &str,
    ) -> Option<(CellChange, String)> {
        let id = self.column_id(table);
        let source = cells.get(table).cloned().unwrap_or_default();
        let shown = match self.edits.get(row, id) {
            Some(CellValue::Text(text)) => text.clone(),
            Some(CellValue::Absent) | None => source.clone(),
        };
        (shown != value).then(|| (CellChange::new(row, id, shown, String::from(value)), source))
    }

    /// 貼った範囲（本体の `plan.top()` 番目から `rows` 行・表示の `plan.left()` 列目から）を選び、カーソルを左上に置いて
    /// 見えるように動かす。
    fn select_pasted(
        &mut self,
        plan: &PendingPaste,
        rows: usize,
        window: WindowView,
    ) -> DocumentEffect {
        let last_column = self.widths.columns().len().saturating_sub(1);
        let left = plan.left().min(last_column);
        let right = plan
            .left()
            .saturating_add(plan.columns().saturating_sub(1))
            .min(last_column);
        let bottom = plan
            .top()
            .saturating_add(u64::try_from(rows.saturating_sub(1)).unwrap_or(u64::MAX))
            .min(self.last_body());
        self.select(CellPosition::new(bottom, right), SelectionMode::Collapse);
        self.select(CellPosition::new(plan.top(), left), SelectionMode::Extend);
        self.counted = None;
        self.first_column = self.reveal_column(left, window);
        let page = self.page_rows(window);
        let current = self.view.first_body();
        let first = clamp_first(
            current,
            reveal_row(current, plan.top(), page),
            page,
            self.extent().rows(),
        );
        DocumentEffect::Read { first_body: first }
    }

    /// 問い `query` で、カーソルのセルの次（`forward`）か前のセルから探し始める（パレットの Enter / Shift+Enter・ADR 0030
    /// の決定 1）。見えている値（原本 ＋ 書き換え ＋ 列の並び）で照らす。原本の行は合成ルートが裏で探す（`DocumentEffect::Find`）。
    /// 空の問いは何もしない。
    pub fn find(&mut self, query: FindQuery, forward: bool) -> DocumentEffect {
        if query.is_empty() || self.closing != CloseState::Open {
            return DocumentEffect::Stay;
        }
        if query.is_broken() {
            self.notice = Some(Notice::Find(FindNotice::BrokenPattern));
            return DocumentEffect::Stay;
        }
        self.notice = None;
        self.commit_edit();
        let scope = self.find_scope(query.clone());
        self.find_query = Some(query);
        let cursor = self.selection.cursor();
        let mut run = FindRun::new(
            scope,
            forward,
            (self.line_of(cursor.body()), cursor.column()),
        );
        run.set_pending(self.find_pass(&run));
        self.finding = Some(run);
        DocumentEffect::Find
    }

    /// 問い `query` を、表示の順の列と今の書き換えで照らす範囲。
    fn find_scope(&self, query: FindQuery) -> FindScope {
        let columns = (0..self.widths.columns().len())
            .map(|display| self.column_id(self.source_column(display)))
            .collect();
        FindScope::new(query, columns, self.edits.clone())
    }

    /// 置換の段の Enter：カーソルのセルが `query` に当たれば、当たった所を `replacement` に置き換えて（1 手）、次を探す。
    /// 当たらなければ次を探すだけ（ADR 0030 の決定 6）。保存中は置き換えない。
    pub fn replace_one(&mut self, query: FindQuery, replacement: &str) -> DocumentEffect {
        if query.is_empty() || self.closing != CloseState::Open {
            return DocumentEffect::Stay;
        }
        if query.is_broken() {
            self.notice = Some(Notice::Find(FindNotice::BrokenPattern));
            return DocumentEffect::Stay;
        }
        if self.saving {
            self.notice = Some(Notice::StillSaving);
            return DocumentEffect::Stay;
        }
        if self.refuses_change() {
            return DocumentEffect::Stay;
        }
        self.commit_edit();
        if let Some((change, source)) = self.replaced_at_cursor(&query, replacement) {
            self.cut = None;
            self.history.record(change, &source, &mut self.edits);
        }
        self.find(query, true)
    }

    /// カーソルのセルの当たった所を置き換える書き換えと、そのセルのファイルの今の値。当たらない・置き換えても変わらない・
    /// 見えていない・1 MiB で切り詰めた行・「セルが無い」なら `None`。
    fn replaced_at_cursor(
        &self,
        query: &FindQuery,
        replacement: &str,
    ) -> Option<(CellChange, String)> {
        let cursor = self.selection.cursor();
        let table = self.source_column(cursor.column());
        let row = self
            .view_row(cursor.body())
            .filter(|row| row.values().holds(table))?;
        if self.cuts.iter().any(|cut| cut.record() == row.line()) {
            return None;
        }
        let id = self.column_id(table);
        let file = row
            .values()
            .get(table)
            .map(String::from)
            .unwrap_or_default();
        let shown = match self.edits.get(row.row(), id) {
            Some(CellValue::Text(value)) => value.clone(),
            Some(CellValue::Absent) => return None,
            None => file.clone(),
        };
        let after = query.replaced(&shown, replacement)?;
        (after != shown).then(|| (CellChange::new(row.row(), id, shown, after), file))
    }

    /// 置換の段の全部（Alt+A）：`query` の当たりを `replacement` に置き換えるセルを、原本の行は裏で、足した行は差分から
    /// 数え始める（ADR 0030 の決定 6）。数え終えたら件数を見せ、Enter で 1 手の束として置き換える。保存中は始めない。
    pub fn replace_all(&mut self, query: FindQuery, replacement: &str) -> DocumentEffect {
        if query.is_empty() || self.closing != CloseState::Open {
            return DocumentEffect::Stay;
        }
        if query.is_broken() {
            self.notice = Some(Notice::Find(FindNotice::BrokenPattern));
            return DocumentEffect::Stay;
        }
        if self.saving {
            self.notice = Some(Notice::StillSaving);
            return DocumentEffect::Stay;
        }
        if self.refuses_change() {
            return DocumentEffect::Stay;
        }
        self.notice = None;
        self.commit_edit();
        self.finding = None;
        let scope = self.find_scope(query.clone());
        self.find_query = Some(query);
        self.replacing = Some(ReplaceRun::new(
            scope,
            String::from(replacement),
            self.history.revision(),
        ));
        DocumentEffect::Replace
    }

    /// 全部の置換で裏で数える問いと置き換える文字（合成ルートが 1 度だけ取りに来る）。無ければ `None`。
    pub fn take_replace_pass(&mut self) -> Option<(FindScope, String)> {
        let run = self.replacing.as_mut()?;
        run.take()
            .then(|| (run.scope().clone(), String::from(run.replacement())))
    }

    /// 裏の数えの進み具合（読んだレコードの数・読む見込み）。
    pub fn replace_progress(&mut self, done: u64, total: u64) {
        if let Some(run) = self.replacing.as_mut() {
            run.set_progress(done, total);
        }
    }

    /// 裏の数えが止まった（やめた・`failed` なら読めなかった）。
    pub fn replace_stopped(&mut self, failed: bool) {
        if self.replacing.take().is_some() {
            self.notice = Some(Notice::Replace(if failed {
                ReplaceNotice::Failed
            } else {
                ReplaceNotice::Stopped
            }));
        }
    }

    /// 裏で数え終えた原本の行の書き換え `changes`（セルの書き換えと、そのセルのファイルの今の値）と、切り詰めたので飛ばした
    /// レコードの数 `skipped`。`too_many` は上限を越えて数えるのをやめた。足した行の当たりを足して、件数を見せる。
    /// 数えている間に表が変わっていれば置き換えない。
    pub fn replace_counted(
        &mut self,
        (mut changes, skipped): (Vec<(CellChange, String)>, u64),
        too_many: bool,
    ) {
        let Some(mut run) = self.replacing.take() else {
            return;
        };
        if run.revision() != self.history.revision() {
            self.notice = Some(Notice::Replace(ReplaceNotice::Changed));
            return;
        }
        changes.extend(self.added_replacements(&run));
        if too_many || u128::try_from(changes.len()).unwrap_or(u128::MAX) > MAX_PASTE_CELLS {
            self.notice = Some(Notice::Replace(ReplaceNotice::TooMany));
            return;
        }
        if changes.is_empty() {
            self.notice = Some(Notice::Replace(ReplaceNotice::NothingToReplace { skipped }));
            return;
        }
        run.set_counted(changes, skipped);
        self.replacing = Some(run);
    }

    /// 足した行の当たりを置き換える書き換え（ファイルの値は空）。
    fn added_replacements(&self, run: &ReplaceRun) -> Vec<(CellChange, String)> {
        let width = self.widths.columns().len();
        let none = |_: usize| None;
        self.added_rows()
            .into_iter()
            .flat_map(|(_, id)| {
                (0..width).filter_map(move |display| {
                    run.scope()
                        .replaced_cell((RowId::Added(id), display), &none, run.replacement())
                })
            })
            .collect()
    }

    /// 全部の置換を数えている間と、件数を見せている間の意図。数えている間の Esc はやめる。件数を見せている間は Enter で
    /// 置き換え、ほかの意図は置き換えをやめる（Esc はそれだけで終わり、ほかの意図はそのまま続ける）。
    fn replace_intent(&mut self, intent: Intent) -> Option<DocumentEffect> {
        let run = self.replacing.as_ref()?;
        if run.counted().is_none() {
            return (intent == Intent::Cancel && self.editor.is_none())
                .then_some(DocumentEffect::StopReplacing);
        }
        if intent == Intent::Confirm {
            return Some(self.apply_replace());
        }
        self.replacing = None;
        self.notice = Some(Notice::Replace(ReplaceNotice::Stopped));
        (intent == Intent::Cancel).then_some(DocumentEffect::Stay)
    }

    /// 置換の欄が開いている間の、全部の置換の確かめの意図（GitHub Issue #189・件数と確かめをパレットの中に出す）。結果と、
    /// 意図をここで使い切ったか。数えている間の Enter は何もせず、Esc はやめる。件数を見せている間の Enter は置き換え、
    /// Esc はやめる（欄は開いたまま）。ほかの意図（欄に打つ・条件を替える）は置き換えをやめ、使い切らない（欄が受け取る）。
    /// 全部の置換が無ければ `None`。
    pub fn replace_prompt_intent(&mut self, intent: Intent) -> Option<(DocumentEffect, bool)> {
        let Some(run) = self.replacing.as_ref() else {
            // 全部の置換が終わった後の、欄への次の意図。置換の知らせ（案内の行にも出す）はここまで。
            self.end_replace_note();
            return None;
        };
        let counted = run.counted().is_some();
        // 表を眺める（スクロール・ドラッグ・離す）は、数えも確かめもやめない（印を見て確かめられる）。
        if matches!(
            intent,
            Intent::Scroll(_) | Intent::Drag { .. } | Intent::Release
        ) {
            return Some((DocumentEffect::Stay, false));
        }
        let consumed = matches!(intent, Intent::Confirm | Intent::Cancel);
        if !counted {
            if intent == Intent::Confirm {
                return Some((DocumentEffect::Stay, true));
            }
            // 数えをやめたら、すぐに手放す（やめる前に数え終えた結果が後から届いても、古い問いの件数を見せない）。
            self.replacing = None;
            self.notice = Some(Notice::Replace(ReplaceNotice::Stopped));
            return Some((DocumentEffect::StopReplacing, consumed));
        }
        if intent == Intent::Confirm {
            return Some((self.apply_replace(), true));
        }
        self.replacing = None;
        self.notice = Some(Notice::Replace(ReplaceNotice::Stopped));
        Some((DocumentEffect::Stay, consumed))
    }

    /// 検索と全部の置換の結果の知らせ（状態の行と欄の案内）を消す。欄に次の意図が来たときと、パレットを開き直したとき
    /// （古い結果で案内のキーを隠さない・GitHub Issue #189。行へ飛ぶ段などに前の「見つかりません」を残さない・#291）。
    pub fn end_replace_note(&mut self) {
        if let Some(Notice::Replace(_) | Notice::Find(_)) = self.notice {
            self.notice = None;
        }
    }

    /// 置換の欄の案内に出す、全部の置換の短い 1 行（GitHub Issue #189）。数えている間は進み具合、件数を見せている間は
    /// 件数と確かめのキー、置き換えた直後は置き換えた数。無ければ `None`（いつものキーの案内）。長い知らせは状態の行に出す。
    #[must_use]
    pub fn replace_guide(&self) -> Option<String> {
        if let Some(run) = &self.replacing {
            return Some(match (run.counted(), run.progress()) {
                (Some((changes, _)), _) => format!(
                    "{} セルを置き換えます · Enter 置き換える · Esc やめる",
                    group_digits(u64::try_from(changes.len()).unwrap_or(u64::MAX))
                ),
                (None, Some((done, total))) if total > 0 && total < u64::MAX => {
                    let percent = u128::from(done).saturating_mul(100) / u128::from(total);
                    format!(
                        "置き換えるセルを数えています… {}% · Esc やめる",
                        percent.min(99)
                    )
                }
                (None, Some(_) | None) => {
                    String::from("置き換えるセルを数えています… · Esc やめる")
                }
            });
        }
        match self.notice {
            Some(Notice::Replace(ReplaceNotice::Replaced { cells, .. })) => Some(format!(
                "{} セルを置き換えました · 表で Ctrl+Z で戻す",
                group_digits(u64::try_from(cells).unwrap_or(u64::MAX))
            )),
            // 置き換えなかった訳は、欄の中でその場に言う（目を状態の行へ送らない・GitHub Issue #291）。
            Some(Notice::Replace(ReplaceNotice::NothingToReplace { .. })) => {
                Some(self.nothing_found("当たるセルがありません"))
            }
            Some(Notice::Replace(ReplaceNotice::TooMany)) => Some(String::from(
                "100 万セルを越えるので置き換えません（範囲や条件を狭めて）",
            )),
            Some(Notice::Replace(ReplaceNotice::Changed)) => Some(String::from(
                "数えている間に表が変わりました（もう一度 Alt+A）",
            )),
            Some(Notice::Replace(ReplaceNotice::Stopped)) => {
                Some(String::from("置き換えをやめました"))
            }
            Some(Notice::Replace(ReplaceNotice::Failed)) => {
                Some(String::from("ファイルを読めず、置き換えをやめました"))
            }
            Some(Notice::Find(notice)) => Some(match notice {
                FindNotice::NotFound => self.nothing_found("見つかりません"),
                FindNotice::Wrapped => String::from("表の端から続けて見つけました"),
                FindNotice::Stopped => String::from("探すのをやめました"),
                FindNotice::Failed => String::from("ファイルを読めず、探すのをやめました"),
                FindNotice::NothingToRepeat => String::from("続けて探すものがありません"),
                FindNotice::BrokenPattern => String::from("正規表現の式が読めません"),
            }),
            Some(_) | None => None,
        }
    }

    /// 全部の置換の件数を見せている間の、内訳の行（何が何に変わるか・残りの組の数・注意。注意は `true`・GitHub Issue #289）。
    /// 件数を見せていなければ空。置換の欄の条件の札の代わりに出す。
    #[must_use]
    pub fn replace_detail(&self) -> Vec<(String, bool)> {
        self.replacing
            .as_ref()
            .and_then(|run| {
                run.breakdown()
                    .map(|breakdown| breakdown.lines(run.replacement()))
            })
            .unwrap_or_default()
    }

    /// 見つからなかったときの短い知らせ `head` に、絞り込む条件のうちオンのものを添える（GitHub Issue #291）。かなの区別は
    /// 既定でオンなので添えない。
    fn nothing_found(&self, head: &str) -> String {
        let Some(options) = self.find_query.as_ref().map(FindQuery::options) else {
            return String::from(head);
        };
        let on: Vec<&str> = [
            (options.match_case(), "大小を区別"),
            (options.whole_cell(), "セル全体"),
            (options.regex(), "正規表現"),
            (options.match_width(), "全角半角を区別"),
        ]
        .into_iter()
        .filter_map(|(on, name)| on.then_some(name))
        .collect();
        if on.is_empty() {
            String::from(head)
        } else {
            format!("{head}（オン: {}）", on.join("・"))
        }
    }

    /// 数え終えた書き換えを 1 手の束として書く（Ctrl+Z 1 回で戻る・ADR 0029 の決定 3 と同じ形）。
    fn apply_replace(&mut self) -> DocumentEffect {
        let Some(mut run) = self.replacing.take() else {
            return DocumentEffect::Stay;
        };
        self.commit_edit();
        if self.refuses_change() {
            return DocumentEffect::Stay;
        }
        if self.saving {
            self.notice = Some(Notice::StillSaving);
            return DocumentEffect::Stay;
        }
        if run.revision() != self.history.revision() {
            self.notice = Some(Notice::Replace(ReplaceNotice::Changed));
            return DocumentEffect::Stay;
        }
        let Some((changes, skipped)) = run.take_counted() else {
            return DocumentEffect::Stay;
        };
        let cells = changes.len();
        self.cut = None;
        self.history
            .record_batch(Vec::new(), changes, &mut self.edits);
        self.notice = Some(Notice::Replace(ReplaceNotice::Replaced { cells, skipped }));
        DocumentEffect::Stay
    }

    /// F3 / Shift+F3：前に探した問いで続けて探す。まだ探していなければ知らせる。
    fn find_again(&mut self, forward: bool) -> DocumentEffect {
        let Some(query) = self.find_query.clone() else {
            self.notice = Some(Notice::Find(FindNotice::NothingToRepeat));
            return DocumentEffect::Stay;
        };
        self.find(query, forward)
    }

    /// 前に探した問い（検索の欄を開くときに入れておく）。
    #[must_use]
    pub const fn find_query(&self) -> Option<&FindQuery> {
        self.find_query.as_ref()
    }

    /// 裏で探す 1 回分と、探す問い・列・書き換えの写し（合成ルートが裏の仕事を始める）。無ければ `None`。
    pub fn take_find_pass(&mut self) -> Option<(FindScope, FindPass)> {
        let run = self.finding.as_mut()?;
        let pass = run.take_pending()?;
        Some((run.scope().clone(), pass))
    }

    /// 裏の検索の進み具合（読んだレコードの数・読む見込み）。
    pub fn find_progress(&mut self, done: u64, total: u64) {
        if let Some(run) = self.finding.as_mut() {
            run.set_progress(done, total);
        }
    }

    /// 裏の検索が止まった（やめた・`failed` なら読めなかった）。
    pub fn find_stopped(&mut self, failed: bool) {
        if self.finding.take().is_some() {
            self.notice = Some(Notice::Find(if failed {
                FindNotice::Failed
            } else {
                FindNotice::Stopped
            }));
        }
    }

    /// 裏の 1 回分の答え（当たったレコードと表示の列・無ければ `None`）。足した行の当たりと比べて近い方へカーソルを動かす。
    /// どちらにも無ければ、反対の端から始めたセルまでを 1 度だけ続けて探す（`DocumentEffect::Find`）。それでも無ければ知らせる。
    pub fn found(&mut self, found: Option<(u64, usize)>, window: WindowView) -> DocumentEffect {
        let Some(mut run) = self.finding.take() else {
            return DocumentEffect::Stay;
        };
        let source = found.and_then(|(record, column)| {
            self.rows
                .position_of(RowId::Source(record))
                .map(|line| (line, column))
        });
        let added = self.added_match(&run);
        let best = match (source, added) {
            (Some(left), Some(right)) => Some(if run.forward() {
                left.min(right)
            } else {
                left.max(right)
            }),
            (Some(one), None) | (None, Some(one)) => Some(one),
            (None, None) => None,
        };
        if let Some((line, column)) = best {
            if run.wrapped() {
                self.notice = Some(Notice::Find(FindNotice::Wrapped));
            }
            return self.reveal_found(line, column, window);
        }
        if run.wrapped() {
            self.notice = Some(Notice::Find(FindNotice::NotFound));
            return DocumentEffect::Stay;
        }
        run.wrap();
        run.set_pending(self.find_pass(&run));
        self.finding = Some(run);
        DocumentEffect::Find
    }

    /// 検索の今の 1 回分（向きと、反対の端から続けているか）で、原本のどのレコードをどう探すか。
    fn find_pass(&self, run: &FindRun) -> FindPass {
        let (line, column) = run.start();
        let records = self.records();
        let rest = |first: u64| records.map_or(u64::MAX, |total| total.saturating_sub(first));
        let here = match self.rows.row_at(line) {
            Some(RowId::Source(record)) => Some(record),
            Some(RowId::Added(_)) | None => None,
        };
        let after = here.or_else(|| self.nearest_source(line, true));
        let before = here.or_else(|| self.nearest_source(line, false));
        let at_cursor = here.map(|record| (record, column));
        match (run.forward(), run.wrapped()) {
            (true, false) => after.map_or(FindPass::new(0, 0, FindBound::After(None)), |first| {
                FindPass::new(first, rest(first), FindBound::After(at_cursor))
            }),
            (true, true) => FindPass::new(
                0,
                before.map_or(0, |last| last.saturating_add(1)),
                FindBound::After(None),
            ),
            (false, false) => FindPass::new(
                0,
                before.map_or(0, |last| last.saturating_add(1)),
                FindBound::Before(at_cursor),
            ),
            (false, true) => after.map_or(FindPass::new(0, 0, FindBound::Before(None)), |first| {
                FindPass::new(first, rest(first), FindBound::Before(None))
            }),
        }
    }

    /// 行の並びの位置 `line` 以降（`after`）か以前で、いちばん近い原本の行のレコード。
    fn nearest_source(&self, line: u64, after: bool) -> Option<u64> {
        let segments = self.source_segments();
        if after {
            return segments.iter().find_map(|&(start, length, first)| {
                (line < start.saturating_add(length))
                    .then(|| first.saturating_add(line.max(start).saturating_sub(start)))
            });
        }
        segments.iter().rev().find_map(|&(start, length, first)| {
            let last = start.saturating_add(length).checked_sub(1)?;
            (start <= line && length > 0)
                .then(|| first.saturating_add(line.min(last).saturating_sub(start)))
        })
    }

    /// 行の並びの区間を、最初の位置・行の数と組にした並び（原本の終わりまでの区間は索引の数か、分からなければ果てまで）。
    fn positioned_runs(&self) -> Vec<(u64, u64, &OrderRun)> {
        let records = self.records().unwrap_or(u64::MAX);
        let mut position = 0_u64;
        self.rows
            .runs()
            .iter()
            .map(|run| {
                let length = run.len(records);
                let start = position;
                position = position.saturating_add(length);
                (start, length, run)
            })
            .collect()
    }

    /// 原本の行の区間（最初の位置・行の数・最初のレコード）。
    fn source_segments(&self) -> Vec<(u64, u64, u64)> {
        self.positioned_runs()
            .into_iter()
            .filter_map(|(start, length, run)| match run {
                OrderRun::Source { first, .. } => Some((start, length, *first)),
                OrderRun::Added(_) => None,
            })
            .collect()
    }

    /// 足した行（位置と通し番号）。
    fn added_rows(&self) -> Vec<(u64, u64)> {
        self.positioned_runs()
            .into_iter()
            .flat_map(|(start, _, run)| match run {
                OrderRun::Added(ids) => ids.iter().zip(start..).map(|(&id, at)| (at, id)).collect(),
                OrderRun::Source { .. } => Vec::new(),
            })
            .collect()
    }

    /// 足した行のうち、今の 1 回分の範囲で当たる、いちばん近いセル（行の並びの位置・表示の列）。
    fn added_match(&self, run: &FindRun) -> Option<(u64, usize)> {
        let (line, column) = run.start();
        let width = self.widths.columns().len();
        let hits = self.added_rows().into_iter().filter_map(|(at, id)| {
            let columns = find_columns(run, (at, line), (column, width));
            let row = RowId::Added(id);
            let hit = if run.forward() {
                run.scope().first_in_row(row, columns, |_| None)
            } else {
                run.scope().last_in_row(row, columns, |_| None)
            };
            hit.map(|found| (at, found))
        });
        if run.forward() {
            hits.min()
        } else {
            hits.max()
        }
    }

    /// 見つけたセル（行の並びの位置 `line`・表示の列 `column`）へカーソルを動かし、見えるように表示を動かす。
    /// 表に重なった面（`cover`）の下に入るなら、面の下へ出るまで表を送る（GitHub Issue #286）。表の先頭で送れないことは
    /// [`Self::cover_hides_cursor`] で分かる。
    fn reveal_found(&mut self, line: u64, column: usize, window: WindowView) -> DocumentEffect {
        let body = body_of(line, self.header_line());
        let page = self.page_rows(window);
        let current = self.view.first_body();
        self.select(CellPosition::new(body, column), SelectionMode::Collapse);
        self.first_column = self.reveal_column(column, window);
        let (above, below) = self.covered_rows(column, window);
        let room = page.saturating_sub(below).max(1);
        self.go_to(reveal_row_below(current, body, room, above), page)
    }

    /// 表示の列 `column` の本体の行のうち、`cover` の下に入る行の数（上から・下から）。面が本体の行の上の端に掛かれば
    /// 上から、そうでなければ面の上の端より下を下から数える。列が面と横に重ならなければ `(0, 0)`。
    fn covered_rows(&self, column: usize, window: WindowView) -> (u64, u64) {
        let Some(cover) = self.cover else {
            return (0, 0);
        };
        let body = self.bars(window).body();
        let layout = self.layout(self.first_column, window.content_metrics());
        let row = layout.row_height().max(1);
        let Some(cell) = layout.cell_rect(0, column) else {
            return (0, 0);
        };
        let left = body.origin().x().saturating_add(cell.origin().x());
        let right = left.saturating_add(cell.size().width());
        let (cover_left, cover_right) = (
            cover.origin().x(),
            cover.origin().x().saturating_add(cover.size().width()),
        );
        if right <= cover_left || cover_right <= left {
            return (0, 0);
        }
        let rows_top = body
            .origin()
            .y()
            .saturating_add(row.saturating_mul(self.fixed_rows()));
        let rows_bottom = body.origin().y().saturating_add(body.size().height());
        let (cover_top, cover_bottom) = (
            cover.origin().y(),
            cover.origin().y().saturating_add(cover.size().height()),
        );
        if cover_top <= rows_top {
            (
                u64::from(cover_bottom.saturating_sub(rows_top).div_ceil(row)),
                0,
            )
        } else {
            (
                0,
                u64::from(rows_bottom.saturating_sub(cover_top).div_ceil(row)),
            )
        }
    }

    /// 表に重なった面（窓の座標）を知らせる。見つけたセルをその下に隠さない（GitHub Issue #286）。`None` で無し。
    pub fn set_cover(&mut self, cover: Option<PixelRect>) {
        self.cover = cover;
    }

    /// 表をどう送っても、カーソルのセルが面の下に残るか（面が上なら表の先頭の行、面が下なら表の末尾の行）。合成ルートが
    /// 面を反対の端へ寄せる（GitHub Issue #286）。
    #[must_use]
    pub fn cover_hides_cursor(&self, window: WindowView) -> bool {
        let cursor = self.selection.cursor();
        let page = self.page_rows(window);
        let (above, below) = self.covered_rows(cursor.column(), window);
        if cursor.body() < above.min(page.saturating_sub(1)) {
            return true;
        }
        let Some(total) = self.extent().rows() else {
            return false;
        };
        let room = page.saturating_sub(below).max(1);
        below > 0 && cursor.body() >= total.saturating_sub(page).saturating_add(room)
    }

    /// IME の確定（ADR 0026 の決定 4）。欄に挿し、確定前の文字列は消える。編集していなければ入力の状態で編集に入る。
    /// 尋ねる画面では受けない。保存中は知らせるだけ。
    pub fn commit_text(&mut self, text: &str) {
        if self.closing != CloseState::Open || text.is_empty() {
            return;
        }
        if self.saving {
            self.notice = Some(Notice::StillSaving);
            return;
        }
        self.notice = None;
        // 読み取り専用なら `begin_edit` が止めて知らせる（EDIT-13）。
        if self.editor.is_none() && !self.begin_edit(EditMode::Entering) {
            return;
        }
        if let Some(editor) = self.editor.as_mut() {
            editor.insert(text);
        }
    }

    /// 編集中のセルの、窓の中の矩形（IME の候補の窓を置く所）。編集していないか、セルが見えていなければ `None`。
    #[must_use]
    pub fn composition_area(&self, window: WindowView) -> Option<PixelRect> {
        self.editor.as_ref()?;
        let cursor = self.selection.cursor();
        let index = usize::try_from(cursor.body().checked_sub(self.view.first_body())?).ok()?;
        self.view.rows().get(index)?;
        let layout = self.layout(self.first_column, window.content_metrics());
        // 上の行は上に固定した行（見出しか列の番号）。
        let cell = layout.cell_rect(index.checked_add(self.fixed_slots())?, cursor.column())?;
        let body = self.bars(window).body();
        let origin = PixelPoint::new(
            body.origin().x().saturating_add(cell.origin().x()),
            body.origin().y().saturating_add(cell.origin().y()),
        );
        let bottom = cell.origin().y().saturating_add(cell.size().height());
        (bottom <= body.size().height()).then_some(PixelRect::new(origin, cell.size()))
    }

    /// 編集を確定して、差分に入れる（見えていた値のままなら何もしない・原本の値に戻ったら差分を消す）。
    fn commit_edit(&mut self) {
        let Some(editor) = self.editor.take() else {
            return;
        };
        match editor.commit() {
            EditCommit::Keep => {}
            // 差分に入れるか消すかは履歴が決める（ファイルの値と同じなら消す）。どちらも 1 手として積む。
            EditCommit::Restore { row, column } | EditCommit::Set { row, column, .. } => {
                let change = CellChange::new(
                    row,
                    column,
                    String::from(editor.shown()),
                    String::from(editor.field().text()),
                );
                let source = editor.source().unwrap_or_default();
                self.history.record(change, source, &mut self.edits);
            }
        }
    }

    /// Ctrl+Z / Ctrl+Y。編集中の Ctrl+Z は編集をやめるだけ。戻した・やり直したセルへカーソルを動かす。
    fn undo_or_redo(&mut self, redo: bool, window: WindowView) -> DocumentEffect {
        if !redo && self.editor.take().is_some() {
            return DocumentEffect::Stay;
        }
        self.commit_edit();
        let step = if redo {
            self.history
                .redo((&mut self.edits, &mut self.rows, &mut self.columns))
        } else {
            self.history
                .undo((&mut self.edits, &mut self.rows, &mut self.columns))
        };
        match step {
            None => {
                self.notice = Some(if redo {
                    Notice::NothingToRedo
                } else {
                    Notice::NothingToUndo
                });
                DocumentEffect::Stay
            }
            Some(EditStep::Cells(changes)) => {
                let Some(first) = changes.first() else {
                    return DocumentEffect::Stay;
                };
                let effect = self.reveal_cell(first.row(), first.column(), window);
                self.notice = Some(if redo {
                    Notice::Redone(changes.len())
                } else {
                    Notice::Undone(changes.len())
                });
                effect
            }
            Some(EditStep::Columns(change)) => {
                self.column_step_taken(&change, redo);
                self.notice = Some(Notice::Columns(if redo {
                    ColumnsNotice::Redone(change.count())
                } else {
                    ColumnsNotice::Undone(change.count())
                }));
                self.after_columns_changed(change.at(), window)
            }
            Some(EditStep::Batch(steps)) => self.batch_undone(&steps, redo, window),
            Some(EditStep::Rows(change)) => {
                let body = body_of(change.at(), self.header_line());
                self.notice = Some(if redo {
                    Notice::Rows(RowsNotice::Redone(change.count()))
                } else {
                    Notice::Rows(RowsNotice::Undone(change.count()))
                });
                self.after_rows_changed(body, window)
            }
        }
    }

    /// 束の 1 手（貼り付け・ADR 0029 の決定 3）を戻した・やり直した後。列の幅を足し消しし、書き換えたセルのうち並びに残る
    /// 左上のセルへカーソルを動かす（残っていなければ、足し消しした行の位置へ）。
    fn batch_undone(
        &mut self,
        steps: &[EditStep],
        redo: bool,
        window: WindowView,
    ) -> DocumentEffect {
        let flat = flattened(steps);
        let cells = flat.iter().map(|step| step.cell_changes().len()).sum();
        // 貼った範囲の左上（並びに残っているセルのうち、いちばん上でいちばん左）へ動く。
        let first_cell = flat
            .iter()
            .flat_map(|step| step.cell_changes())
            .filter_map(|change| {
                let line = self.rows.position_of(change.row())?;
                let column = self.columns.position_of(change.column())?;
                Some(((line, column), (change.row(), change.column())))
            })
            .min_by_key(|&(place, _)| place)
            .map(|(_, cell)| cell);
        let columns: Vec<&ColumnChange> = flat
            .iter()
            .filter_map(|step| step.column_change())
            .collect();
        for change in &columns {
            self.column_step_taken(change, redo);
        }
        self.notice = Some(if redo {
            Notice::Redone(cells)
        } else {
            Notice::Undone(cells)
        });
        let mut effect = DocumentEffect::Stay;
        if let Some(change) = columns.first() {
            effect = self.after_columns_changed(change.at(), window);
        }
        // 左上のセルが並びに残っていれば、そこへ（動かなくてよければ `Stay`・GitHub Issue #153）。
        // 列を足し消ししたなら、動かなくても見えている行を読み直す。
        if let Some((row, column)) = first_cell {
            let revealed = self.reveal_cell(row, column, window);
            return if revealed == DocumentEffect::Stay {
                effect
            } else {
                revealed
            };
        }
        if let Some(change) = flat.iter().find_map(|step| step.row_change()) {
            return self.after_rows_changed(body_of(change.at(), self.header_line()), window);
        }
        effect
    }

    /// 列の足し消しの手 `change` を戻した（`redo` ならやり直した）。戻すと足した列が消え、消した列が戻る。幅を合わせる。
    fn column_step_taken(&mut self, change: &ColumnChange, redo: bool) {
        if change.adds() == redo {
            self.columns_inserted(change.at(), change.widths());
        } else {
            let _widths = self.columns_removed(change.at(), change.count());
        }
    }

    /// Ctrl++ と Ctrl+- が列に効くか（列全体を選んでいて、行全体ではない・ADR 0027 の決定 2）。すべてを選んでいれば行。
    fn columns_chosen(&self) -> bool {
        let (top, bottom) = (self.selection.top_left(), self.selection.bottom_right());
        let last_column = self.widths.columns().len().saturating_sub(1);
        let whole_columns = top.body() == 0 && bottom.body() == TO_END;
        let whole_rows = top.column() == 0 && bottom.column() == last_column;
        whole_columns && !whole_rows
    }

    /// 足し消しする表の列（最初・最後・数）。列全体を選んでいれば選んだ列、そうでなければカーソルの列。
    /// 表示の上で入れ替えた列が離れていれば `None`。
    fn chosen_columns(&self) -> Option<(usize, usize, usize)> {
        let (top, bottom) = (self.selection.top_left(), self.selection.bottom_right());
        let whole = top.body() == 0 && bottom.body() == TO_END && self.others.is_empty();
        let cursor = self.selection.cursor().column();
        let (from, to) = if whole {
            (top.column(), bottom.column())
        } else {
            (cursor, cursor)
        };
        let mapped: Vec<usize> = (from..=to)
            .map(|column| self.source_column(column))
            .collect();
        let first = mapped.iter().copied().min()?;
        let last = mapped.iter().copied().max()?;
        (last.saturating_sub(first).saturating_add(1) == mapped.len()).then_some((
            first,
            last,
            mapped.len(),
        ))
    }

    /// 空の列を足す（ADR 0027 の決定 2）。足した最初の列へカーソルを動かす。
    fn insert_columns(&mut self, place: ColumnPlace, window: WindowView) -> DocumentEffect {
        let Some((first, last, count)) = self.chosen_columns() else {
            self.notice = Some(Notice::Columns(ColumnsNotice::Reordered));
            return DocumentEffect::Stay;
        };
        let at = match place {
            ColumnPlace::Left => first,
            ColumnPlace::Right => last.saturating_add(1),
        };
        let change = self.add_columns(at, count);
        self.history.record_columns(change);
        self.notice = Some(Notice::Columns(ColumnsNotice::Inserted(count)));
        self.after_columns_changed(at, window)
    }

    /// 選んだ範囲にかかる列を消す（尋ねない・ADR 0027 の決定 4）。すべての列は消さない。
    fn remove_columns(&mut self, window: WindowView) -> DocumentEffect {
        let Some((first, _, count)) = self.chosen_columns() else {
            self.notice = Some(Notice::Columns(ColumnsNotice::Reordered));
            return DocumentEffect::Stay;
        };
        if count >= self.widths.columns().len() {
            self.notice = Some(Notice::Columns(ColumnsNotice::KeepOne));
            return DocumentEffect::Stay;
        }
        let runs = self.columns.remove(first, count);
        let widths = self.columns_removed(first, count);
        self.history.record_columns(ColumnChange::Removed {
            at: first,
            runs,
            widths,
        });
        self.notice = Some(Notice::Columns(ColumnsNotice::Removed(count)));
        self.after_columns_changed(first, window)
    }

    /// 表の `at` 列目の前に、幅 `widths` の列が入った。幅・利用者の幅・固定した列の数をずらす。
    fn columns_inserted(&mut self, at: usize, widths: &[u32]) {
        let count = widths.len();
        self.widths.insert(at, widths);
        self.custom = core::mem::take(&mut self.custom)
            .into_iter()
            .map(|(column, chars)| {
                let moved = if column >= at {
                    column.saturating_add(count)
                } else {
                    column
                };
                (moved, chars)
            })
            .collect();
        if self.frozen > at {
            self.frozen = self.frozen.saturating_add(count);
        }
    }

    /// 表の `at` 列目から `count` 列が抜けた。外した幅を返し、利用者の幅・固定した列の数をずらす。
    fn columns_removed(&mut self, at: usize, count: usize) -> Vec<u32> {
        let removed = self.widths.remove(at, count);
        let end = at.saturating_add(count);
        self.custom = core::mem::take(&mut self.custom)
            .into_iter()
            .filter(|(column, _)| !(at..end).contains(column))
            .map(|(column, chars)| {
                let moved = if column >= end {
                    column.saturating_sub(count)
                } else {
                    column
                };
                (moved, chars)
            })
            .collect();
        if self.frozen > at {
            self.frozen = self
                .frozen
                .saturating_sub(count.min(self.frozen.saturating_sub(at)));
        }
        removed
    }

    /// 列を足し消しした（戻した）後。表示だけの列の入れ替えは元に戻し、見出しを並べ直し、カーソルを表の `column` 列目
    /// （最後の列で止める）に置いて、見えている行を読み直させる。
    fn after_columns_changed(&mut self, column: usize, window: WindowView) -> DocumentEffect {
        self.order.clear();
        self.counted = None;
        self.header = self
            .header
            .iter()
            .zip(&self.header_source)
            .map(|(header, cells)| {
                let cells = self.arranged(&CellRow::new(cells));
                ViewRow::new(header.line(), &cells).with_row(header.row())
            })
            .collect();
        let column = column.min(self.widths.columns().len().saturating_sub(1));
        let body = self.selection.cursor().body();
        self.select(CellPosition::new(body, column), SelectionMode::Collapse);
        self.first_column = self.reveal_column(column, window);
        DocumentEffect::Read {
            first_body: self.view.first_body(),
        }
    }

    /// 選んだ行の数（行全体を選んでいれば選んだ行の数、そうでなければ 1）と、範囲の最初と最後の行（本体の番目）。
    fn chosen_rows(&self) -> (u64, u64, u64) {
        let (top, bottom) = self.row_span();
        let last_column = self.widths.columns().len().saturating_sub(1);
        let whole = self.selection.top_left().column() == 0
            && self.selection.bottom_right().column() == last_column
            && self.others.is_empty();
        let count = if whole {
            bottom.saturating_sub(top).saturating_add(1)
        } else {
            1
        };
        (top, bottom, count)
    }

    /// 空の行を足す（ADR 0027 の決定 2）。足した最初の行へカーソルを動かす。
    fn insert_rows(&mut self, place: RowPlace, window: WindowView) -> DocumentEffect {
        let (top, bottom, count) = self.chosen_rows();
        let (body, count) = match place {
            RowPlace::Above => (top, count),
            RowPlace::Below => (bottom.saturating_add(1), count),
            RowPlace::End => {
                let Some(records) = self.records() else {
                    self.notice = Some(Notice::Rows(RowsNotice::AwaitIndex));
                    return DocumentEffect::Stay;
                };
                (self.body_lines(records), 1)
            }
        };
        let (_added, change) = self.add_rows(self.line_of(body), count);
        self.history.record_rows(change);
        self.notice = Some(Notice::Rows(RowsNotice::Inserted(count)));
        self.after_rows_changed(body, window)
    }

    /// 選んだ範囲にかかる行を消す（尋ねない・ADR 0027 の決定 4）。見出しの行は消さない（前書きと本体にまたがる
    /// 範囲は、見出しの前と後ろを別の手として消す）。
    fn remove_rows(&mut self, window: WindowView) -> DocumentEffect {
        let to_end = self
            .ranges()
            .all()
            .any(|range| range.bottom_right().body() == TO_END);
        if to_end && self.records().is_none() {
            self.notice = Some(Notice::Rows(RowsNotice::AwaitIndex));
            return DocumentEffect::Stay;
        }
        let (top, bottom, _) = self.chosen_rows();
        let (first, last) = (self.line_of(top), self.line_of(bottom));
        let spans = match self.header_line() {
            Some(header) if first < header.first() && header.end() <= last => {
                vec![
                    (header.end(), last),
                    (first, header.first().saturating_sub(1)),
                ]
            }
            Some(_) | None => vec![(first, last)],
        };
        let mut removed = 0_u64;
        for (from, to) in spans {
            let count = to.saturating_sub(from).saturating_add(1);
            let runs = self.rows.remove(from, count);
            let change = RowChange::Removed { at: from, runs };
            removed = removed.saturating_add(change.count());
            self.history.record_rows(change);
        }
        self.notice = Some(Notice::Rows(RowsNotice::Removed(removed)));
        self.after_rows_changed(top, window)
    }

    /// 行を足し消しした（戻した）後。カーソルを本体の `body` 番目（最後の行で止める）に置き、見えるように読み直させる。
    fn after_rows_changed(&mut self, body: u64, window: WindowView) -> DocumentEffect {
        let body = body.min(self.last_body());
        let column = self.selection.cursor().column();
        self.select(CellPosition::new(body, column), SelectionMode::Collapse);
        let page = self.page_rows(window);
        let current = self.view.first_body();
        let first = clamp_first(
            current,
            reveal_row(current, body, page),
            page,
            self.extent().rows(),
        );
        DocumentEffect::Read { first_body: first }
    }

    /// セル（`row`・`column`）へカーソルを動かし、見えるように表示を動かす。並びに無い行・足した列なら動かない。
    fn reveal_cell(&mut self, row: RowId, column: ColumnId, window: WindowView) -> DocumentEffect {
        let (Some(line), Some(source)) =
            (self.rows.position_of(row), self.columns.position_of(column))
        else {
            return DocumentEffect::Stay;
        };
        let body = body_of(line, self.header_line());
        let display = self
            .order
            .iter()
            .position(|&shown| shown == source)
            .unwrap_or(source);
        let page = self.page_rows(window);
        let current = self.view.first_body();
        self.select(CellPosition::new(body, display), SelectionMode::Collapse);
        self.first_column = self.reveal_column(display, window);
        self.go_to(reveal_row(current, body, page), page)
    }

    /// 本体の `body` 番目の行（見えていなければ `None`）。
    fn view_row(&self, body: u64) -> Option<&ViewRow> {
        body.checked_sub(self.view.first_body())
            .and_then(|index| usize::try_from(index).ok())
            .and_then(|index| self.view.rows().get(index))
    }

    /// 編集中のセルの描き方（表示の字数に直す）。
    fn editing_cell(&self) -> Option<EditingCell> {
        let editor = self.editor.as_ref()?;
        let (text, caret) = editor.field().display();
        let columns = |index: usize| {
            let head: String = text.chars().take(index).collect();
            display_columns(&single_line(&head))
        };
        let underline = caret
            .underline()
            .map(|(start, end)| (columns(start), columns(end)));
        Some(EditingCell::new(
            single_line(&text),
            columns(caret.cursor()),
            underline,
        ))
    }

    /// カーソルを `step` で動かし（`mode` で畳む・広げる）、見えるように横を動かして、縦の表示の先頭を返す（1 画面 `page` 行）。
    fn move_cursor(
        &mut self,
        (step, mode): (CursorStep, SelectionMode),
        page: u64,
        window: WindowView,
    ) -> u64 {
        let current = self.view.first_body();
        let to = step_cursor(self.selection.cursor(), step, page, self.extent());
        self.select(to, mode);
        self.first_column = self.reveal_column(to.column(), window);
        reveal_row(current, to.body(), page)
    }

    /// ホイール。横はその場で動かし、縦は動いた先の本体の先頭を返す。ドラッグ中なら、動いた後の指の下まで選択を広げる。
    fn wheel(&mut self, scroll: Scroll, current: u64, window: WindowView) -> u64 {
        let target = self.scroll(scroll, current);
        self.follow_wheel(target, self.page_rows(window), window);
        target
    }

    /// 縦の表示を動かさない、すべて・行全体・列全体の選択と、列の並びの意図（Issue #62）。ほかの意図は何もしない。
    /// 縦の表示の先頭（動かない）を返す。
    fn arrange(&mut self, intent: Intent, window: WindowView) -> u64 {
        match intent {
            Intent::SelectAll => self.select_line(LineHit::All, SelectionMode::Collapse),
            Intent::SelectRows => self.select_whole_rows(),
            Intent::SelectColumns => self.select_whole_columns(),
            Intent::MoveColumns(shift) => self.shift_columns(shift, window),
            Intent::ResetColumns => {
                self.order.clear();
                self.counted = None;
            }
            Intent::ResetWidths => self.custom.clear(),
            Intent::FreezeColumns => {
                self.freeze_columns(self.selection.cursor().column().saturating_add(1), window);
            }
            Intent::UnfreezeColumns => self.freeze_columns(0, window),
            Intent::Scroll(_)
            | Intent::Move { .. }
            | Intent::Point { .. }
            | Intent::DoubleClick { .. }
            | Intent::Drag { .. }
            | Intent::Release
            | Intent::GoTo(_)
            | Intent::OpenGoTo
            | Intent::Copy
            | Intent::Cut
            | Intent::OpenFind { .. }
            | Intent::FindAgain { .. }
            | Intent::FindToggle(_)
            | Intent::ReplaceAll
            | Intent::ToggleDetail
            | Intent::ToggleReadOnly
            | Intent::Theme(_)
            | Intent::Zoom(_)
            | Intent::OpenPalette
            | Intent::Type(_)
            | Intent::Erase
            | Intent::Confirm
            | Intent::Cancel
            | Intent::Edit
            | Intent::Delete
            | Intent::BreakLine
            | Intent::FillSelection
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
            | Intent::RemoveHere => {}
        }
        self.view.first_body()
    }

    /// セルの上の `at` を押した。そのセルを選び（`mode` で畳む・広げる・足す）、離すまでドラッグを数える。
    /// 行番号・見出し・左上の角なら、行全体・列全体・すべてを選ぶ（Issue #47）。
    fn press_cell(&mut self, at: PixelPoint, mode: SelectionMode, window: WindowView) {
        let before = CellRestore::new(
            (self.selection, self.others.len()),
            self.view.first_body(),
            self.first_column,
        );
        if let Some(display) = self.column_grip(at, window) {
            self.start_resize(at, display);
            return;
        }
        let Some(to) = self.cell_at(at, window) else {
            if let Some(kind) = self.press_line(at, mode, window) {
                self.start_hold(at, kind, before);
            }
            return;
        };
        self.select(to, mode);
        self.start_hold(at, PressKind::Cells, before);
    }

    /// 行番号・見出し・左上の角の `at` を押した。行全体・列全体・すべてを選び、ドラッグで広げるもの（行か列）を返す。
    fn press_line(
        &mut self,
        at: PixelPoint,
        mode: SelectionMode,
        window: WindowView,
    ) -> Option<PressKind> {
        let hit = self.line_at(at, window)?;
        if let LineHit::Column(column) = hit
            && mode == SelectionMode::Collapse
            && self.whole_columns_selected(column)
        {
            // 選んでいる列の見出しを押した。動かせば移し、動かさずに離せばその列だけを選ぶ（Issue #49）。
            return Some(PressKind::MoveColumns);
        }
        self.select_line(hit, mode);
        match hit {
            LineHit::Row(_) => Some(PressKind::Rows),
            LineHit::Column(_) => Some(PressKind::Columns),
            LineHit::All => None,
        }
    }

    /// 見出しの行の、列の右の境目（左右 `COLUMN_GRIP` 画素）の上なら、その表示の列（Issue #64）。
    fn column_grip(&self, at: PixelPoint, window: WindowView) -> Option<usize> {
        let bars = self.bars(window);
        let (body, fixed) = (bars.body(), bars.fixed());
        let x = at.x().checked_sub(body.origin().x())?;
        let y = at.y().checked_sub(body.origin().y())?;
        if y >= fixed.header() || x < fixed.gutter() || x >= body.size().width() {
            return None;
        }
        let grip = window.metrics().px(COLUMN_GRIP);
        let layout = self.layout(self.first_column, window.content_metrics());
        layout
            .visible_columns()
            .filter_map(|column| layout.column_right(column).map(|right| (column, right)))
            .find(|&(_, right)| right.abs_diff(x) <= grip)
            .map(|(column, _)| column)
    }

    /// 表示の列 `display` の境目を `at` で押した。離すまで、その元の列の幅を変える。
    fn start_resize(&mut self, at: PixelPoint, display: usize) {
        let column = self.source_column(display);
        let start = self
            .widths
            .resized(&self.custom)
            .columns()
            .get(column)
            .copied()
            .unwrap_or(MIN_COLUMN_COLUMNS);
        self.hold = Some(PointerHold::ColumnWidth {
            column,
            press: pointer_of(at),
            start,
            original: self.custom.get(&column).copied(),
        });
    }

    /// 列の幅を変えるドラッグ `hold` で、指を `at` へ動かした。押した所からの横の距離を半角の字数に丸めて足す。
    fn resize_to(&mut self, hold: PointerHold, at: PointerPoint, window: WindowView) {
        let PointerHold::ColumnWidth {
            column,
            press,
            start,
            ..
        } = hold
        else {
            return;
        };
        let half = i64::from(window.content_metrics().half_width().max(1));
        let dx = i64::from(at.x()) - i64::from(press.x());
        let chars = (i64::from(start) + (dx + half / 2).div_euclid(half)).clamp(
            i64::from(MIN_COLUMN_COLUMNS),
            i64::from(MAX_RESIZED_COLUMNS),
        );
        self.custom
            .insert(column, u32::try_from(chars).unwrap_or(MIN_COLUMN_COLUMNS));
    }

    /// 中身の上で出すカーソルの形。列の境目の上か、列の幅を変えている間は左右の矢印（Issue #64）。
    #[must_use]
    pub fn pointer_shape(&self) -> PointerShape {
        match self.hold {
            Some(PointerHold::ColumnWidth { .. }) => PointerShape::ColumnResize,
            Some(
                PointerHold::Thumb { .. } | PointerHold::Track { .. } | PointerHold::Cells { .. },
            )
            | None => {
                if self.grip {
                    PointerShape::ColumnResize
                } else {
                    PointerShape::Arrow
                }
            }
        }
    }

    /// 押した所 `at` から、離すまで `kind` を広げるドラッグを数える。
    fn start_hold(&mut self, at: PixelPoint, kind: PressKind, before: CellRestore) {
        let press = pointer_of(at);
        self.hold = Some(PointerHold::Cells {
            kind,
            press,
            at: press,
            dragging: false,
            before,
            ticks: 0,
        });
    }

    /// カーソルを `to` へ動かす。畳むなら前に足した範囲も消し、広げるなら今の範囲だけを広げ、足すなら今の範囲を残して
    /// `to` から新しい範囲を始める（Ctrl+クリック・Issue #46）。
    fn select(&mut self, to: CellPosition, mode: SelectionMode) {
        match mode {
            SelectionMode::Collapse => self.others.clear(),
            SelectionMode::Extend => {}
            SelectionMode::Add => self.others.push(self.selection),
        }
        self.selection = self.selection.moved(to, mode);
    }

    /// 選んだ範囲の並び（今の範囲と前に足した範囲）。
    #[must_use]
    pub fn ranges(&self) -> Ranges<'_> {
        Ranges::new(self.selection, &self.others)
    }

    /// 縦の表示の先頭を本体の `target` 番目へ（1 画面 `page` 行・末尾は行の数が分かってから）。変わるならそこから読ませる。
    fn go_to(&self, target: u64, page: u64) -> DocumentEffect {
        let current = self.view.first_body();
        let first = clamp_first(current, target, page, self.extent().rows());
        if first == current {
            DocumentEffect::Stay
        } else {
            DocumentEffect::Read { first_body: first }
        }
    }

    /// スクロールバーの上で押した・押したまま動かした・離した。スクロールバーの外を押したなら `None`（セルを選ぶ）。
    fn pointer_intent(&mut self, intent: Intent, window: WindowView) -> Option<DocumentEffect> {
        match intent {
            Intent::Point { at, mode } => self.press_bar(at, mode, window),
            Intent::DoubleClick { at } => self.press_bar(at, SelectionMode::Collapse, window),
            Intent::Drag { at } => Some(self.drag(at, window)),
            Intent::Release => {
                self.release(window);
                Some(DocumentEffect::Stay)
            }
            Intent::Scroll(_)
            | Intent::Move { .. }
            | Intent::Copy
            | Intent::Cut
            | Intent::OpenFind { .. }
            | Intent::FindAgain { .. }
            | Intent::FindToggle(_)
            | Intent::ReplaceAll
            | Intent::ToggleDetail
            | Intent::ToggleReadOnly
            | Intent::Theme(_)
            | Intent::Zoom(_)
            | Intent::OpenPalette
            | Intent::Type(_)
            | Intent::Erase
            | Intent::Confirm
            | Intent::Cancel
            | Intent::SelectAll
            | Intent::SelectRows
            | Intent::SelectColumns
            | Intent::MoveColumns(_)
            | Intent::ResetColumns
            | Intent::ResetWidths
            | Intent::FreezeColumns
            | Intent::UnfreezeColumns
            | Intent::GoTo(_)
            | Intent::OpenGoTo
            | Intent::Edit
            | Intent::Delete
            | Intent::BreakLine
            | Intent::FillSelection
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
            | Intent::RemoveHere => None,
        }
    }

    /// スクロールバーの上の `at` を押した。つまみならドラッグを始め、溝なら 1 画面ぶん動かして押し続けを数え始める。
    /// Shift（`Extend`）で溝を押したら、その位置へつまみを跳ばしてドラッグを始める（Windows の慣習）。
    fn press_bar(
        &mut self,
        at: PixelPoint,
        mode: SelectionMode,
        window: WindowView,
    ) -> Option<DocumentEffect> {
        let (scrollbar, hit) = self.bars(window).hit(at)?;
        let orientation = scrollbar.orientation();
        let along = scrollbar.along(pointer_of(at));
        let grab = match (hit, mode) {
            (
                BarHit::Thumb { grab },
                SelectionMode::Collapse | SelectionMode::Extend | SelectionMode::Add,
            ) => grab,
            (BarHit::Track(_), SelectionMode::Extend) => i64::from(scrollbar.thumb().1 / 2),
            (BarHit::Track(side), SelectionMode::Collapse | SelectionMode::Add) => {
                self.hold = Some(PointerHold::Track {
                    orientation,
                    side,
                    along,
                    ticks: 0,
                });
                return Some(self.page_toward(orientation, side, window));
            }
        };
        self.hold = Some(PointerHold::Thumb {
            orientation,
            grab,
            axis: scrollbar.axis(),
        });
        Some(self.drag_thumb(scrollbar, along.saturating_sub(grab), window))
    }

    /// 押したまま指を `at` へ動かした。つまみなら表示を動かし、溝なら指の位置を覚える（押し続けの止めどころ）。
    fn drag(&mut self, at: PointerPoint, window: WindowView) -> DocumentEffect {
        let Some(hold) = self.hold else {
            return DocumentEffect::Stay;
        };
        if let PointerHold::Cells { .. } = hold {
            self.drag_cells(hold, at, window);
            return DocumentEffect::Stay;
        }
        if let PointerHold::ColumnWidth { .. } = hold {
            self.resize_to(hold, at, window);
            return DocumentEffect::Stay;
        }
        let Some(scrollbar) = hold
            .orientation()
            .and_then(|orientation| self.bars(window).bar(orientation))
        else {
            return DocumentEffect::Stay;
        };
        match hold {
            PointerHold::Thumb { grab, axis, .. } => self.drag_thumb(
                scrollbar.with_axis(axis),
                scrollbar.along(at).saturating_sub(grab),
                window,
            ),
            PointerHold::Track {
                orientation,
                side,
                ticks,
                ..
            } => {
                self.hold = Some(PointerHold::Track {
                    orientation,
                    side,
                    along: scrollbar.along(at),
                    ticks,
                });
                DocumentEffect::Stay
            }
            PointerHold::Cells { .. } | PointerHold::ColumnWidth { .. } => DocumentEffect::Stay,
        }
    }

    /// セルの上で押したまま `at` へ動かした。押した所から `DRAG_THRESHOLD` を超えたらドラッグとし、指の下のセルまで広げる。
    fn drag_cells(&mut self, hold: PointerHold, at: PointerPoint, window: WindowView) {
        let PointerHold::Cells {
            kind,
            press,
            dragging,
            before,
            ticks,
            ..
        } = hold
        else {
            return;
        };
        let dragging = dragging || is_far(press, at, window.metrics());
        self.hold = Some(PointerHold::Cells {
            kind,
            press,
            at,
            dragging,
            before,
            ticks,
        });
        if dragging {
            self.extend_to(at, self.view.first_body(), window);
        }
    }

    /// 縦の表示の先頭が本体の `first` 番目のときの指 `at` の下のセルまで、選択を広げる。表の外なら端のセル。
    fn extend_to(&mut self, at: PointerPoint, first: u64, window: WindowView) {
        let cell = self.cell_toward(at, first, window);
        let kind = match self.hold {
            Some(PointerHold::Cells { kind, .. }) => kind,
            Some(
                PointerHold::Thumb { .. }
                | PointerHold::Track { .. }
                | PointerHold::ColumnWidth { .. },
            )
            | None => PressKind::Cells,
        };
        // 行全体は行だけ、列全体は列だけを広げる（起点はファイルの終わり・最後の列のまま）。
        let to = match kind {
            PressKind::Cells => cell,
            PressKind::Rows => CellPosition::new(cell.body(), 0),
            PressKind::Columns => CellPosition::new(0, cell.column()),
            // 移している間は選択を変えない（落とす位置は離したときに決める）。
            PressKind::MoveColumns => return,
        };
        self.selection = self.selection.moved(to, SelectionMode::Extend);
    }

    /// 行番号・見出し・左上の角の `hit` を選ぶ。列全体はファイルの終わりまで（`TO_END`）、カーソルは列の最初の行・行の最初の
    /// 列・左上。Shift なら今の起点の行（列）から広げ、Ctrl なら足す。
    fn select_line(&mut self, hit: LineHit, mode: SelectionMode) {
        let last_column = self.widths.columns().len().saturating_sub(1);
        let anchor = self.selection.anchor();
        let extend = mode == SelectionMode::Extend;
        let (from, to) = match hit {
            LineHit::Column(column) => {
                let start = if extend { anchor.column() } else { column };
                (
                    CellPosition::new(TO_END, start),
                    CellPosition::new(0, column),
                )
            }
            LineHit::Row(body) => {
                let start = if extend { anchor.body() } else { body };
                (
                    CellPosition::new(start, last_column),
                    CellPosition::new(body, 0),
                )
            }
            LineHit::All => (
                CellPosition::new(TO_END, last_column),
                CellPosition::new(0, 0),
            ),
        };
        match mode {
            SelectionMode::Collapse => self.others.clear(),
            SelectionMode::Extend => {}
            SelectionMode::Add => self.others.push(self.selection),
        }
        self.selection = Selection::at(from).moved(to, SelectionMode::Extend);
    }

    /// 今の選択が掛かる行の全体を選ぶ。起点とカーソルの行はそのままで、カーソルは行の最初の列。ほかの範囲は外す（Issue #62）。
    fn select_whole_rows(&mut self) {
        let last_column = self.widths.columns().len().saturating_sub(1);
        let (anchor, cursor) = (self.selection.anchor(), self.selection.cursor());
        self.others.clear();
        self.selection = Selection::at(CellPosition::new(anchor.body(), last_column))
            .moved(CellPosition::new(cursor.body(), 0), SelectionMode::Extend);
    }

    /// 今の選択が掛かる列の全体を、ファイルの終わりまで選ぶ。カーソルは列の最初の行。ほかの範囲は外す（Issue #62）。
    fn select_whole_columns(&mut self) {
        let (anchor, cursor) = (self.selection.anchor(), self.selection.cursor());
        self.others.clear();
        self.selection = Selection::at(CellPosition::new(TO_END, anchor.column()))
            .moved(CellPosition::new(0, cursor.column()), SelectionMode::Extend);
    }

    /// 今の選択が掛かる列を、表示の上で `shift` の向きへ 1 つ移し、列全体で選び直す（`move_columns` の経路・Issue #62）。
    /// 端なら動かさない。
    fn shift_columns(&mut self, shift: ColumnShift, window: WindowView) {
        let (start, end) = (
            self.selection.top_left().column(),
            self.selection.bottom_right().column(),
        );
        let before = match shift {
            ColumnShift::Left => start.checked_sub(1),
            ColumnShift::Right => {
                let next = end.saturating_add(1);
                (next < self.widths.columns().len()).then(|| next.saturating_add(1))
            }
        };
        let Some(before) = before else {
            return;
        };
        self.move_columns(start..=end, before);
        self.first_column = self.reveal_column(self.selection.cursor().column(), window);
    }

    /// 飛ぶ先 `target` のセルを選び、見えるように表示を動かした先の縦の表示の先頭を返す（1 画面 `page` 行・Issue #54）。
    /// レコードの番号は本体の行に写す（見出しのレコードならその次の行）。行の数より先なら最後の行。列が無ければ今の列。
    fn jump(&mut self, target: GoToTarget, page: u64, window: WindowView) -> u64 {
        let line = target.record();
        let body = body_of(line, self.header_line()).min(self.last_body());
        let last_column = self.widths.columns().len().saturating_sub(1);
        let column = target
            .column()
            .map_or(self.selection.cursor().column(), |source| {
                self.display_column(source)
            })
            .min(last_column);
        self.others.clear();
        self.selection = Selection::at(CellPosition::new(body, column));
        self.first_column = self.reveal_column(column, window);
        reveal_row(self.view.first_body(), body, page)
    }

    /// 元の列 `source` の、今の表示の位置。
    fn display_column(&self, source: usize) -> usize {
        self.order
            .iter()
            .position(|&column| column == source)
            .unwrap_or(source)
    }

    /// 表示の位置 `display` の列の、元の列の番号。
    fn source_column(&self, display: usize) -> usize {
        self.order.get(display).copied().unwrap_or(display)
    }

    /// 表示の列の範囲の並び `display` を、元の列の範囲の並びにする（表示の順のまま・続く列はまとめる）。
    fn source_columns(&self, display: &[RangeInclusive<usize>]) -> Vec<RangeInclusive<usize>> {
        let mut out: Vec<RangeInclusive<usize>> = Vec::new();
        for source in display
            .iter()
            .cloned()
            .flatten()
            .map(|column| self.source_column(column))
        {
            match out.last_mut() {
                Some(last) if last.end().checked_add(1) == Some(source) => {
                    *last = *last.start()..=source;
                }
                Some(_) | None => out.push(source..=source),
            }
        }
        out
    }

    /// 今の範囲が列全体で、`column` 列目を含むか。
    fn whole_columns_selected(&self, column: usize) -> bool {
        let (top, bottom) = (self.selection.top_left(), self.selection.bottom_right());
        top.body() == 0
            && bottom.body() == TO_END
            && (top.column()..=bottom.column()).contains(&column)
    }

    /// 左のボタンを離した。選んだ列を移していれば落とす位置へ移し、動かさずに離していればその列だけを選ぶ。
    fn release(&mut self, window: WindowView) {
        let hold = self.hold.take();
        let Some(PointerHold::Cells {
            kind: PressKind::MoveColumns,
            press,
            at,
            dragging,
            ..
        }) = hold
        else {
            return;
        };
        if dragging {
            let (top, bottom) = (self.selection.top_left(), self.selection.bottom_right());
            let before = self.drop_index(at, window);
            self.move_columns(top.column()..=bottom.column(), before);
            return;
        }
        let pressed = PixelPoint::new(
            u32::try_from(press.x()).unwrap_or(0),
            u32::try_from(press.y()).unwrap_or(0),
        );
        if let Some(hit) = self.line_at(pressed, window) {
            self.select_line(hit, SelectionMode::Collapse);
        }
    }

    /// 指 `at` の所で離したときに列を入れる、表示の位置（その位置の列の前）。列の左半分なら前、右半分なら後ろ。
    fn drop_index(&self, at: PointerPoint, window: WindowView) -> usize {
        let body = self.bars(window).body();
        let layout = self.layout(self.first_column, window.content_metrics());
        let x = i64::from(at.x()) - i64::from(body.origin().x());
        for column in layout.visible_columns() {
            let (Some(left), Some(right)) =
                (layout.column_left(column), layout.column_right(column))
            else {
                break;
            };
            let middle = i64::from(left.midpoint(right));
            if x < middle {
                return column;
            }
        }
        self.widths.columns().len()
    }

    /// 表示の列 `block` を、表示の位置 `before` の前へ移す（表示だけ・原本は変えない）。移した列を列全体で選び直す。
    fn move_columns(&mut self, block: RangeInclusive<usize>, before: usize) {
        let count = self.widths.columns().len();
        if self.order.len() < count {
            let known = self.order.len();
            self.order.extend(known..count);
        }
        let (start, end) = (*block.start(), *block.end());
        if (start..=end.saturating_add(1)).contains(&before) || end >= count {
            return;
        }
        let moved: Vec<usize> = self.order.drain(start..=end).collect();
        let length = moved.len();
        let at = if before > end {
            before - length
        } else {
            before
        };
        for (offset, column) in moved.into_iter().enumerate() {
            self.order.insert(at.saturating_add(offset), column);
        }
        self.others.clear();
        let last = at.saturating_add(length).saturating_sub(1);
        self.selection = Selection::at(CellPosition::new(TO_END, last))
            .moved(CellPosition::new(0, at), SelectionMode::Extend);
        self.counted = None;
    }

    /// 列を移している間の、落とす位置の線（表の所の左上が原点・`bars` は表の所の置き場所）。
    fn drop_line(
        &self,
        bars: Scrollbars,
        window: WindowView,
        palette: Palette,
    ) -> Option<DrawCommand> {
        let Some(PointerHold::Cells {
            kind: PressKind::MoveColumns,
            dragging: true,
            at,
            ..
        }) = self.hold
        else {
            return None;
        };
        let metrics = window.content_metrics();
        let layout = self.layout(self.first_column, metrics);
        let before = self.drop_index(at, window);
        let line = metrics.px(DROP_LINE);
        // 前の列の右端。先頭なら行番号の列の右端、隠れた列（固定した列と表示の最初の列の間）の後ろなら固定した列の右端。
        let x = before
            .checked_sub(1)
            .and_then(|previous| layout.column_right(previous))
            .unwrap_or_else(|| {
                if before == 0 {
                    layout.gutter_width()
                } else if before <= self.first_column {
                    layout.gutter_width().saturating_add(layout.frozen_width())
                } else {
                    layout.table_width()
                }
            });
        Some(DrawCommand::Fill {
            rect: PixelRect::new(
                PixelPoint::new(x.saturating_sub(line / 2), 0),
                PixelSize::new(line, bars.body().size().height()),
            ),
            color: palette.cursor(),
        })
    }

    /// 窓の中の `at` が、行番号（本体の行）・見出しか列の番号（列）・左上の角のどれか。
    fn line_at(&self, at: PixelPoint, window: WindowView) -> Option<LineHit> {
        let bars = self.bars(window);
        let (body, fixed) = (bars.body(), bars.fixed());
        let x = at.x().checked_sub(body.origin().x())?;
        let y = at.y().checked_sub(body.origin().y())?;
        if x >= body.size().width() || y >= body.size().height() {
            return None;
        }
        let in_gutter = x < fixed.gutter();
        if y < fixed.header() {
            if in_gutter {
                return Some(LineHit::All);
            }
            let layout = self.layout(self.first_column, window.content_metrics());
            return layout.column_at(x).map(LineHit::Column);
        }
        if !in_gutter {
            return None;
        }
        let index = (y - fixed.header()) / fixed.row().max(1);
        let index = usize::try_from(index).ok()?;
        self.view.rows().get(index)?;
        Some(LineHit::Row(
            self.view
                .first_body()
                .saturating_add(u64::try_from(index).ok()?),
        ))
    }

    /// 本体の最後の行の番号（ファイルの終わりまでの選択を切る所・索引を作っている間は見積もり）。
    fn last_body(&self) -> u64 {
        self.row_total().saturating_sub(1)
    }

    /// 選んだ範囲を囲む本体の行（ファイルの終わりまでの選択は最後の行で切る）。
    fn row_span(&self) -> (u64, u64) {
        let (first, last) = self.ranges().row_span();
        (first, last.min(self.last_body()))
    }

    /// 縦の表示の先頭が本体の `first` 番目のときの、指 `at` の下のセル。見出しの行より上なら見えている最初の行、
    /// 下の端より下なら見えている最後の行、行番号の列より左なら表示の最初の列、最後の列より右なら最後の列（端で止める）。
    fn cell_toward(&self, at: PointerPoint, first: u64, window: WindowView) -> CellPosition {
        let bars = self.bars(window);
        let (body, fixed) = (bars.body(), bars.fixed());
        let page = self.page_rows(window).max(1);
        let rows_top = i64::from(body.origin().y()) + i64::from(fixed.header());
        let below = i64::from(at.y()) - rows_top;
        let slot = u64::try_from(below.max(0)).unwrap_or(0) / u64::from(fixed.row().max(1));
        let last_row = self.row_total().saturating_sub(1);
        let row = first
            .saturating_add(slot.min(page.saturating_sub(1)))
            .min(last_row);
        let last_column = self.widths.columns().len().saturating_sub(1);
        let right = i64::from(at.x()) - i64::from(body.origin().x());
        let layout = self.layout(self.first_column, window.content_metrics());
        let x = u32::try_from(right.clamp(0, i64::from(body.size().width().saturating_sub(1))))
            .unwrap_or(0);
        let column = if x < fixed.gutter() {
            layout.visible_columns().next().unwrap_or(self.first_column)
        } else {
            layout.column_at(x).unwrap_or(last_column)
        };
        CellPosition::new(row, column.min(last_column))
    }

    /// ドラッグの自動スクロールの 1 刻みの量（縦は行・上が負、横は列・左が負）。指が本体のセルの所の外にある距離から決める。
    /// 最後の列より右の空き・最後の行より下の空きでは流さない（端のセルで止める）。
    fn autoscroll(&self, at: PointerPoint, ticks: u64, window: WindowView) -> (i64, i64) {
        let bars = self.bars(window);
        let (body, fixed) = (bars.body(), bars.fixed());
        let top = i64::from(body.origin().y()) + i64::from(fixed.header());
        let bottom = i64::from(body.origin().y()) + i64::from(body.size().height());
        let left = i64::from(body.origin().x()) + i64::from(fixed.gutter());
        let right = i64::from(body.origin().x()) + i64::from(body.size().width());
        let (x, y) = (i64::from(at.x()), i64::from(at.y()));
        let page = i64::try_from(self.page_rows(window)).unwrap_or(1);
        let rows = if y < top {
            -rows_per_tick(top - y, page)
        } else if y >= bottom {
            rows_per_tick(y - bottom + 1, page)
        } else {
            0
        };
        let more_right = self
            .layout(self.first_column, window.content_metrics())
            .column_right(self.widths.columns().len().saturating_sub(1))
            .is_none_or(|edge| i64::from(edge) > right - i64::from(body.origin().x()));
        let columns = if x < left {
            -columns_per_tick(left - x, ticks)
        } else if x >= right && more_right {
            columns_per_tick(x - right + 1, ticks)
        } else {
            0
        };
        (rows, columns)
    }

    /// セルのドラッグの 1 刻み。指が本体の外なら表示を流し、選択を指の下（端）のセルまで広げる。
    fn tick_cells(&mut self, ticks: u64, window: WindowView) -> DocumentEffect {
        let Some(PointerHold::Cells {
            kind,
            press,
            at,
            dragging: true,
            before,
            ticks: before_ticks,
        }) = self.hold
        else {
            return DocumentEffect::Stay;
        };
        let after = before_ticks.saturating_add(ticks);
        self.hold = Some(PointerHold::Cells {
            kind,
            press,
            at,
            dragging: true,
            before,
            ticks: after,
        });
        let (rows, columns) = self.autoscroll(at, after, window);
        if rows == 0 && columns == 0 {
            return DocumentEffect::Stay;
        }
        let last_column = self.widths.columns().len().saturating_sub(1);
        let shifted = i64::try_from(self.first_column)
            .unwrap_or(0)
            .saturating_add(columns);
        self.first_column = usize::try_from(shifted.max(0))
            .unwrap_or(0)
            .max(self.frozen)
            .min(last_column);
        let page = self.page_rows(window);
        let current = self.view.first_body();
        let target = u64::try_from(
            i64::try_from(current)
                .unwrap_or(i64::MAX)
                .saturating_add(rows)
                .max(0),
        )
        .unwrap_or(0);
        let effect = self.go_to(target, page);
        let first = match effect {
            DocumentEffect::Read { first_body } => first_body,
            DocumentEffect::Stay
            | DocumentEffect::Copy { .. }
            | DocumentEffect::StopTotals
            | DocumentEffect::Save
            | DocumentEffect::SaveAs
            | DocumentEffect::StopSaving
            | DocumentEffect::Find
            | DocumentEffect::StopFinding
            | DocumentEffect::Replace
            | DocumentEffect::StopReplacing
            | DocumentEffect::ReadPasted { .. } => current,
        };
        self.extend_to(at, first, window);
        effect
    }

    /// ドラッグ中のホイール。縦の表示の先頭が `target` へ動いた後の、指の下のセルまで広げる（Excel と同じ）。
    fn follow_wheel(&mut self, target: u64, page: u64, window: WindowView) {
        if let Some(PointerHold::Cells {
            at, dragging: true, ..
        }) = self.hold
        {
            let first = clamp_first(self.view.first_body(), target, page, self.extent().rows());
            self.extend_to(at, first, window);
        }
    }

    /// ドラッグ中の Esc。押す前の選択と表示に戻す。ドラッグしていなければ `None`。
    fn cancel_drag(&mut self, window: WindowView) -> Option<DocumentEffect> {
        if let Some(PointerHold::ColumnWidth {
            column, original, ..
        }) = self.hold
        {
            self.hold = None;
            match original {
                Some(chars) => self.custom.insert(column, chars),
                None => self.custom.remove(&column),
            };
            return Some(DocumentEffect::Stay);
        }
        let Some(PointerHold::Cells {
            dragging: true,
            before,
            ..
        }) = self.hold
        else {
            return None;
        };
        self.hold = None;
        self.selection = before.selection();
        self.others.truncate(before.others());
        self.first_column = before.first_column();
        let page = self.page_rows(window);
        Some(self.go_to(before.first_body(), page))
    }

    /// つまみの始まりを溝の `offset` に置いた位置へ表示を動かす。縦は索引を作った所より先へは動かさない。
    fn drag_thumb(
        &mut self,
        scrollbar: ScrollBar,
        offset: i64,
        window: WindowView,
    ) -> DocumentEffect {
        let position = scrollbar.axis().first_at(scrollbar.length(), offset);
        match scrollbar.orientation() {
            ScrollOrientation::Vertical => {
                let page = self.page_rows(window);
                let reachable = self
                    .reachable_rows()
                    .map_or(position, |rows| position.min(rows.saturating_sub(page)));
                self.go_to(reachable, page)
            }
            ScrollOrientation::Horizontal => {
                self.first_column = self.column_from(position, window);
                DocumentEffect::Stay
            }
        }
    }

    /// 溝の `side` 側を押した 1 回ぶん。縦は 1 画面の行、横は見えている列の数だけ動かす。
    fn page_toward(
        &mut self,
        orientation: ScrollOrientation,
        side: TrackSide,
        window: WindowView,
    ) -> DocumentEffect {
        match orientation {
            ScrollOrientation::Vertical => {
                let page = self.page_rows(window);
                let current = self.view.first_body();
                let target = match side {
                    TrackSide::Before => current.saturating_sub(page),
                    TrackSide::After => current.saturating_add(page),
                };
                self.go_to(target, page)
            }
            ScrollOrientation::Horizontal => {
                let shown = self.columns_shown(window).max(1);
                let last = self.widths.columns().len().saturating_sub(1);
                self.first_column = match side {
                    TrackSide::Before => self.first_column.saturating_sub(shown).max(self.frozen),
                    TrackSide::After => self.first_column.saturating_add(shown).min(last),
                };
                DocumentEffect::Stay
            }
        }
    }

    /// 時計の刻み `ticks` を取り込む。溝を押し続けていれば、押してから 500ms（10 刻み）の後は 1 刻みごとに 1 画面ぶん
    /// 動かし、つまみが指の位置まで来たら止める（Issue #44）。
    pub fn tick(&mut self, ticks: u64, window: WindowView) -> DocumentEffect {
        if self.hold.is_some_and(PointerHold::dragging_cells) {
            return self.tick_cells(ticks, window);
        }
        let Some(PointerHold::Track {
            orientation,
            side,
            along,
            ticks: before,
        }) = self.hold
        else {
            return DocumentEffect::Stay;
        };
        let after = before.saturating_add(ticks);
        self.hold = Some(PointerHold::Track {
            orientation,
            side,
            along,
            ticks: after,
        });
        let reached = self.bars(window).bar(orientation).is_none_or(|scrollbar| {
            scrollbar.thumb_covers(along) || passed(scrollbar, side, along)
        });
        if after <= REPEAT_DELAY_TICKS || reached {
            return DocumentEffect::Stay;
        }
        self.page_toward(orientation, side, window)
    }

    /// 時計の刻みが要るか（溝を押し続けている間と、セルをドラッグしている間）。
    #[must_use]
    pub fn wants_ticks(&self) -> bool {
        matches!(self.hold, Some(PointerHold::Track { .. }))
            || self.hold.is_some_and(PointerHold::dragging_cells)
    }

    /// 指が窓の中の `at` に乗った（`None` は窓の外）。つまみの上かどうかが変わったら `true`（描き直す）。
    pub fn hover(&mut self, at: Option<PixelPoint>, window: WindowView) -> bool {
        let hovered = at.and_then(|at| match self.bars(window).hit(at)? {
            (scrollbar, BarHit::Thumb { .. }) => Some(scrollbar.orientation()),
            (_, BarHit::Track(_)) => None,
        });
        let changed = hovered != self.hovered;
        self.hovered = hovered;
        self.grip = at.is_some_and(|at| self.column_grip(at, window).is_some());
        changed
    }

    /// 窓 `window` の表の所の、本体とスクロールバー。
    fn bars(&self, window: WindowView) -> Scrollbars {
        self.bars_in(self.table_area(window), window.content_metrics())
    }

    /// 表の所 `area` の、本体とスクロールバー。縦は本体の行、横は列の画素で測る。つまみのドラッグ中は押したときの総数。
    fn bars_in(&self, area: PixelRect, metrics: GlyphMetrics) -> Scrollbars {
        let whole = self.layout(0, metrics);
        let row = whole.row_height();
        // 上に固定した行は、見出しの行か、見出しが無ければ列の番号の行（Issue #48・ADR 0034）。
        let header = row.saturating_mul(self.fixed_rows());
        let gutter = whole.gutter_width();
        let rows = match self.hold {
            Some(PointerHold::Thumb {
                orientation: ScrollOrientation::Vertical,
                axis,
                ..
            }) => axis.total(),
            Some(
                PointerHold::Thumb { .. }
                | PointerHold::Track { .. }
                | PointerHold::Cells { .. }
                | PointerHold::ColumnWidth { .. },
            )
            | None => self.row_total(),
        };
        // 横は、固定した列を除いた動く列の画素で測る（VIEW-01）。
        let frozen = whole.frozen_width();
        let columns = u64::from(
            whole
                .table_width()
                .saturating_sub(gutter)
                .saturating_sub(frozen),
        );
        Scrollbars::fit(
            area,
            FixedParts::new(header, row, gutter)
                .with_frozen(frozen)
                .with_scale(metrics.scale()),
            ScrollAxis::new(rows, 0, self.view.first_body()),
            ScrollAxis::new(
                columns,
                0,
                u64::from(column_left(&whole, self.first_column).saturating_sub(frozen)),
            ),
        )
    }

    /// 本体の行の数。索引を作り終えていればその数、作っている間は読んだバイトの割合からの見積もり
    /// （見えている行より少なくはしない）。
    fn row_total(&self) -> u64 {
        let seen = self
            .view
            .first_body()
            .saturating_add(u64::try_from(self.view.rows().len()).unwrap_or(u64::MAX));
        let estimate = match self.index {
            // 作り終えていれば正しい数（足し消しを戻した直後の古い表示の行の数を混ぜない・GitHub Issue #147）。
            JobState::Finished(summary) => return self.body_lines(summary.records()),
            JobState::Running(progress) | JobState::CancelRequested(progress) => {
                let records = u128::from(progress.records()) * u128::from(progress.total())
                    / u128::from(progress.done().max(1));
                self.body_lines(u64::try_from(records).unwrap_or(u64::MAX))
            }
            JobState::Waiting | JobState::Failed | JobState::Cancelled => seen,
        };
        estimate.max(seen)
    }

    /// 索引を作っている間の、目印が付いた本体の行の数（その先へはつまみで動かさない）。作り終えていれば `None`。
    fn reachable_rows(&self) -> Option<u64> {
        match self.index {
            JobState::Running(progress) | JobState::CancelRequested(progress) => {
                Some(self.body_lines(progress.records()))
            }
            JobState::Finished(_) | JobState::Waiting | JobState::Failed | JobState::Cancelled => {
                None
            }
        }
    }

    /// 列の画素 `position`（行番号の列と固定した列を除いた左端から）の所から見せる、表示の最初の列。その位置より後ろで
    /// 最初に始まる列（つまみを右端まで寄せたら最後の列まで見える）。固定した列より前にはしない。
    fn column_from(&self, position: u64, window: WindowView) -> usize {
        let whole = self.layout(0, window.content_metrics());
        let frozen = whole.frozen_width();
        let count = self.widths.columns().len();
        (self.frozen..count)
            .find(|&column| {
                u64::from(column_left(&whole, column).saturating_sub(frozen)) >= position
            })
            .unwrap_or(count.saturating_sub(1))
            .max(self.frozen)
    }

    /// 今の最初の列から、本体の幅に全部入る列の数。
    fn columns_shown(&self, window: WindowView) -> usize {
        let width = self.bars(window).body().size().width();
        let layout = self.layout(self.first_column, window.content_metrics());
        (self.first_column..self.widths.columns().len())
            .take_while(|&column| {
                layout
                    .column_right(column)
                    .is_some_and(|right| right <= width)
            })
            .count()
    }

    /// スクロールバー `scrollbar` の印（選んだ範囲とカーソル）。
    fn marks(&self, scrollbar: ScrollBar, metrics: GlyphMetrics) -> BarMarks {
        let cursor = self.selection.cursor();
        let ranges = self.ranges().all();
        match scrollbar.orientation() {
            ScrollOrientation::Vertical => BarMarks::new(
                ranges
                    .map(|range| {
                        (
                            range.top_left().body(),
                            range.bottom_right().body().saturating_add(1),
                        )
                    })
                    .collect(),
                cursor.body(),
            ),
            ScrollOrientation::Horizontal => {
                let whole = self.layout(0, metrics);
                let left = |column: usize| u64::from(column_left(&whole, column));
                let center = left(cursor.column())
                    .saturating_add(left(cursor.column().saturating_add(1)))
                    / 2;
                BarMarks::new(
                    ranges
                        .map(|range| {
                            (
                                left(range.top_left().column()),
                                left(range.bottom_right().column().saturating_add(1)),
                            )
                        })
                        .collect(),
                    center,
                )
            }
        }
    }

    /// スクロールバー `scrollbar` のつまみの見え方。
    fn thumb_state(&self, scrollbar: ScrollBar) -> ThumbState {
        let orientation = scrollbar.orientation();
        if self
            .hold
            .is_some_and(|hold| hold.orientation() == Some(orientation))
        {
            ThumbState::Held
        } else if self.hovered == Some(orientation) {
            ThumbState::Hovered
        } else {
            ThumbState::Normal
        }
    }

    /// 表の所（大きさ `size`・左上が原点）のスクロールバーの命令。縦のつまみのドラッグ中は、先頭の行の番号の札も。
    fn bar_commands(
        &self,
        bars: Scrollbars,
        view: WindowView,
        palette: Palette,
    ) -> Vec<DrawCommand> {
        let painter = ScrollbarPainter::new(bars, palette);
        let mut commands = painter.frame();
        for scrollbar in bars.bars() {
            commands.extend(painter.bar(
                scrollbar,
                self.thumb_state(scrollbar),
                &self.marks(scrollbar, view.content_metrics()),
            ));
        }
        if let (
            Some(PointerHold::Thumb {
                orientation: ScrollOrientation::Vertical,
                ..
            }),
            Some(scrollbar),
        ) = (self.hold, bars.bar(ScrollOrientation::Vertical))
        {
            let line = self.line_of(self.view.first_body());
            let text = format!("{} 行目", group_digits(line.saturating_add(1)));
            commands.extend(painter.tip(scrollbar, text, view.metrics()));
        }
        commands
    }

    /// 選択の行をレコードの範囲にして読ませる。`MAX_COPY_ROWS` を超えればコピーせず知らせる。
    /// ファイルの終わりまでの選択は、索引を作り終えて行の数が決まるまでコピーしない（GitHub Issue #39。分かった行だけを
    /// コピーして黙って欠けていた）。
    /// 切り取り（Ctrl+X・ADR 0029 の決定 5）。コピーと同じ行を読ませ、`copy_text` で枠を作る。離れた範囲は切り取らない。
    fn cut_request(&mut self) -> DocumentEffect {
        if self.ranges().is_multiple() {
            self.notice = Some(Notice::Paste(PasteNotice::CutManyRanges));
            return DocumentEffect::Stay;
        }
        let effect = self.copy_request();
        if let DocumentEffect::Copy { .. } = effect {
            self.cutting = Some(self.selection);
        }
        effect
    }

    fn copy_request(&mut self) -> DocumentEffect {
        self.cutting = None;
        if self.ranges().copy_shape().is_none() {
            self.notice = Some(Notice::RangesNotAligned);
            return DocumentEffect::Stay;
        }
        let to_end = self
            .ranges()
            .all()
            .any(|range| range.bottom_right().body() == TO_END);
        if to_end && self.records().is_none() {
            self.notice = Some(Notice::CopyAwaitsIndex);
            return DocumentEffect::Stay;
        }
        let (top, bottom) = self.row_span();
        if bottom.saturating_sub(top) >= MAX_COPY_ROWS {
            self.notice = Some(Notice::TooLargeToCopy);
            return DocumentEffect::Stay;
        }
        let first_line = self.line_of(top);
        let last_line = self.line_of(bottom);
        DocumentEffect::Copy {
            first_line,
            count: last_line.saturating_sub(first_line).saturating_add(1),
        }
    }

    /// レコード `first_record` から読んだ行 `rows`（復号したセルの値）から、見出しのレコードを除いて選択の列だけを
    /// タブ区切りにする。`MAX_COPY_BYTES` を超えればコピーせず知らせる。コピーしたら行と列の数を知らせる。
    /// コピーする行が無ければ、クリップボードを替えずに知らせる（GitHub Issue #39）。`cuts` は読んだ行のうち上限で
    /// 切り詰めたレコードで、コピーした行に入っていれば知らせに数える（GitHub Issue #36）。
    pub fn copy_text(
        &mut self,
        first_record: u64,
        rows: &[CellRow],
        cuts: &[CutRecord],
    ) -> Option<ClipboardText> {
        let cutting = self.cutting.take();
        let shape = self.ranges().copy_shape()?;
        let (top, _) = self.ranges().row_span();
        // 書き換えたセルは、書き換えた値でコピーする（画面と同じ）。
        // 切り取りの元のファイルの値は、並べ直す前の原本の行から読む（並べ直した行を並べ直さない・GitHub Issue #153）。
        let source_rows = rows;
        // タブ区切りはセルごとの文字列で作る（写すのは読んだ範囲の分だけ）。
        let rows: Vec<Vec<String>> = if self.edits.is_empty() && self.columns.is_whole() {
            rows.iter().map(CellRow::to_strings).collect()
        } else {
            self.overlaid(first_record, rows)
        };
        let kept: Vec<(&[String], u64)> = self
            .body_cells(first_record, &rows)
            .into_iter()
            .zip(top..)
            .map(|(cells, body)| (cells.as_slice(), body))
            .filter(|&(_, body)| shape.has_row(body))
            .collect();
        let cut = kept
            .iter()
            .filter(|&&(_, body)| {
                let line = self.line_of(body);
                cuts.iter().any(|cut| cut.record() == line)
            })
            .count();
        let body: Vec<&[String]> = kept.into_iter().map(|(cells, _)| cells).collect();
        if body.is_empty() {
            self.notice = Some(Notice::NothingToCopy);
            return None;
        }
        let text = tsv(&body, &self.source_columns(shape.columns()));
        if text.len() > MAX_COPY_BYTES {
            self.notice = Some(Notice::TooLargeToCopy);
            return None;
        }
        let rows_copied = u64::try_from(body.len()).unwrap_or(u64::MAX);
        if let Some(range) = cutting {
            let clears = self.cut_cells(range, first_record, source_rows);
            self.cut = Some(CutMark::new(
                range,
                clears,
                text.clone(),
                self.history.revision(),
            ));
            self.notice = Some(Notice::Paste(PasteNotice::Cut {
                rows: rows_copied,
                columns: shape.column_count(),
            }));
            return Some(ClipboardText::new(text));
        }
        self.notice = Some(Notice::Copied {
            rows: rows_copied,
            columns: shape.column_count(),
            cut: u64::try_from(cut).unwrap_or(u64::MAX),
        });
        Some(ClipboardText::new(text))
    }

    /// 切り取る範囲 `range` の、空でないセルを空にする書き換えと、それぞれのファイルの今の値（位置 `first_record` から
    /// 読んだ行 `rows` から）。「セルが無い」にしたセルは触らない。
    fn cut_cells(
        &self,
        range: Selection,
        first_record: u64,
        rows: &[CellRow],
    ) -> Vec<(CellChange, String)> {
        let (top, bottom) = self.row_span();
        let (left, right) = (range.top_left().column(), range.bottom_right().column());
        let tables: Vec<usize> = (left..=right)
            .map(|display| self.source_column(display))
            .collect();
        self.body_cells(first_record, rows)
            .into_iter()
            .zip(top..=bottom)
            .flat_map(|(cells, body)| {
                let line = self.line_of(body);
                let row = self.rows.row_at(line).unwrap_or(RowId::Source(line));
                let arranged = self.arranged(cells);
                tables
                    .iter()
                    .filter_map(|&table| self.cleared_cell((row, table), &arranged))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    /// 行 `row` の表の `table` 列目（原本のセル `cells`）を空にする書き換えと、そのファイルの今の値。もう空・セルが無い
    /// なら `None`。
    fn cleared_cell(
        &self,
        (row, table): (RowId, usize),
        cells: &[String],
    ) -> Option<(CellChange, String)> {
        let id = self.column_id(table);
        let source = cells.get(table).cloned().unwrap_or_default();
        let shown = match self.edits.get(row, id) {
            Some(CellValue::Text(text)) => text.clone(),
            Some(CellValue::Absent) => return None,
            None => source.clone(),
        };
        (!shown.is_empty()).then(|| (CellChange::new(row, id, shown, String::new()), source))
    }

    /// 選んだ範囲が変わっていれば集計を作り直す（VIEW-14）。1 つのセルなら出さない。
    /// 範囲が見えている行の中なら読み直さずに数え、外なら選んだ行を読ませる（`count_totals` に渡す）。
    /// `MAX_TOTAL_ROWS` を超えれば裏で数えさせる（`totals_report` に知らせる・Issue #43）。
    pub fn refresh_totals(&mut self) -> TotalsRequest {
        let ranges: Vec<Selection> = self.ranges().all().collect();
        // ドラッグ中は数えず、離してから数える（裏の計算が指の動きのたびに始まっては止まるのを避ける）。
        if self.counted.as_ref() == Some(&ranges)
            || self.hold.is_some_and(PointerHold::dragging_cells)
        {
            return TotalsRequest::Unchanged;
        }
        self.counted = Some(ranges);
        let (top, bottom) = (self.selection.top_left(), self.selection.bottom_right());
        if top == bottom && !self.ranges().is_multiple() {
            self.totals = SelectionTotals::Hidden;
            return TotalsRequest::Settled;
        }
        let (first, last) = self.row_span();
        // 見えている行でも、窓の行が範囲の列を持っていなければ、原本から読んで数える（ADR 0033 の決定 3）。
        if let Some(rows) = self
            .visible_rows(first, last)
            .filter(|rows| self.rows_hold_ranges(rows))
        {
            let stats = self.stats_of(first, rows.iter(), |row, column| row.values().get(column));
            self.totals = SelectionTotals::Counted(stats);
            return TotalsRequest::Settled;
        }
        let first_line = self.line_of(first);
        let count = self
            .line_of(last)
            .saturating_sub(first_line)
            .saturating_add(1);
        if last.saturating_sub(first) >= MAX_TOTAL_ROWS {
            // 🔴 裏の集計は原本の連続したレコードを数える。行を足し消しした後は、位置とレコードがずれるので出さない
            // （保存すれば数えられる・ADR 0027）。
            if !self.rows.is_whole() || !self.columns.is_whole() {
                self.totals = SelectionTotals::Failed;
                return TotalsRequest::Settled;
            }
            self.totals = SelectionTotals::Counting(JobProgress::new(0, count, None));
            return TotalsRequest::Background { first_line, count };
        }
        // 読めなければ出さないまま（前の範囲の集計を残さない）。
        self.totals = SelectionTotals::Hidden;
        TotalsRequest::Read { first_line, count }
    }

    /// 本体の `first` 番目から続く行 `rows`（復号した値）を、選んだ範囲の帯で数える（重なりは 1 回だけ）。
    fn stats_of<R, C>(&self, first: u64, rows: impl Iterator<Item = R>, cell: C) -> SelectionStats
    where
        C: for<'row> Fn(&'row R, usize) -> Option<&'row str>,
    {
        let bands = self.ranges().bands();
        let mut stats = SelectionStats::empty();
        for (row, body) in rows.zip(first..) {
            let Some(band) = bands.iter().find(|band| band.rows().contains(&body)) else {
                continue;
            };
            stats.add_cells(
                |column| cell(&row, column),
                &self.source_columns(band.columns()),
            );
        }
        stats
    }

    /// 裏の集計の知らせ（進み具合・数え終えた・止めた・読めなかった）を取り込む。裏で数えている間だけ受け付ける。
    pub fn totals_report(&mut self, totals: SelectionTotals) {
        if let SelectionTotals::Counting(_) = self.totals {
            self.totals = totals;
        }
    }

    /// 選んだ範囲の帯（行はレコードの番号）と、見出しの行の範囲（裏の集計に渡す）。
    #[must_use]
    pub fn totals_scope(&self) -> (Vec<RowBand>, Option<HeaderRows>) {
        let bands = self
            .ranges()
            .bands()
            .into_iter()
            .map(|band| {
                let rows = band.rows();
                let end = (*rows.end()).min(self.last_body());
                RowBand::new(
                    self.line_of(*rows.start())..=self.line_of(end),
                    self.source_columns(band.columns()),
                )
            })
            .collect();
        (bands, self.header_line())
    }

    /// レコード `first_record`（選んだ範囲の最初の行）から読んだ行 `rows`（復号したセルの値）から、見出しのレコードを除いて
    /// 選んだ範囲を集計する（重なりは 1 回だけ）。
    pub fn count_totals(&mut self, first_record: u64, rows: &[CellRow]) {
        let body = self.body_cells(first_record, rows);
        let (first, _) = self.ranges().row_span();
        let stats = self.stats_of(first, body.into_iter(), |row, column| row.get(column));
        self.totals = SelectionTotals::Counted(stats);
    }

    /// 行 `rows` が、選んだ範囲の列（元の列）をどれも持っているか（窓の行が一部の列だけを持つとき・ADR 0033 の決定 3）。
    fn rows_hold_ranges(&self, rows: &[ViewRow]) -> bool {
        let columns: Vec<RangeInclusive<usize>> = self
            .ranges()
            .bands()
            .iter()
            .flat_map(|band| self.source_columns(band.columns()))
            .collect();
        rows.iter().all(|row| {
            row.values().is_whole()
                || columns
                    .iter()
                    .all(|span| span.clone().all(|column| row.values().holds(column)))
        })
    }

    /// 本体の `first` 番目から `last` 番目までが見えていれば、その行。
    fn visible_rows(&self, first: u64, last: u64) -> Option<&[ViewRow]> {
        let start = usize::try_from(first.checked_sub(self.view.first_body())?).ok()?;
        let end = usize::try_from(last.checked_sub(self.view.first_body())?).ok()?;
        self.view.rows().get(start..=end)
    }

    /// レコード `first_record` から読んだ行 `rows` のうち、見出しのレコードを除いたもの。
    /// レコード `first_record` から読んだ行 `rows` に、書き換えたセルを重ねた写し。「セルが無い」にしたセルは空にする。
    fn overlaid(&self, first_record: u64, rows: &[CellRow]) -> Vec<Vec<String>> {
        // 列の数が足りない行にも、足したセルの値を出す（欠けたセルはコピーで空になるので、空で埋めても同じ）。
        let width = self.widths.columns().len();
        rows.iter()
            .zip(first_record..)
            .map(|(cells, line)| {
                let row = self.rows.row_at(line).unwrap_or(RowId::Source(line));
                let cells = self.arranged(cells);
                (0..cells.len().max(width))
                    .map(|column| {
                        let cell = cells.get(column).map_or("", String::as_str);
                        self.overlaid_cell(row, column, cell)
                    })
                    .collect()
            })
            .collect()
    }

    /// 行 `row` の列 `column` の、書き換えを重ねた値。
    fn overlaid_cell(&self, row: RowId, column: usize, cell: &str) -> String {
        match self.edits.get(row, self.column_id(column)) {
            Some(CellValue::Text(value)) => value.clone(),
            Some(CellValue::Absent) => String::new(),
            None => String::from(cell),
        }
    }

    fn body_cells<'rows, T>(&self, first_record: u64, rows: &'rows [T]) -> Vec<&'rows T> {
        let header = self.header_line();
        rows.iter()
            .zip(first_record..)
            .filter(|&(_, line)| !header.is_some_and(|header| header.contains(line)))
            .map(|(cells, _)| cells)
            .collect()
    }

    /// ホイールで動いた先の縦の表示の先頭。横はここで動かす（読み直しが要らない）。
    fn scroll(&mut self, scroll: Scroll, current: u64) -> u64 {
        let last_column = self.widths.columns().len().saturating_sub(1);
        match scroll {
            Scroll::Up { rows } => current.saturating_sub(rows),
            Scroll::Down { rows } => current.saturating_add(rows),
            Scroll::Left { columns } => {
                self.first_column = self.first_column.saturating_sub(columns).max(self.frozen);
                current
            }
            Scroll::Right { columns } => {
                self.first_column = self.first_column.saturating_add(columns).min(last_column);
                current
            }
        }
    }

    /// 表の大きさ（本体の行の数は索引を作り終えてから）。
    fn extent(&self) -> TableExtent {
        let rows = self.records().map(|records| self.body_lines(records));
        TableExtent::new(rows, self.widths.columns().len())
    }

    /// 列 `column` が表の幅に全部入るようにした、表示の最初の列。今の最初の列から見えていれば動かさない。
    /// 固定した列は常に見えているので動かさない。
    fn reveal_column(&self, column: usize, window: WindowView) -> usize {
        if column < self.frozen {
            return self.first_column;
        }
        if column < self.first_column {
            return column;
        }
        let width = self.bars(window).body().size().width();
        // 配置は 1 回だけ作る（候補ごとに作り直すと列の数の 2 乗の手間になる・GitHub Issue #236）。
        self.layout(self.first_column, window.content_metrics())
            .first_column_showing(column, self.first_column, width)
    }

    /// 窓の中の位置 `at` の、本体のセル。見出しの行・行番号の列・見えている行と列の外なら `None`。
    fn cell_at(&self, at: PixelPoint, window: WindowView) -> Option<CellPosition> {
        let area = self.bars(window).body();
        let x = at.x().checked_sub(area.origin().x())?;
        let y = at.y().checked_sub(area.origin().y())?;
        if x >= area.size().width() || y >= area.size().height() {
            return None;
        }
        let layout = self.layout(self.first_column, window.content_metrics());
        let slot = layout.row_at(y);
        // 上の行は上に固定した行（見出しか列の番号）。
        let index = slot.checked_sub(self.fixed_slots())?;
        self.view.rows().get(index)?;
        let body = self
            .view
            .first_body()
            .saturating_add(u64::try_from(index).ok()?);
        Some(CellPosition::new(body, layout.column_at(x)?))
    }

    /// 窓 `window` の表の所（中身から詳細ペインと下の状態の行を除いた所）。
    fn table_area(&self, window: WindowView) -> PixelRect {
        let client = window.chrome().client();
        let bands = ClientBands::split(client.size().height(), window, self.detail);
        PixelRect::new(
            client.origin(),
            PixelSize::new(client.size().width(), bands.table()),
        )
    }

    /// 表示の最初の列 `first_column` からの表の配置（左に固定した列を含む）。
    fn layout(&self, first_column: usize, metrics: GlyphMetrics) -> TableLayout {
        TableLayout::new(
            &self.widths.resized(&self.custom).reordered(&self.order),
            first_column,
            self.last_number(),
            metrics,
        )
        .with_frozen(self.frozen)
    }

    /// 左の `wanted` 列を固定する（0 で外す・表示だけ・原本も列の順も変えない・VIEW-01）。固定した列の幅が本体の幅
    /// （行番号の列を除く）の 2/3 を超えるなら、収まる数まで減らす（横に動く列の場所を残す）。横の表示は固定した列の次から。
    fn freeze_columns(&mut self, wanted: usize, window: WindowView) {
        let room = self
            .bars(window)
            .body()
            .size()
            .width()
            .saturating_sub(self.layout(0, window.content_metrics()).gutter_width());
        let fits = |frozen: usize| {
            self.layout(frozen, window.content_metrics())
                .with_frozen(frozen)
                .frozen_width()
                <= room / 3 * 2
        };
        let frozen = (0..=wanted.min(self.widths.columns().len()))
            .rev()
            .find(|&frozen| fits(frozen))
            .unwrap_or(0);
        self.frozen = frozen;
        self.first_column = self.first_column.max(frozen);
        self.notice = Some(Notice::Frozen { columns: frozen });
    }

    /// 行番号の列の幅を決める最後の行の番号。見えている最後の行と（分かれば）レコードの数の大きいほう。
    fn last_number(&self) -> u64 {
        let visible_last = self
            .view
            .rows()
            .last()
            .map_or(0, |row| row.line().saturating_add(1));
        let lines = self.records().map_or(0, |records| self.rows.len(records));
        visible_last.max(lines)
    }

    /// 状態の行に一度だけ知らせる（次の操作で消える）。
    pub fn notify(&mut self, notice: Notice) {
        self.notice = Some(notice);
    }

    /// 索引の作業のスレッドからの知らせを取り込む。
    ///
    /// 索引を作り終えた知らせなら、今の縦の表示の行を読み直させる（`DocumentEffect::Read`）。開いた直後の行は索引の
    /// 無いまま読んだので、1 MiB を超えるレコードで窓が短く止まっていることがある（GitHub Issue #32）。
    pub fn report(&mut self, report: JobReport) -> DocumentEffect {
        let finished_before = self.records().is_some();
        // 索引がファイル全体で数えた改行の内訳に替える（開くときは先頭の標本だけで数えている・OPEN-08）。
        if let JobReport::Finished(summary) = report
            && let Some(endings) = summary.line_endings()
        {
            self.format = FormatSummary::new(self.format.format(), endings);
        }
        self.index = self.index.apply(report);
        // ファイルの終わりまでの選択は、行の数が決まったとき 1 度だけ数え直す（見積もりで切っていた）。
        if !finished_before
            && self.records().is_some()
            && self
                .ranges()
                .all()
                .any(|range| range.bottom_right().body() == TO_END)
        {
            self.counted = None;
        }
        if !finished_before && self.records().is_some() {
            DocumentEffect::Read {
                first_body: self.view.first_body(),
            }
        } else {
            DocumentEffect::Stay
        }
    }

    /// 見出しのレコードの範囲（無ければ `None`・ADR 0034）。
    #[must_use]
    pub fn header_record(&self) -> Option<HeaderRows> {
        let count = u64::try_from(self.header.len()).ok()?;
        match self.header.first().map(ViewRow::row) {
            Some(RowId::Source(record)) => HeaderRows::new(record, count),
            Some(RowId::Added(_)) | None => None,
        }
    }

    /// 見出しの行の、行の並びの位置の範囲（前書きの行を足し消しすると動く。見出しの行の間には足せないので続いている）。
    fn header_line(&self) -> Option<HeaderRows> {
        let header = self.header_record()?;
        header.at(self.rows.position_of(RowId::Source(header.first()))?)
    }

    /// 上に固定した行の数（見出しの行の数か、見出しが無ければ列の番号の 1 行・ADR 0034）。
    fn fixed_slots(&self) -> usize {
        self.header.len().max(1)
    }

    /// [`Self::fixed_slots`] を画素の掛け算に使う形で。
    fn fixed_rows(&self) -> u32 {
        u32::try_from(self.fixed_slots()).unwrap_or(1)
    }

    /// 原本のレコードの数 `records` のときの、本体の行の数（行の並びの行の数から見出しを除く）。
    fn body_lines(&self, records: u64) -> u64 {
        body_count(self.rows.len(records), self.header_line())
    }

    /// 索引を作り終えていれば、レコードの数。
    fn records(&self) -> Option<u64> {
        match self.index {
            JobState::Finished(summary) => Some(summary.records()),
            JobState::Waiting
            | JobState::Running(_)
            | JobState::CancelRequested(_)
            | JobState::Failed
            | JobState::Cancelled => None,
        }
    }

    /// 窓 `view` の画面。上にタイトルバー、その下に表、下の 1 行に索引の状態、最後に縁の線（ADR 0013）。
    #[must_use]
    pub fn frame(&self, view: WindowView, palette: Palette) -> Frame {
        self.frame_found(view, palette, None)
    }

    /// [`Self::frame`] に、問い `found` に当たる見えているセルと字の印を付けた画面（検索と置換の欄が開いている間・
    /// GitHub Issue #189）。全部の置換の件数を見せている間は、`found` が無くても数えた問いに印を付ける。
    #[must_use]
    pub fn frame_found(
        &self,
        view: WindowView,
        palette: Palette,
        found: Option<&FindQuery>,
    ) -> Frame {
        let client = view.chrome().client();
        let (dx, dy) = (client.origin().x(), client.origin().y());
        let found = found.or_else(|| self.replacing.as_ref().map(|run| run.scope().query()));
        let mut commands: Vec<DrawCommand> = self
            .client_commands(client.size(), view, (palette, found))
            .into_iter()
            .map(|command| command.offset(dx, dy))
            .collect();
        commands.extend(ChromePainter::new(view, palette).title_bar(&self.window_name()));
        commands.extend(ChromePainter::new(view, palette).border());
        if self.closing == CloseState::Asking {
            // 尋ねる画面は窓の真ん中に、表の上に重ねる（位置は窓の左上が原点・押した位置と同じ）。
            commands.extend(ClosePrompt::new(view).paint(palette));
        }
        Frame::new(view.size(), commands)
    }

    /// 中身（大きさ `size`）の命令。左上を原点とする。上に表、下の 1 行に索引の状態。
    fn client_commands(
        &self,
        size: PixelSize,
        view: WindowView,
        (palette, found): (Palette, Option<&FindQuery>),
    ) -> Vec<DrawCommand> {
        let (metrics, content) = (view.metrics(), view.content_metrics());
        let bands = ClientBands::split(size.height(), view, self.detail);
        let table_height = bands.table();
        let status_height = bands.status();
        let area = PixelRect::new(
            PixelPoint::new(0, 0),
            PixelSize::new(size.width(), table_height),
        );
        let bars = self.bars_in(area, content);
        let mut commands = self.table_commands(bars.body().size(), content, (palette, found));
        commands.extend(self.bar_commands(bars, view, palette));
        commands.extend(self.drop_line(bars, view, palette));
        if bands.detail() > 0 {
            let pane = PixelRect::new(
                PixelPoint::new(0, table_height),
                PixelSize::new(size.width(), bands.detail()),
            );
            commands.extend(self.detail_commands(pane, content, palette));
        }
        let status_top = table_height.saturating_add(bands.detail());
        let band = PixelRect::new(
            PixelPoint::new(0, status_top),
            PixelSize::new(size.width(), status_height),
        );
        commands.extend(self.status_commands(band, metrics, palette));
        commands
    }

    /// カーソルのセルの、詳細ペインに出す中身。
    fn detail_content(&self) -> DetailContent<'_> {
        let cursor = self.selection.cursor();
        if let Some(editor) = &self.editor {
            // 編集中は欄の確定した字（確定前の文字列はセルの中だけに出す）。
            return DetailContent::Value(editor.field().text());
        }
        let row = cursor
            .body()
            .checked_sub(self.view.first_body())
            .and_then(|index| usize::try_from(index).ok())
            .and_then(|index| self.view.rows().get(index));
        let Some(row) = row else {
            // 索引を作り終えて本体の行が無いと分かっていれば、「見えている範囲の外」でなくそう言う（GitHub Issue #256）。
            let empty = self
                .records()
                .is_some_and(|records| self.body_lines(records) == 0);
            return if empty {
                DetailContent::NoRows
            } else {
                DetailContent::OutOfView
            };
        };
        let cut = self
            .cuts
            .iter()
            .find(|cut| cut.record() == row.line())
            .copied();
        let column = self.source_column(cursor.column());
        match self.edits.get(row.row(), self.column_id(column)) {
            Some(CellValue::Text(value)) => return DetailContent::Value(value),
            Some(CellValue::Absent) => return DetailContent::Missing,
            None => {}
        }
        // 窓の行がカーソルの列を持っていなければ（読み直す前）、空の値として見せる（「セルが無い」と取り違えない）。
        if !row.values().holds(column) {
            return DetailContent::Value("");
        }
        row.values()
            .get(column)
            .map_or(DetailContent::Missing, |value| match cut {
                Some(cut) => DetailContent::Cut { value, cut },
                None => DetailContent::Value(value),
            })
    }

    /// 見えている行のうち、上限で切り詰めて先頭の部分だけを読んだレコード（読むたびに app が渡す・GitHub Issue #36）。
    /// `show` で行を差し替えると空に戻る。
    pub fn mark_cut(&mut self, cuts: Vec<CutRecord>) {
        self.cuts = cuts;
    }

    /// 詳細ペイン（矩形 `pane`）の命令。
    fn detail_commands(
        &self,
        pane: PixelRect,
        metrics: GlyphMetrics,
        palette: Palette,
    ) -> Vec<DrawCommand> {
        let cursor = self.selection.cursor();
        let name = self.column_name(self.source_column(cursor.column()));
        let place = DetailPlace::new(
            self.line_of(cursor.body()),
            self.source_column(cursor.column()),
            name.as_deref(),
        );
        DetailPainter::new(pane, metrics, palette, self.format.format().encoding())
            .paint(place, self.detail_content())
    }

    /// 状態の行に見せる索引の状態。作り終えていれば、行の数は足し消しの後の行の並びの数（ADR 0027 の決定 1）。
    fn shown_index(&self) -> JobState {
        match self.index {
            JobState::Finished(summary) => JobState::Finished(IndexSummary::new(
                self.rows.len(summary.records()),
                summary.unclosed_quote(),
            )),
            other @ (JobState::Waiting
            | JobState::Running(_)
            | JobState::CancelRequested(_)
            | JobState::Failed
            | JobState::Cancelled) => other,
        }
    }

    /// 状態の行の左の文言。索引の状態 · 読み方 · 列の数 · カーソルの位置（VIEW-13）。
    fn status(&self) -> String {
        let cursor = self.selection.cursor();
        format!(
            "{} · {} · {} 列 · {}",
            index_status(self.shown_index()),
            format_status(self.format),
            self.widths.columns().len(),
            cursor_status(
                self.line_of(cursor.body()),
                self.source_column(cursor.column())
            )
        )
    }

    /// 状態の行（帯 `band`）の命令。左に状態の文言、右端に集計かパレットの案内。
    fn status_commands(
        &self,
        band: PixelRect,
        metrics: GlyphMetrics,
        palette: Palette,
    ) -> Vec<DrawCommand> {
        let (status_top, status_height) = (band.origin().y(), band.size().height());
        let width = band.size().width();
        let spare = status_height.saturating_sub(metrics.line_height());
        let baseline_y = status_top
            .saturating_add(spare / 2)
            .saturating_add(metrics.ascent());
        // 右端に選んだ範囲の集計、無ければパレットの案内（Issue #41・#43）。左の文言はその手前で切る。
        let right = self.right_status();
        let hint_width = display_columns(&right)
            .saturating_mul(metrics.half_width())
            .saturating_add(metrics.half_width().saturating_mul(2));
        let hint_left = width.saturating_sub(hint_width);
        // 左の文言は右端の文言の全角 1 字ぶん手前で切る（続けて読み違えないよう）。
        let left_width = hint_left.saturating_sub(metrics.half_width().saturating_mul(2));
        let left_clip = PixelRect::new(band.origin(), PixelSize::new(left_width, status_height));
        let hint_clip = PixelRect::new(
            PixelPoint::new(hint_left, status_top),
            PixelSize::new(hint_width, status_height),
        );
        vec![
            DrawCommand::Fill {
                rect: band,
                color: palette.gutter(),
            },
            // 状態の行が表に溶けないよう、上に罫線。
            DrawCommand::Fill {
                rect: PixelRect::new(
                    PixelPoint::new(0, status_top),
                    PixelSize::new(width, metrics.px(GRID_LINE).min(status_height)),
                ),
                color: palette.grid(),
            },
            DrawCommand::Text {
                baseline: PixelPoint::new(metrics.half_width(), baseline_y),
                text: self.status(),
                color: palette.gutter_text(),
                clip: left_clip,
                role: TextRole::Chrome,
            },
            DrawCommand::Text {
                baseline: PixelPoint::new(hint_left, baseline_y),
                text: right,
                color: palette.gutter_text(),
                clip: hint_clip,
                role: TextRole::Chrome,
            },
        ]
    }

    /// 状態の行の右端の文言。ドラッグ中は選んでいる行と列の数（Excel の「5R x 3C」）。一度だけの知らせがあればそれ
    /// （次の操作で消え、集計に戻る・Issue #46）。そうでなければ選んだ範囲の集計（VIEW-14・Excel と同じ右端）、出さなければパレットの案内（気づけるように）。
    fn right_status(&self) -> String {
        if let Some(PointerHold::Cells {
            kind: PressKind::MoveColumns,
            dragging: true,
            ..
        }) = self.hold
        {
            return String::from("離した所へ列を移します（Esc でやめる）");
        }
        if self.hold.is_some_and(PointerHold::dragging_cells) {
            let (top, bottom) = (self.selection.top_left(), self.selection.bottom_right());
            return format!(
                "{} 行 × {} 列",
                group_digits(
                    bottom
                        .body()
                        .min(self.last_body())
                        .saturating_sub(top.body())
                        .saturating_add(1)
                ),
                bottom
                    .column()
                    .saturating_sub(top.column())
                    .saturating_add(1)
            );
        }
        if let Some(notice) = self.notice {
            return notice_status(notice);
        }
        if self.closing == CloseState::AfterSave {
            return String::from("保存してから閉じます…");
        }
        if self.saving {
            return saving_status(self.save_progress);
        }
        if let Some(run) = &self.finding {
            return finding_status(run.progress());
        }
        if let Some(run) = &self.replacing {
            return replacing_status(run);
        }
        if let Some(editor) = &self.editor {
            return String::from(match editor.mode() {
                EditMode::Entering => EDITING_HINT_ENTERING,
                EditMode::Editing => EDITING_HINT_EDITING,
            });
        }
        totals_status(self.totals).unwrap_or_else(|| String::from(PALETTE_HINT))
    }

    /// 表（大きさ `size`）の命令。表示の最初の列から、選択を付けて描く。
    fn table_commands(
        &self,
        size: PixelSize,
        metrics: GlyphMetrics,
        (palette, found): (Palette, Option<&FindQuery>),
    ) -> Vec<DrawCommand> {
        let layout = self.layout(self.first_column, metrics);
        let editing = self.editing_cell();
        let ids = if self.columns.is_whole() {
            Vec::new()
        } else {
            self.table_ids()
        };
        FramePainter::new(&layout, metrics, palette, size)
            .with_ranges(self.ranges())
            .with_cut(self.live_cut())
            .with_order(&self.order)
            .with_column_ids(&ids)
            .with_edits(&self.edits)
            .with_editing(editing.as_ref())
            .with_found(found)
            .paint(self.header(), self.column_numbers().as_ref(), &self.view)
    }

    /// まだ使える切り取りの範囲（その後に表が変わっていない）。
    fn live_cut(&self) -> Option<Selection> {
        self.cut
            .as_ref()
            .filter(|mark| mark.is_current(self.history.revision()))
            .map(CutMark::range)
    }

    /// 見出しの行が無い表の、列の番号の行（1 始まり・Issue #48）。見出しの行があれば `None`。
    fn column_numbers(&self) -> Option<ViewRow> {
        if !self.header.is_empty() {
            return None;
        }
        let numbers: Vec<String> = (1..=self.widths.columns().len())
            .map(|number| format!("{number}"))
            .collect();
        Some(ViewRow::new(0, &numbers))
    }
}

/// 行 `rows` の表示の文字列から、今より多い列の幅を決めて `widths` に足す（VIEW-02・今ある列の幅は変えない）。
/// 読んだ行 `rows` の中の、見出しの行 `header`（ファイルが短ければ、ある行だけ）。
fn header_rows_in(header: Option<HeaderRows>, rows: &[CellRow]) -> Vec<ViewRow> {
    header
        .iter()
        .flat_map(|header| header.lines())
        .map_while(|record| {
            let cells = rows.get(usize::try_from(record).ok()?)?;
            Some(ViewRow::from_values(record, cells.clone()))
        })
        .collect()
}

/// 見出しの行 `header`（上から）の、行の並びの位置の範囲。
fn lines_of(header: &[ViewRow]) -> Option<HeaderRows> {
    HeaderRows::new(header.first()?.line(), u64::try_from(header.len()).ok()?)
}

/// 小さい順の列の番号 `columns`（重ならない）を、続く列をまとめた範囲の並びにする。
fn spans_of(columns: &[usize]) -> Vec<Range<usize>> {
    let mut spans: Vec<Range<usize>> = Vec::new();
    for &column in columns {
        match spans.last_mut() {
            Some(last) if last.end == column => last.end = column.saturating_add(1),
            Some(_) | None => spans.push(column..column.saturating_add(1)),
        }
    }
    spans
}

/// 窓の行が一部の列だけを持つなら、その列の範囲（窓の行は 1 回の読みで同じ列を持つ・ADR 0033 の決定 1）を見て、まだ
/// 決めていない列の幅は持っている列だけ決める。持っていない列の仮の幅は、見出しの名前の幅（見出しが無ければ下限）。
fn widen_to(widths: &mut ColumnWidths, rows: &[&ViewRow]) {
    let all = 0..usize::MAX;
    let held = rows
        .iter()
        .find_map(|row| row.values().held_spans())
        .unwrap_or(core::slice::from_ref(&all));
    let header = rows.iter().find(|row| row.values().is_whole());
    widths.widen_held(
        rows,
        (
            |row: &&ViewRow| row.cells().len(),
            |row: &&ViewRow, column| row.cells().get(column),
        ),
        (held, |column| {
            header
                .and_then(|row| row.cells().get(column))
                .map_or(MIN_COLUMN_COLUMNS, display_columns)
        }),
    );
}

/// 列 `column` の左端（行番号の列を除いた画素・`whole` は最初の列からの配置）。
fn column_left(whole: &TableLayout, column: usize) -> u32 {
    let gutter = whole.gutter_width();
    match column.checked_sub(1) {
        Some(previous) => whole
            .column_right(previous)
            .unwrap_or_else(|| whole.table_width())
            .saturating_sub(gutter),
        None => 0,
    }
}

/// 窓の中の位置 `at` を、窓の外にも出うる指の位置にする。
fn pointer_of(at: PixelPoint) -> PointerPoint {
    PointerPoint::new(
        i32::try_from(at.x()).unwrap_or(i32::MAX),
        i32::try_from(at.y()).unwrap_or(i32::MAX),
    )
}

/// 押した所 `press` から `at` が `DRAG_THRESHOLD`（画面の倍率を掛けたもの）を超えて離れたか。
fn is_far(press: PointerPoint, at: PointerPoint, metrics: GlyphMetrics) -> bool {
    let threshold = i64::from(metrics.px(DRAG_THRESHOLD));
    let dx = i64::from(at.x()) - i64::from(press.x());
    let dy = i64::from(at.y()) - i64::from(press.y());
    dx.abs() > threshold || dy.abs() > threshold
}

/// 縁から `distance` 画素出たときの、1 刻みに流す行の数。24 までは 1、72 までは 3、その先は 2^(d/24)。
/// 上限は 1 画面 `page` の 4 分の 1（1 秒に約 5 画面）。
fn rows_per_tick(distance: i64, page: i64) -> i64 {
    let limit = (page / 4).max(3);
    if distance <= 24 {
        1
    } else if distance <= 72 {
        3
    } else {
        let power = u32::try_from((distance / 24).min(30)).unwrap_or(30);
        2_i64.saturating_pow(power).min(limit).max(3)
    }
}

/// 縁から `distance` 画素出たときの、刻み `ticks` で流す列の数。48 までは 2 刻みに 1 列、その先は 1 刻みに 1 列。
fn columns_per_tick(distance: i64, ticks: u64) -> i64 {
    i64::from(distance > 48 || ticks.is_multiple_of(2))
}

/// 溝の `side` 側を押し続けて、つまみが指の位置 `along` を越えたか（行き過ぎない）。
fn passed(scrollbar: ScrollBar, side: TrackSide, along: i64) -> bool {
    let (offset, length) = scrollbar.thumb();
    match side {
        TrackSide::Before => along >= i64::from(offset),
        TrackSide::After => along < i64::from(offset) + i64::from(length),
    }
}

/// 探している間の状態の行。進み具合が分かれば割合と、やめ方を添える。
fn finding_status(progress: Option<(u64, u64)>) -> String {
    match progress {
        Some((done, total)) if total > 0 && total < u64::MAX => {
            let percent = u128::from(done).saturating_mul(100) / u128::from(total);
            format!("探しています… {}%（Esc でやめる）", percent.min(99))
        }
        Some(_) | None => String::from("探しています…（Esc でやめる）"),
    }
}

/// 全部の置換の状態の行。数えている間は進み具合、数え終えたら件数と決め方。
fn replacing_status(run: &ReplaceRun) -> String {
    if let Some((changes, skipped)) = run.counted() {
        return format!(
            "{} セルを置き換えます{}（Enter で置き換える・Esc でやめる）",
            group_digits(u64::try_from(changes.len()).unwrap_or(u64::MAX)),
            skipped_note(*skipped)
        );
    }
    match run.progress() {
        Some((done, total)) if total > 0 && total < u64::MAX => {
            let percent = u128::from(done).saturating_mul(100) / u128::from(total);
            format!(
                "置き換えるセルを数えています… {}%（Esc でやめる）",
                percent.min(99)
            )
        }
        Some(_) | None => String::from("置き換えるセルを数えています…（Esc でやめる）"),
    }
}

/// 保存の途中の状態の行。進み具合が分かれば割合（書き終える前は 99% で止める）と、やめ方を添える。
fn saving_status(progress: Option<(u64, u64)>) -> String {
    match progress {
        Some((written, total)) if total > 0 => {
            let percent = written.saturating_mul(100) / total;
            format!("保存しています… {}%（Esc でやめる）", percent.min(99))
        }
        Some(_) | None => String::from("保存しています…（Esc でやめる）"),
    }
}

/// 選択を畳んで `step` の向きへ動く意図（編集していないときの Enter・Tab）。
const fn plain_move(step: CursorStep) -> Intent {
    Intent::Move {
        step,
        mode: SelectionMode::Collapse,
    }
}

/// 検索の 1 回分（`run`）で、行の並びの位置 `at` の行のどの列を照らすか（始めたセルは位置 `line`・列 `column`、表の幅
/// `width`）。前へ探す最初の回は始めたセルより後ろ、反対の端から続ける回は始めたセルまで（後ろへ探すときは逆）。
fn find_columns(
    run: &FindRun,
    (at, line): (u64, u64),
    (column, width): (usize, usize),
) -> Range<usize> {
    let after = column.saturating_add(1).min(width);
    match (run.forward(), run.wrapped(), at.cmp(&line)) {
        (true, false, Ordering::Greater)
        | (true, true, Ordering::Less)
        | (false, false, Ordering::Less)
        | (false, true, Ordering::Greater) => 0..width,
        (true, false, Ordering::Equal) => after..width,
        (true, true, Ordering::Equal) => 0..after,
        (false, false, Ordering::Equal) => 0..column.min(width),
        (false, true, Ordering::Equal) => column.min(width)..width,
        (true, false, Ordering::Less)
        | (true, true, Ordering::Greater)
        | (false, false, Ordering::Greater)
        | (false, true, Ordering::Less) => 0..0,
    }
}

/// 表を変える意図か（保存の途中は受けない・SAVE-02）。
fn changes_the_table(intent: Intent) -> bool {
    matches!(
        intent,
        Intent::Type(_)
            | Intent::Edit
            | Intent::Delete
            | Intent::BreakLine
            | Intent::FillSelection
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
            | Intent::RemoveHere
    )
}

/// 読み取り専用の間は止める、表の値・行・列を変える意図（EDIT-13・GitHub Issue #238）。名前を付けて保存と新しい表は
/// 止めない（今の表を変えない）。見る・選ぶ・動く・コピー・検索・表示だけの列の入れ替えと固定も止めない。
const fn changes_when_writable(intent: Intent) -> bool {
    matches!(
        intent,
        Intent::Type(_)
            | Intent::Edit
            | Intent::Delete
            | Intent::BreakLine
            | Intent::FillSelection
            | Intent::Save
            | Intent::Paste
            | Intent::Cut
            | Intent::ReplaceAll
            | Intent::Undo
            | Intent::Redo
            | Intent::InsertRows(_)
            | Intent::RemoveRows
            | Intent::InsertColumns(_)
            | Intent::RemoveColumns
            | Intent::InsertHere
            | Intent::RemoveHere
    )
}

/// 意図 `intent` の後も切り取りの枠を残すか（見るだけ・選ぶだけ・動くだけの意図）。表の値・並び・列の見せる順を変える
/// 意図・Esc・別のコピーと切り取り・直し始めは消す（ADR 0029 の決定 5）。
const fn keeps_cut(intent: Intent) -> bool {
    match intent {
        Intent::Scroll(_)
        | Intent::Move { .. }
        | Intent::Point { .. }
        | Intent::Drag { .. }
        | Intent::Release
        | Intent::GoTo(_)
        | Intent::OpenGoTo
        | Intent::SelectAll
        | Intent::SelectRows
        | Intent::SelectColumns
        | Intent::ResetWidths
        | Intent::FreezeColumns
        | Intent::UnfreezeColumns
        | Intent::ToggleDetail
        | Intent::ToggleReadOnly
        | Intent::Theme(_)
        | Intent::Zoom(_)
        | Intent::OpenPalette
        | Intent::Confirm
        | Intent::Advance(_)
        | Intent::NewTable
        | Intent::Paste
        | Intent::OpenFind { .. }
        | Intent::FindAgain { .. }
        | Intent::FindToggle(_)
        | Intent::ReplaceAll => true,
        Intent::Copy
        | Intent::Cut
        | Intent::Cancel
        | Intent::Type(_)
        | Intent::Erase
        | Intent::Edit
        | Intent::Delete
        | Intent::BreakLine
        | Intent::FillSelection
        | Intent::DoubleClick { .. }
        | Intent::MoveColumns(_)
        | Intent::ResetColumns
        | Intent::Save
        | Intent::SaveAs
        | Intent::Undo
        | Intent::Redo
        | Intent::InsertRows(_)
        | Intent::RemoveRows
        | Intent::InsertColumns(_)
        | Intent::RemoveColumns
        | Intent::InsertHere
        | Intent::RemoveHere => false,
    }
}

/// 束を開いた手の並び（束の中の束も開く）。
fn flattened(steps: &[EditStep]) -> Vec<&EditStep> {
    steps
        .iter()
        .flat_map(|step| match step {
            EditStep::Batch(inner) => flattened(inner),
            EditStep::Cells(_) | EditStep::Rows(_) | EditStep::Columns(_) => vec![step],
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cursor_step::CursorStep;
    use crate::index_summary::IndexSummary;
    use crate::scroll::Scroll;
    use crate::selection_mode::SelectionMode;
    use nene_ledger_core::chrome_button::ChromeButton;
    use nene_ledger_core::color::Color;
    use nene_ledger_core::window_placement::WindowPlacement;

    /// 試しの配色（表の先頭のライト `neutral-light` を導いたもの）。
    fn palette() -> Palette {
        crate::theme_state::ThemeState::at(1).palette()
    }

    /// 索引の知らせが無い間の、1 列の表の状態の行。
    const WAITING: &str =
        "索引を作る準備をしています · UTF-8 · カンマ区切り · LF · 1 列 · 1 行目 · 1 列目";

    /// 詳細ペインを閉じた画面（表と状態の行だけの位置を確かめるテスト）。
    fn closed(mut document: Document) -> Document {
        document.detail = DetailPane::Closed;
        document
    }

    /// UTF-8・カンマ区切り・LF の読み方。
    fn summary() -> FormatSummary {
        use nene_ledger_core::dialect::Dialect;
        use nene_ledger_core::line_ending::LineEnding;
        use nene_ledger_core::line_ending_counts::LineEndingCounts;
        use nene_ledger_core::text_encoding::TextEncoding;
        use nene_ledger_core::text_format::TextFormat;
        let format = TextFormat::new(TextEncoding::Utf8, Dialect::rfc4180());
        FormatSummary::new(
            format,
            LineEndingCounts::tally([LineEnding::Lf].into_iter()),
        )
    }

    fn view(width: u32, height: u32, placement: WindowPlacement) -> WindowView {
        view_over(width, height, placement, None)
    }

    fn view_over(
        width: u32,
        height: u32,
        placement: WindowPlacement,
        hovered: Option<ChromeButton>,
    ) -> WindowView {
        WindowView::new(
            PixelSize::new(width, height),
            GlyphMetrics::new(8, 16, 12),
            placement,
            hovered,
        )
    }

    use nene_ledger_core::find_options::FindOptions;

    fn normal() -> WindowView {
        view(400, 300, WindowPlacement::Normal)
    }

    fn document() -> Document {
        let rows: Vec<Vec<String>> = vec![vec![String::from("a")]];
        closed(Document::open(
            String::from("a.csv — NeNe Ledger"),
            summary(),
            None,
            cell_rows(&rows),
        ))
    }

    /// 文字 `wanted` を書く命令の基準線と切り取りの矩形。
    fn text_at(frame: &Frame, wanted: &str) -> (PixelPoint, PixelRect) {
        frame
            .commands()
            .iter()
            .find_map(|command| match command {
                DrawCommand::Text {
                    text,
                    baseline,
                    clip,
                    ..
                } if text == wanted => Some((*baseline, *clip)),
                DrawCommand::Text { .. }
                | DrawCommand::Fill { .. }
                | DrawCommand::Line { .. }
                | DrawCommand::TextSpan { .. } => None,
            })
            .unwrap_or_else(|| panic!("{wanted} を書く命令が無い"))
    }

    /// 色 `wanted` で塗る矩形。
    fn fills(frame: &Frame, wanted: Color) -> Vec<PixelRect> {
        frame
            .commands()
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

    /// 線の命令の色。
    fn line_colors(frame: &Frame) -> Vec<Color> {
        frame
            .commands()
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Line { color, .. } => Some(*color),
                DrawCommand::Fill { .. }
                | DrawCommand::Text { .. }
                | DrawCommand::TextSpan { .. } => None,
            })
            .collect()
    }

    #[test]
    fn the_status_line_sits_at_the_bottom_of_the_client() {
        let mut document = document();
        document.report(JobReport::Finished(IndexSummary::new(1, None)));
        assert_eq!(
            document.index(),
            JobState::Finished(IndexSummary::new(1, None))
        );
        let frame = document.frame(normal(), palette());
        // 中身は (1, 28) から 398 × 271。状態の行の高さは表の行と同じ 25 で、中身の下に寄せる。
        let (baseline, clip) = text_at(
            &frame,
            "1 行 · UTF-8 · カンマ区切り · LF · 1 列 · 1 行目 · 1 列目",
        );
        // 右端に「Ctrl+Shift+P でコマンド」（23 字 × 8 ＋ 16 = 200）。左の文言はその手前（398 − 200 = 198）から、
        // さらに全角 1 字ぶん（16）手前で切る。
        assert_eq!(
            clip,
            PixelRect::new(PixelPoint::new(1, 274), PixelSize::new(182, 25))
        );
        assert_eq!(baseline, PixelPoint::new(1 + 8, 274 + 4 + 12));
        let (hint, hint_clip) = text_at(&frame, PALETTE_HINT);
        assert_eq!(hint, PixelPoint::new(1 + 198, 274 + 4 + 12));
        assert_eq!(hint_clip.size(), PixelSize::new(200, 25));
        assert_eq!(document.view().rows().len(), 1);
    }

    #[test]
    fn the_table_starts_below_the_title_bar() {
        let frame = document().frame(normal(), palette());
        // 最初の命令は中身の背景。タイトルバー 28 の下・左の縁の線 1 の右から。
        let Some(DrawCommand::Fill { rect, color }) = frame.commands().first() else {
            panic!("最初は中身の背景");
        };
        assert_eq!(*color, palette().background());
        assert_eq!(rect.origin(), PixelPoint::new(1, 28));
        // 見出しの行が無いので、上に列の番号の行（高さ 25）があり、その下に本体の最初の行。
        let (_, number) = text_at(&frame, "1");
        assert_eq!(number.origin().y(), 28);
        let (_, cell) = text_at(&frame, "a");
        assert_eq!(cell.origin().y(), 28 + 25);
    }

    #[test]
    fn the_title_bar_has_the_title_and_three_buttons() {
        let frame = document().frame(normal(), palette());
        assert_eq!(fills(&frame, palette().title_bar()).len(), 1);
        let (baseline, clip) = text_at(&frame, "a.csv — NeNe Ledger");
        // 題名は左から半角 1.5 字。縦は 28 の余り 12 の半分 6 ＋ ascent 12。ボタン 3 つ（42 × 3）と右の縁の線の左で切る。
        assert_eq!(baseline, PixelPoint::new(12, 6 + 12));
        assert_eq!(clip.size(), PixelSize::new(400 - 1 - 126, 28));
        // 印の線は最小化 1・最大化 4・閉じる 2。指が乗っていないので背景を塗らない。
        assert_eq!(line_colors(&frame), vec![palette().title_text(); 7]);
        assert!(fills(&frame, palette().button_hover()).is_empty());
        assert!(fills(&frame, palette().close_hover()).is_empty());
    }

    #[test]
    fn the_border_is_drawn_last() {
        let frame = document().frame(normal(), palette());
        assert_eq!(fills(&frame, palette().border()).len(), 4);
        assert!(matches!(
            frame.commands().last(),
            Some(DrawCommand::Fill { color, .. }) if *color == palette().border()
        ));
    }

    #[test]
    fn a_hovered_close_button_turns_red() {
        let over_close = view_over(400, 300, WindowPlacement::Normal, Some(ChromeButton::Close));
        let frame = document().frame(over_close, palette());
        let close = over_close.chrome().button(ChromeButton::Close);
        assert_eq!(fills(&frame, palette().close_hover()), vec![close]);
        let colors = line_colors(&frame);
        assert_eq!(
            colors.get(5..),
            Some(&[palette().close_hover_text(); 2][..])
        );
        assert_eq!(over_close.hovered(), Some(ChromeButton::Close));
    }

    #[test]
    fn another_hovered_button_is_highlighted() {
        let over_minimize = view_over(
            400,
            300,
            WindowPlacement::Normal,
            Some(ChromeButton::Minimize),
        );
        let frame = document().frame(over_minimize, palette());
        assert_eq!(fills(&frame, palette().button_hover()).len(), 1);
        assert_eq!(line_colors(&frame), vec![palette().title_text(); 7]);
    }

    #[test]
    fn a_maximized_window_has_no_border_and_a_restore_icon() {
        let frame = document().frame(view(400, 300, WindowPlacement::Maximized), palette());
        assert!(fills(&frame, palette().border()).is_empty());
        // 元に戻すの印は前の四角 4 本と後ろの 2 本。
        assert_eq!(line_colors(&frame).len(), 1 + 6 + 2);
        let (_, clip) = text_at(&frame, WAITING);
        assert_eq!(clip.origin(), PixelPoint::new(0, 275));
    }

    #[test]
    fn a_page_excludes_the_title_bar_and_the_border() {
        // 高さ 303: 中身は 303 − 28 − 1 = 274、状態の行 25 と列の番号の行 25 を除いて 224 に高さ 25 の行が 8 行。
        // 最大化すると縁の線が無く 225 で 9 行。
        assert_eq!(
            document().page_rows(view(400, 303, WindowPlacement::Normal)),
            8
        );
        assert_eq!(
            document().page_rows(view(400, 303, WindowPlacement::Maximized)),
            9
        );
        assert_eq!(
            document().page_rows(view(400, 10, WindowPlacement::Normal)),
            1
        );
    }

    fn moving(step: CursorStep) -> Intent {
        Intent::Move {
            step,
            mode: SelectionMode::Collapse,
        }
    }

    #[test]
    fn scrolling_uses_the_record_count_once_the_index_is_finished() {
        let mut document = document();
        let window = view(400, 303, WindowPlacement::Normal);
        // 行の数が分からない間は末尾へ行けない。下へは止めない（1 画面は 8 行）。
        assert_eq!(
            document.apply(moving(CursorStep::Bottom), window),
            DocumentEffect::Stay
        );
        assert_eq!(
            document.apply(Intent::Scroll(Scroll::Down { rows: 9 }), window),
            DocumentEffect::Read { first_body: 9 }
        );
        document.report(JobReport::Finished(IndexSummary::new(100, None)));
        // 末尾の行へカーソルを動かし、最後の画面が埋まる所まで表示を動かす。
        assert_eq!(
            document.apply(moving(CursorStep::Bottom), window),
            DocumentEffect::Read { first_body: 92 }
        );
        assert_eq!(document.selection().cursor(), CellPosition::new(99, 0));
        let rows: Vec<Vec<String>> = vec![vec![String::from("b")]];
        document.show(92, cell_rows(&rows));
        assert_eq!(document.view().first_body(), 92);
        assert_eq!(
            document.apply(Intent::Scroll(Scroll::Down { rows: 1 }), window),
            DocumentEffect::Stay
        );
    }

    #[test]
    fn a_tiny_window_gives_the_title_bar_all_it_has() {
        let frame = document().frame(view(100, 10, WindowPlacement::Normal), palette());
        let (_, clip) = text_at(&frame, WAITING);
        // タイトルバーが窓の高さ 10 を全部取り、中身の高さは 0。
        assert_eq!((clip.origin().y(), clip.size().height()), (10, 0));
        let bars = fills(&frame, palette().title_bar());
        assert_eq!(bars.first().map(|title| title.size().height()), Some(10));
    }

    fn table(cells: &[&[&str]]) -> Vec<CellRow> {
        cells.iter().map(|row| CellRow::new(row)).collect()
    }

    /// セルごとの文字列の行を、窓の行の形にする。
    fn cell_rows(rows: &[Vec<String>]) -> Vec<CellRow> {
        rows.iter().map(|row| CellRow::new(row)).collect()
    }

    fn headed() -> Document {
        let rows = table(&[&["id", "name"], &["1", "a"], &["2", "b"]]);
        closed(Document::open(
            String::from("t"),
            summary(),
            Some(HeaderRows::single(0)),
            rows.clone(),
        ))
    }

    #[test]
    fn the_header_is_frozen_and_left_out_of_the_body() {
        let document = headed();
        assert_eq!(document.header().first().map(ViewRow::line), Some(0));
        let records: Vec<u64> = document.view().rows().iter().map(ViewRow::line).collect();
        assert_eq!(records, vec![1, 2]);
        let frame = document.frame(normal(), palette());
        // 見出しは表の最初の行（タイトルバー 28 の下）、本体の最初の行はその下（行の高さ 25）。
        let (_, id) = text_at(&frame, "id");
        let (_, first) = text_at(&frame, "a");
        assert_eq!(id.origin().y(), 28);
        assert_eq!(first.origin().y(), 28 + 25);
        assert_eq!(document.line_of(0), 1);
    }

    #[test]
    fn a_header_beyond_the_rows_is_dropped() {
        let rows = table(&[&["only"]]);
        let document = Document::open(
            String::from("t"),
            summary(),
            Some(HeaderRows::single(3)),
            rows.clone(),
        );
        assert!(document.header().is_empty());
        assert_eq!(document.view().rows().len(), 1);
    }

    /// 前書き 1 行（レコード 0）と 2 行の見出し（レコード 1・2）の表（ADR 0034）。
    fn two_header_rows() -> (Document, Vec<CellRow>) {
        let rows = table(&[
            &["note"],
            &["id", "n"],
            &["", "円"],
            &["1", "10"],
            &["2", "20"],
        ]);
        let mut document = closed(Document::open(
            String::from("t"),
            summary(),
            HeaderRows::new(1, 2),
            rows.clone(),
        ));
        document.report(JobReport::Finished(IndexSummary::new(5, None)));
        (document, rows)
    }

    #[test]
    fn every_header_row_is_frozen_and_left_out_of_the_body() {
        let (document, _) = two_header_rows();
        let lines: Vec<u64> = document.header().iter().map(ViewRow::line).collect();
        assert_eq!(lines, [1, 2]);
        let records: Vec<u64> = document.view().rows().iter().map(ViewRow::line).collect();
        assert_eq!(records, [0, 3, 4]);
        assert_eq!(document.header_record(), HeaderRows::new(1, 2));
        assert_eq!((document.line_of(0), document.line_of(1)), (0, 3));
        // カーソルはデータの 1 行目（本体の 1 番目）。
        assert_eq!(document.selection().cursor(), CellPosition::new(1, 0));
        // 見出しの 2 行が上に固定され、本体はその下（タイトルバー 28・行の高さ 25）。
        let frame = document.frame(normal(), palette());
        assert_eq!(text_at(&frame, "id").1.origin().y(), 28);
        assert_eq!(text_at(&frame, "円").1.origin().y(), 28 + 25);
        assert_eq!(text_at(&frame, "note").1.origin().y(), 28 + 50);
        // 1 画面の行の数は固定した 2 行を除く（1 行の見出しなら 8 行）。
        assert_eq!(
            document.page_rows(view(400, 303, WindowPlacement::Normal)),
            7
        );
    }

    /// 2 列目の見出し「n」のセルの真ん中の x（列の幅は前書きの行でも変わる。境目の近くは幅を変える所）。
    fn second_column_x(document: &Document) -> u32 {
        let (_, cell) = text_at(&document.frame(normal(), palette()), "n");
        cell.origin().x() + cell.size().width() / 2
    }

    #[test]
    fn column_names_join_the_header_rows_with_a_slash() {
        let (mut document, _) = two_header_rows();
        assert_eq!(document.column_name(0).as_deref(), Some("id"));
        assert_eq!(document.column_name(1).as_deref(), Some("n / 円"));
        assert_eq!(document.column_name(2), None);
        assert_eq!(headed().column_name(1).as_deref(), Some("name"));
        assert_eq!(self::document().column_name(0), None);
        // 詳細ペインの見出しにも出る（本体の 1 番目の 2 列目を押す）。
        let x = second_column_x(&document);
        document.apply(pointing(x, 110, SelectionMode::Collapse), normal());
        document.apply(Intent::Release, normal());
        assert_eq!(document.selection().cursor(), CellPosition::new(1, 1));
        document.detail = DetailPane::Open;
        text_at(&document.frame(normal(), palette()), "n / 円");
    }

    #[test]
    fn any_header_row_selects_the_whole_column() {
        let (mut document, _) = two_header_rows();
        // 2 行目の見出しの 2 列目。
        let x = second_column_x(&document);
        document.apply(pointing(x, 60, SelectionMode::Collapse), normal());
        document.apply(Intent::Release, normal());
        assert_eq!(
            document.selection().bottom_right(),
            CellPosition::new(TO_END, 1)
        );
    }

    #[test]
    fn copying_across_a_header_of_two_rows_skips_both() {
        let (mut document, rows) = two_header_rows();
        document.apply(moving(CursorStep::Up), normal());
        document.apply(
            Intent::Move {
                step: CursorStep::Down,
                mode: SelectionMode::Extend,
            },
            normal(),
        );
        let text = copy_from(&mut document, &rows).unwrap();
        assert_eq!(text.as_str(), "note\r\n1\r\n");
    }

    #[test]
    fn a_page_excludes_the_header_row() {
        // 中身 274 から状態の行と見出しの行（25 ずつ）を除いて 224 に 8 行。
        assert_eq!(
            headed().page_rows(view(400, 303, WindowPlacement::Normal)),
            8
        );
    }

    #[test]
    fn column_widths_stay_while_scrolling_and_grow_for_new_columns() {
        let mut document = headed();
        assert_eq!(document.widths().columns(), &[3, 4]);
        document.show(10, table(&[&["123456789", "a very long name", "extra"]]));
        // 今ある列は変えず、増えた列だけを足す。
        assert_eq!(document.widths().columns(), &[3, 4, 5]);
        assert_eq!(document.view().first_body(), 9);
        // 見出しだけの範囲は差し替えない。
        document.show(0, table(&[&["id", "name"]]));
        assert_eq!(document.view().first_body(), 9);
    }

    #[test]
    fn scrolling_counts_body_rows_once_the_index_is_finished() {
        let mut document = headed();
        document.report(JobReport::Finished(IndexSummary::new(100, None)));
        // 本体は 99 行。1 画面 8 行なら末尾の先頭は本体の 91 番目（レコード 92）。
        let window = view(400, 303, WindowPlacement::Normal);
        assert_eq!(
            document.apply(moving(CursorStep::Bottom), window),
            DocumentEffect::Read { first_body: 91 }
        );
        assert_eq!(document.selection().cursor(), CellPosition::new(98, 1));
        assert_eq!(document.line_of(91), 92);
    }

    fn wide() -> Document {
        let long = "x".repeat(40);
        let row: Vec<String> = (0..10_u32).map(|_| long.clone()).collect();
        closed(Document::open(
            String::from("t"),
            summary(),
            None,
            vec![CellRow::new(&row)],
        ))
    }

    #[test]
    fn moving_right_reveals_the_column_and_row_start_goes_back() {
        // 窓の幅 400 の表の所は 398。行番号の列 41 ＋ 40 字の列 337 で 1 列だけが全部入る。
        let window = view(400, 300, WindowPlacement::Normal);
        let mut document = wide();
        assert_eq!(
            document.apply(moving(CursorStep::Right), window),
            DocumentEffect::Stay
        );
        assert_eq!(document.selection().cursor(), CellPosition::new(0, 1));
        assert_eq!(document.first_column(), 1);
        document.apply(moving(CursorStep::RowEnd), window);
        assert_eq!(document.first_column(), 9);
        document.apply(moving(CursorStep::RowStart), window);
        assert_eq!(document.first_column(), 0);
        let frame = document.frame(window, palette());
        // 1 つのセルの選択: カーソルの枠 4 辺 ＋ 選んだ範囲の外周 4 辺 ＋ 横のスクロールバーの範囲の棒。
        assert_eq!(fills(&frame, palette().cursor()).len(), 9);
    }

    /// 3 字の列が 10（`c0`〜`c9`）。
    fn narrow() -> Document {
        let row: Vec<String> = (0..10_u32).map(|n| format!("c{n}")).collect();
        closed(Document::open(
            String::from("t"),
            summary(),
            None,
            vec![CellRow::new(&row)],
        ))
    }

    fn shown_texts(frame: &Frame) -> Vec<&str> {
        frame
            .commands()
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Text { text, .. } => Some(text.as_str()),
                DrawCommand::Fill { .. }
                | DrawCommand::Line { .. }
                | DrawCommand::TextSpan { .. } => None,
            })
            .collect()
    }

    #[test]
    fn frozen_columns_stay_while_scrolling_sideways() {
        let window = view(300, 300, WindowPlacement::Normal);
        let mut document = narrow();
        document.apply(moving(CursorStep::Right), window);
        document.apply(moving(CursorStep::Right), window);
        document.apply(Intent::FreezeColumns, window);
        assert_eq!(document.frozen_columns(), 3);
        assert_eq!(document.first_column(), 3);
        assert!(
            shown_texts(&document.frame(window, palette()))
                .contains(&"左の 3 列を固定しました（横に動いても残ります）")
        );
        document.apply(Intent::Scroll(Scroll::Right { columns: 4 }), window);
        assert_eq!(document.first_column(), 7);
        let frame = document.frame(window, palette());
        let texts = shown_texts(&frame);
        assert!(texts.contains(&"c0") && texts.contains(&"c2") && texts.contains(&"c7"));
        assert!(!texts.contains(&"c3"));
    }

    #[test]
    fn scrolling_left_stops_at_the_frozen_columns_until_they_are_unfrozen() {
        let window = view(300, 300, WindowPlacement::Normal);
        let mut document = narrow();
        document.apply(moving(CursorStep::Right), window);
        document.apply(moving(CursorStep::Right), window);
        document.apply(Intent::FreezeColumns, window);
        document.apply(Intent::Scroll(Scroll::Right { columns: 4 }), window);
        assert_eq!(document.first_column(), 7);
        // 左へは固定した列の次まで。固定を外せば先頭まで。
        document.apply(Intent::Scroll(Scroll::Left { columns: 30 }), window);
        assert_eq!(document.first_column(), 3);
        document.apply(Intent::UnfreezeColumns, window);
        assert_eq!(document.frozen_columns(), 0);
        assert!(shown_texts(&document.frame(window, palette())).contains(&"列の固定を外しました"));
        document.apply(Intent::Scroll(Scroll::Left { columns: 30 }), window);
        assert_eq!(document.first_column(), 0);
    }

    #[test]
    fn frozen_and_scrolled_cells_are_hit_where_they_are_drawn() {
        let window = view(300, 300, WindowPlacement::Normal);
        let mut document = narrow();
        document.apply(moving(CursorStep::Right), window);
        document.apply(Intent::FreezeColumns, window);
        document.apply(Intent::Scroll(Scroll::Right { columns: 5 }), window);
        assert_eq!((document.frozen_columns(), document.first_column()), (2, 7));
        let layout = document.layout(7, window.content_metrics());
        let body = document.bars(window).body();
        let inside = |column: usize| {
            let rect = layout.cell_rect(1, column).unwrap();
            PixelPoint::new(
                body.origin().x() + rect.origin().x() + 5,
                body.origin().y() + rect.origin().y() + 5,
            )
        };
        document.apply(pressing(inside(0), SelectionMode::Collapse), window);
        document.apply(Intent::Release, window);
        assert_eq!(document.selection().cursor(), CellPosition::new(0, 0));
        document.apply(pressing(inside(8), SelectionMode::Collapse), window);
        document.apply(Intent::Release, window);
        assert_eq!(document.selection().cursor(), CellPosition::new(0, 8));
        // 固定した列へカーソルを戻しても横の表示は動かない。
        document.apply(moving(CursorStep::RowStart), window);
        assert_eq!(document.selection().cursor(), CellPosition::new(0, 0));
        assert_eq!(document.first_column(), 7);
    }

    #[test]
    fn freezing_more_than_fits_keeps_room_for_the_scrolling_columns() {
        let window = view(300, 300, WindowPlacement::Normal);
        let mut document = narrow();
        document.apply(moving(CursorStep::RowEnd), window);
        document.apply(Intent::FreezeColumns, window);
        // 本体の幅から行番号の列 41 を除いた 2/3 に収まる、41 の列の数まで。
        let room = document.bars(window).body().size().width() - 41;
        let expected = usize::try_from(room / 3 * 2 / 41).unwrap();
        assert_eq!(document.frozen_columns(), expected);
        assert!((1..10).contains(&expected));
        assert!(document.first_column() >= expected);
    }

    #[test]
    fn the_wheel_scrolls_sideways_without_moving_the_cursor() {
        let window = view(400, 300, WindowPlacement::Normal);
        let mut document = wide();
        document.apply(Intent::Scroll(Scroll::Right { columns: 3 }), window);
        assert_eq!(document.first_column(), 3);
        document.apply(Intent::Scroll(Scroll::Right { columns: 30 }), window);
        assert_eq!(document.first_column(), 9);
        document.apply(Intent::Scroll(Scroll::Left { columns: 4 }), window);
        assert_eq!(document.first_column(), 5);
        assert_eq!(document.selection().cursor(), CellPosition::new(0, 0));
        assert_eq!(
            document.apply(Intent::Scroll(Scroll::Up { rows: 3 }), window),
            DocumentEffect::Stay
        );
    }

    fn pointing(x: u32, y: u32, mode: SelectionMode) -> Intent {
        Intent::Point {
            at: PixelPoint::new(x, y),
            mode,
        }
    }

    #[test]
    fn clicking_selects_a_body_cell_and_shift_click_extends() {
        let window = normal();
        let mut document = headed();
        // 表の所は (1, 28) から。見出しの行の下（y = 28 + 25 + 5）・1 列目（x = 1 + 41 + 5）は本体の 0 番目。
        document.apply(pointing(47, 58, SelectionMode::Collapse), window);
        assert_eq!(document.selection().cursor(), CellPosition::new(0, 0));
        // 2 列目（1 列目は 3 字で 41）・本体の 1 番目を Shift で。
        document.apply(pointing(1 + 82 + 5, 28 + 55, SelectionMode::Extend), window);
        let selection = document.selection();
        assert_eq!(selection.cursor(), CellPosition::new(1, 1));
        assert!(selection.contains(0, 0) && selection.contains(1, 1));
        // 行の無い所・窓の外では選び直さない（見出しの行と行番号の列は Issue #47 で行全体・列全体を選ぶ）。
        for (x, y) in [(47, 200), (0, 0), (500, 58)] {
            document.apply(pointing(x, y, SelectionMode::Collapse), window);
            assert_eq!(document.selection(), selection, "{x} {y}");
        }
    }

    #[test]
    fn shift_with_keys_extends_the_selection() {
        let window = normal();
        let mut document = headed();
        document.apply(
            Intent::Move {
                step: CursorStep::Down,
                mode: SelectionMode::Extend,
            },
            window,
        );
        document.apply(
            Intent::Move {
                step: CursorStep::Right,
                mode: SelectionMode::Extend,
            },
            window,
        );
        let selection = document.selection();
        assert!(selection.contains(0, 0) && selection.contains(1, 1));
        let frame = document.frame(window, palette());
        // 選んだセル 4 つ ＋ 選んだ行の行番号 2 つ ＋ 選んだ列の見出し 2 つ。
        assert_eq!(fills(&frame, palette().selection()).len(), 8);
    }

    #[test]
    fn the_status_line_follows_the_cursor_in_record_numbers() {
        let window = normal();
        let mut document = headed();
        document.apply(moving(CursorStep::Down), window);
        document.apply(moving(CursorStep::Right), window);
        // 見出しがレコード 0 なので、本体の 1 番目はレコード 2（3 行目）。
        let frame = document.frame(window, palette());
        text_at(
            &frame,
            "索引を作る準備をしています · UTF-8 · カンマ区切り · LF · 2 列 · 3 行目 · 2 列目",
        );
    }

    fn copy_from(document: &mut Document, rows: &[CellRow]) -> Option<ClipboardText> {
        let DocumentEffect::Copy { first_line, count } = document.apply(Intent::Copy, normal())
        else {
            panic!("コピーなら読ませる");
        };
        let end = usize::try_from(first_line + count).ok()?;
        let start = usize::try_from(first_line).ok()?;
        document.copy_text(first_line, rows.get(start..end)?, &[])
    }

    #[test]
    fn copying_reads_the_selected_records_and_leaves_out_the_header() {
        let rows = table(&[&["id", "name"], &["1", "a"], &["2", "b\tc"]]);
        let mut document = Document::open(
            String::from("t"),
            summary(),
            Some(HeaderRows::single(0)),
            rows.clone(),
        );
        let window = normal();
        document.apply(
            Intent::Move {
                step: CursorStep::Down,
                mode: SelectionMode::Extend,
            },
            window,
        );
        document.apply(
            Intent::Move {
                step: CursorStep::Right,
                mode: SelectionMode::Extend,
            },
            window,
        );
        // 本体の 0〜1 番目はレコード 1〜2。見出しは入らない。
        assert_eq!(
            document.apply(Intent::Copy, window),
            DocumentEffect::Copy {
                first_line: 1,
                count: 2
            }
        );
        let text = copy_from(&mut document, &rows).unwrap();
        assert_eq!(text.as_str(), "1\ta\r\n2\t\"b\tc\"\r\n");
        let frame = document.frame(window, palette());
        // 知らせは状態の行の右端に（集計の代わりに）。
        text_at(&frame, "2 行 × 2 列をコピーしました");
        // 次の操作で知らせは消える（1 つのセルに戻ったのでパレットの案内）。
        document.apply(moving(CursorStep::Up), window);
        document.refresh_totals();
        assert_eq!(document.right_status(), PALETTE_HINT);
    }

    #[test]
    fn copying_across_a_header_after_preamble_rows_skips_it() {
        // 前書き 1 行（レコード 0）、見出しはレコード 1。本体の 0〜1 番目はレコード 0 と 2。
        let rows = table(&[&["note"], &["id"], &["1"]]);
        let mut document = Document::open(
            String::from("t"),
            summary(),
            Some(HeaderRows::single(1)),
            rows.clone(),
        );
        // 開いたときのカーソルはデータの 1 行目（GitHub Issue #273）。前書きの行へ上がってから広げる。
        assert_eq!(document.selection().cursor(), CellPosition::new(1, 0));
        document.apply(moving(CursorStep::Up), normal());
        document.apply(
            Intent::Move {
                step: CursorStep::Down,
                mode: SelectionMode::Extend,
            },
            normal(),
        );
        let text = copy_from(&mut document, &rows).unwrap();
        assert_eq!(text.as_str(), "note\r\n1\r\n");
    }

    #[test]
    fn a_huge_selection_is_not_copied() {
        let mut document = document();
        document.report(JobReport::Finished(IndexSummary::new(1_000_000, None)));
        document.apply(
            Intent::Move {
                step: CursorStep::Bottom,
                mode: SelectionMode::Extend,
            },
            normal(),
        );
        assert_eq!(document.apply(Intent::Copy, normal()), DocumentEffect::Stay);
        assert_eq!(
            document.right_status(),
            "選択が大きすぎるのでコピーしませんでした"
        );
        // 文字列が大きすぎるときも同じ。
        let big = vec![vec!["x".repeat(MAX_COPY_BYTES)]];
        let mut small = Document::open(String::from("t"), summary(), None, cell_rows(&big));
        assert_eq!(small.copy_text(0, &cell_rows(&big), &[]), None);
        assert_eq!(
            small.right_status(),
            "選択が大きすぎるのでコピーしませんでした"
        );
    }

    /// 書いた文字の一覧。
    fn all_texts(frame: &Frame) -> Vec<&str> {
        frame
            .commands()
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Text { text, .. } => Some(text.as_str()),
                DrawCommand::Fill { .. }
                | DrawCommand::Line { .. }
                | DrawCommand::TextSpan { .. } => None,
            })
            .collect()
    }

    /// 列の幅（最大 40 字）に収まらない値は `…` で省略して描き、ちょうど収まる値はそのまま描く（GitHub Issue #7）。
    #[test]
    fn values_wider_than_the_column_end_with_an_ellipsis() {
        let long = "x".repeat(60);
        let exact = "y".repeat(40);
        let rows = table(&[&["id", "note"], &["1", &long], &["2", &exact]]);
        let document = closed(opened_with(&rows));
        let frame = document.frame(normal(), palette());
        let texts = all_texts(&frame);
        assert!(
            texts.contains(&format!("{}…", "x".repeat(38)).as_str()),
            "{texts:?}"
        );
        assert!(texts.contains(&exact.as_str()));
        assert!(!texts.contains(&long.as_str()));
    }

    /// 上限で切り詰めた行は、詳細ペインの見出しの行に切り詰めたことと全体の大きさを出す。行を差し替えると外れる
    /// （GitHub Issue #36）。コピーの知らせも切り詰めた値の行を数える。
    #[test]
    fn a_cut_row_says_so_in_the_detail_pane_and_in_the_copy_notice() {
        let rows = table(&[&["id", "memo"], &["1", "short"], &["2", "zzzz"]]);
        let mut document = opened_with(&rows);
        document.set_detail(DetailPane::Open);
        document.report(JobReport::Finished(IndexSummary::new(3, None)));
        document.mark_cut(vec![CutRecord::new(2, Some(2 * 1_048_576))]);
        document.apply(moving(CursorStep::Down), normal());
        let texts = |shown: &Document| -> Vec<String> {
            all_texts(&shown.frame(normal(), palette()))
                .into_iter()
                .map(String::from)
                .collect()
        };
        let cut_line = "2.0 MiB";
        assert!(
            texts(&document).iter().any(|text| text.contains(cut_line)),
            "{:?}",
            texts(&document)
        );
        document.apply(Intent::SelectAll, normal());
        let copied = document.copy_text(1, &rows[1..], &[CutRecord::new(2, None)]);
        assert!(copied.is_some());
        assert_eq!(
            document.notice,
            Some(Notice::Copied {
                rows: 2,
                columns: 2,
                cut: 1
            })
        );
        // 行を差し替えると、切り詰めの印は外れる（読み直した行の分を app が渡し直す）。
        document.show(1, rows[1..].to_vec());
        assert!(!texts(&document).iter().any(|text| text.contains(cut_line)));
    }

    fn opened_with(rows: &[CellRow]) -> Document {
        Document::open(
            String::from("t"),
            summary(),
            Some(HeaderRows::single(0)),
            rows.to_vec(),
        )
    }

    #[test]
    fn the_pane_shows_the_full_value_of_the_cursor_cell() {
        let rows = table(&[&["id", "note"], &["1", "東京都千代田区\n丸の内"]]);
        let mut document = opened_with(&rows);
        assert_eq!(document.detail(), DetailPane::Open);
        let window = view(400, 400, WindowPlacement::Normal);
        document.apply(moving(CursorStep::Right), window);
        let frame = document.frame(window, palette());
        let texts = all_texts(&frame);
        for wanted in [
            "note",
            "2 行目　2 列目　11 文字　UTF-8 で 31 バイト",
            "Ctrl+Shift+U で閉じる",
            "東京都千代田区",
            "丸の内",
        ] {
            assert!(texts.contains(&wanted), "{wanted} が無い: {texts:?}");
        }
        // 本文の左にカーソルの枠と同じ青の帯。
        assert!(
            fills(&frame, palette().cursor())
                .iter()
                .any(|rect| rect.size().width() == 3)
        );
    }

    #[test]
    fn the_pane_says_why_there_is_no_value() {
        let rows = table(&[&["a", "b"], &["", "x"], &["only"]]);
        let window = view(400, 400, WindowPlacement::Normal);
        let mut document = opened_with(&rows);
        assert!(all_texts(&document.frame(window, palette())).contains(&"〈空のセル〉"));
        document.apply(moving(CursorStep::Down), window);
        document.apply(moving(CursorStep::Right), window);
        let texts_missing = document.frame(window, palette());
        assert!(all_texts(&texts_missing).contains(&"〈この行にはこの列のセルが無い〉"));
        // 見出しが無いときは列の番号を名前にし、補助の情報は行だけ。
        let mut bare = Document::open(String::from("t"), summary(), None, rows.clone());
        bare.show(50, rows.clone());
        let texts_out = bare.frame(window, palette());
        let texts = all_texts(&texts_out);
        assert!(texts.contains(&"〈カーソルの行は見えている範囲の外〉"));
        assert!(texts.contains(&"1 列目") && texts.contains(&"1 行目"));
    }

    #[test]
    fn a_long_value_ends_with_the_count_of_hidden_lines() {
        let long = (1..=8_u32)
            .map(|n| alloc::format!("{n}"))
            .collect::<Vec<_>>()
            .join("\n");
        let rows = vec![vec![String::from("h")], vec![long]];
        let document = opened_with(&cell_rows(&rows));
        let frame = document.frame(view(400, 400, WindowPlacement::Normal), palette());
        let texts = all_texts(&frame);
        // 5 行のうち 4 行を出し、残りの 4 行を数える。
        assert!(texts.contains(&"4") && !texts.contains(&"5"));
        assert!(texts.contains(&"〈あと 4 行〉"));
    }

    #[test]
    fn toggling_the_pane_gives_its_rows_back_to_the_table() {
        let window = view(400, 400, WindowPlacement::Normal);
        let mut document = opened_with(&table(&[&["h"], &["1"]]));
        let open = document.page_rows(window);
        document.apply(Intent::ToggleDetail, window);
        assert_eq!(document.detail(), DetailPane::Closed);
        let closed = document.page_rows(window);
        // ペインは 158。高さ 25 の行で 6 行ぶん。
        assert_eq!(closed - open, 6);
        assert!(!all_texts(&document.frame(window, palette())).contains(&"Ctrl+Shift+U で閉じる"));
    }

    #[test]
    fn rules_separate_the_title_bar_and_the_status_line() {
        let frame = document().frame(normal(), palette());
        // タイトルバー（高さ 28）の最後の 1 画素に区切り。状態の行（中身の下・y = 274）の上端に罫線。
        let title = fills(&frame, palette().chrome_rule());
        assert_eq!(
            title,
            vec![PixelRect::new(
                PixelPoint::new(0, 27),
                PixelSize::new(400, 1)
            )]
        );
        assert!(fills(&frame, palette().grid()).contains(&PixelRect::new(
            PixelPoint::new(1, 274),
            PixelSize::new(398, 1)
        )));
    }

    /// 文字コード `encoding` のファイルとして `rows` を開いた画面。
    fn framed_as(
        encoding: nene_ledger_core::text_encoding::TextEncoding,
        rows: &[&[&str]],
    ) -> Frame {
        use nene_ledger_core::dialect::Dialect;
        use nene_ledger_core::text_format::TextFormat;
        let format = FormatSummary::new(
            TextFormat::new(encoding, Dialect::rfc4180()),
            summary().line_endings(),
        );
        Document::open(
            String::from("t"),
            format,
            Some(HeaderRows::single(0)),
            table(rows),
        )
        .frame(view(400, 400, WindowPlacement::Normal), palette())
    }

    #[test]
    fn the_pane_counts_bytes_in_the_file_encoding() {
        use nene_ledger_core::text_encoding::TextEncoding;
        // UTF-16 は 1 字 2 バイト。BOM はセルに関わらないので名前に添えない（Issue #59）。
        let utf16 = framed_as(TextEncoding::Utf16LeWithBom, &[&["id"], &["0012"]]);
        text_at(&utf16, "2 行目　1 列目　4 文字　UTF-16LE で 8 バイト");
        let sjis = framed_as(TextEncoding::ShiftJis, &[&["名"], &["山田ｱ"]]);
        text_at(&sjis, "2 行目　1 列目　3 文字　Shift_JIS で 5 バイト");
    }

    #[test]
    fn the_column_name_in_the_pane_is_not_cut_at_the_estimate() {
        let rows = table(&[&["PRODUCTID"], &["CC-9259"]]);
        let document = opened_with(&rows);
        let frame = document.frame(view(400, 400, WindowPlacement::Normal), palette());
        // 英大文字は近似の幅より長いので、名前は中身の幅 398 の半分 199 まで見せる。補助の情報は近似の 1.5 倍の後ろから。
        // 表の見出しのセルにも同じ文字があるので、後に描くペインのほうを見る。
        let name = frame
            .commands()
            .iter()
            .rev()
            .find_map(|command| match command {
                DrawCommand::Text { text, clip, .. } if text == "PRODUCTID" => Some(*clip),
                DrawCommand::Text { .. }
                | DrawCommand::Fill { .. }
                | DrawCommand::Line { .. }
                | DrawCommand::TextSpan { .. } => None,
            })
            .unwrap();
        assert_eq!(name.size().width(), 199);
        let (aux, _) = text_at(&frame, "2 行目　1 列目　7 文字　UTF-8 で 7 バイト");
        assert_eq!(aux.x(), 1 + 12 + 9 * 8 * 3 / 2 + 8 * 4);
    }

    #[test]
    fn zooming_changes_only_the_table_and_the_pane() {
        use nene_ledger_core::text_role::TextRole;
        let normal = view(400, 400, WindowPlacement::Normal);
        // 中身の字だけを 1.5 倍（行の高さ 25 → 37）にする。
        let zoomed = normal.with_content_metrics(GlyphMetrics::new(12, 24, 18));
        let document = closed(opened_with(&table(&[&["h"], &["1"]])));
        assert!(document.page_rows(zoomed) < document.page_rows(normal));
        let frame = document.frame(zoomed, palette());
        // タイトルバー（28）と状態の行（25）の高さは変わらない。状態の行の字は窓の部品、表の字は中身。
        let roles: Vec<(String, TextRole, PixelRect)> = frame
            .commands()
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Text {
                    text, role, clip, ..
                } => Some((text.clone(), *role, *clip)),
                DrawCommand::Fill { .. }
                | DrawCommand::Line { .. }
                | DrawCommand::TextSpan { .. } => None,
            })
            .collect();
        let status = roles
            .iter()
            .find(|(text, ..)| text.starts_with("索引"))
            .unwrap();
        assert_eq!(status.1, TextRole::Chrome);
        assert_eq!(status.2.size().height(), 25);
        let title = roles.iter().find(|(text, ..)| text == "t").unwrap();
        assert_eq!((title.1, title.2.size().height()), (TextRole::Chrome, 28));
        let cell = roles
            .iter()
            .find(|(text, ..)| text == "1" && text.len() == 1)
            .unwrap();
        assert_eq!(cell.1, TextRole::Content);
        // 見出しの行（中身の行の高さ 37）の下から本体。
        let (_, body) = text_at(&frame, "h");
        assert_eq!(body.size().height(), 36);
    }

    #[test]
    fn a_zoomed_pane_shows_fewer_lines_and_counts_the_rest() {
        let long = [
            "line-a", "line-b", "line-c", "line-d", "line-e", "line-f", "line-g", "line-h",
        ]
        .join("\n");
        let rows = vec![vec![String::from("h")], vec![long]];
        let document = opened_with(&cell_rows(&rows));
        // 175% 相当（行の高さ 28）: 本文は 2 行。1 行を出し、残りの 7 行を数える。
        let zoomed = view(400, 400, WindowPlacement::Normal)
            .with_content_metrics(GlyphMetrics::new(14, 28, 21));
        let frame = document.frame(zoomed, palette());
        let texts = all_texts(&frame);
        assert!(texts.contains(&"line-a") && !texts.contains(&"line-b"));
        assert!(texts.contains(&"〈あと 7 行〉"));
    }

    #[test]
    fn a_very_large_zoom_still_shows_the_first_line_of_the_value() {
        let long = ["line-a", "line-b", "line-c"].join("\n");
        let rows = vec![vec![String::from("h")], vec![long]];
        let document = opened_with(&cell_rows(&rows));
        // 300% 相当（行の高さ 48）でも本文は 2 行: 値の 1 行目と「あと 2 行」。
        let zoomed = view(400, 600, WindowPlacement::Normal)
            .with_content_metrics(GlyphMetrics::new(24, 48, 36));
        let texts_frame = document.frame(zoomed, palette());
        let texts = all_texts(&texts_frame);
        assert!(texts.contains(&"line-a") && !texts.contains(&"line-b"));
        assert!(texts.contains(&"〈あと 2 行〉"));
    }

    fn extend(document: &mut Document, step: CursorStep) -> DocumentEffect {
        document.apply(
            Intent::Move {
                step,
                mode: SelectionMode::Extend,
            },
            normal(),
        )
    }

    #[test]
    fn a_single_cell_has_no_totals_and_a_range_is_counted_without_the_header() {
        let mut document = headed();
        assert_eq!(document.refresh_totals(), TotalsRequest::Settled);
        assert_eq!(document.right_status(), PALETTE_HINT);
        extend(&mut document, CursorStep::Down);
        extend(&mut document, CursorStep::Right);
        // 見えている行の中なので読み直さない。見出しの id と name は数えない。
        assert_eq!(document.refresh_totals(), TotalsRequest::Settled);
        assert_eq!(
            document.right_status(),
            "件数 4 · 合計 3 · 平均 1.5 · 数でない 2"
        );
        // 右端のパレットの案内の代わりに描く。
        let frame = document.frame(view(1000, 300, WindowPlacement::Normal), palette());
        let (totals, _) = text_at(&frame, "件数 4 · 合計 3 · 平均 1.5 · 数でない 2");
        assert!(totals.x() > 500);
        assert!(!frame.commands().iter().any(|command| matches!(
            command,
            DrawCommand::Text { text, .. } if text == PALETTE_HINT
        )));
        // 1 つのセルに戻せば消える。
        document.apply(moving(CursorStep::Up), normal());
        document.refresh_totals();
        assert_eq!(document.right_status(), PALETTE_HINT);
    }

    #[test]
    fn a_range_outside_the_visible_rows_is_read_and_counted() {
        let rows = table(&[&["id", "n"], &["1", "0.1"], &["2", "0.2"]]);
        let mut document = closed(Document::open(
            String::from("t"),
            summary(),
            Some(HeaderRows::single(0)),
            rows.clone(),
        ));
        extend(&mut document, CursorStep::Down);
        extend(&mut document, CursorStep::Right);
        // 表示が 2 行目からだけになると、選んだ範囲の 1 行目が見えていない。
        document.show(2, table(&[&["2", "0.2"]]));
        assert_eq!(
            document.refresh_totals(),
            TotalsRequest::Read {
                first_line: 1,
                count: 2
            }
        );
        assert_eq!(document.right_status(), PALETTE_HINT);
        // 見出しのレコードを含めて渡されても除く。
        document.count_totals(0, &rows);
        assert_eq!(
            document.right_status(),
            "件数 4 · 合計 3.3 · 平均 0.825 · 数でない 0"
        );
        // 範囲が変わらなければ作り直さない。
        assert_eq!(document.refresh_totals(), TotalsRequest::Unchanged);
    }

    #[test]
    fn a_range_over_the_row_limit_is_counted_in_the_background_and_esc_stops_it() {
        let mut document = headed();
        let records = MAX_TOTAL_ROWS + 2;
        document.report(JobReport::Finished(IndexSummary::new(records, None)));
        extend(&mut document, CursorStep::Bottom);
        // 見出しのレコード 0 の次から最後まで。
        assert_eq!(
            document.refresh_totals(),
            TotalsRequest::Background {
                first_line: 1,
                count: MAX_TOTAL_ROWS + 1
            }
        );
        assert_eq!(
            document.totals_scope(),
            (
                vec![RowBand::new(
                    1..=MAX_TOTAL_ROWS + 1,
                    vec![core::ops::RangeInclusive::new(0, 1)]
                )],
                Some(HeaderRows::single(0))
            )
        );
        assert_eq!(
            document.right_status(),
            "合計を計算しています 0%（Esc で止める）"
        );
        let total = MAX_TOTAL_ROWS + 1;
        document.totals_report(SelectionTotals::Counting(JobProgress::new(
            total / 2,
            total,
            None,
        )));
        assert_eq!(
            document.right_status(),
            "合計を計算しています 49%（Esc で止める）"
        );
        // 数えている間の Esc は止めさせる。
        assert_eq!(
            document.apply(Intent::Cancel, normal()),
            DocumentEffect::StopTotals
        );
        document.totals_report(SelectionTotals::Stopped);
        assert_eq!(document.right_status(), "合計を止めました");
        // 止めた後の Esc と、遅れて来た知らせは何もしない。
        assert_eq!(
            document.apply(Intent::Cancel, normal()),
            DocumentEffect::Stay
        );
        document.totals_report(SelectionTotals::Counted(SelectionStats::empty()));
        assert_eq!(document.right_status(), "合計を止めました");
    }

    /// 見出しと 99 行の本体（索引を作り終えた）。
    fn tall() -> Document {
        let mut document = headed();
        document.report(JobReport::Finished(IndexSummary::new(100, None)));
        document
    }

    fn pressing(at: PixelPoint, mode: SelectionMode) -> Intent {
        Intent::Point { at, mode }
    }

    fn center(rect: PixelRect) -> PixelPoint {
        PixelPoint::new(
            rect.origin().x() + rect.size().width() / 2,
            rect.origin().y() + rect.size().height() / 2,
        )
    }

    fn vertical(document: &Document, window: WindowView) -> ScrollBar {
        document
            .bars(window)
            .bar(ScrollOrientation::Vertical)
            .expect("縦のスクロールバー")
    }

    #[test]
    fn a_tall_table_gets_a_vertical_bar_and_the_page_excludes_it() {
        let window = normal();
        assert!(headed().bars(window).bars().next().is_none());
        let document = tall();
        let scrollbar = vertical(&document, window);
        // 縦のバーだけ。表の所 398 のうち 17 を取る。
        assert!(
            document
                .bars(window)
                .bar(ScrollOrientation::Horizontal)
                .is_none()
        );
        assert_eq!(document.bars(window).body().size().width(), 398 - 17);
        assert_eq!(scrollbar.axis().total(), 99);
        let frame = document.frame(window, palette());
        assert_eq!(fills(&frame, palette().scroll_thumb()).len(), 1);
        // バーの上を押してもセルは選ばない。
        let mut pressed = tall();
        pressed.apply(
            pressing(center(scrollbar.track()), SelectionMode::Collapse),
            window,
        );
        assert_eq!(pressed.selection().cursor(), CellPosition::new(0, 0));
    }

    #[test]
    fn dragging_the_thumb_scrolls_and_shows_the_row_tip() {
        let window = normal();
        let mut document = tall();
        let scrollbar = vertical(&document, window);
        let thumb = scrollbar.thumb_rect(false);
        assert_eq!(
            document.apply(pressing(center(thumb), SelectionMode::Collapse), window),
            DocumentEffect::Stay
        );
        // つまみを溝の下の端まで引くと末尾へ。
        let end = PointerPoint::new(390, 2_000);
        let DocumentEffect::Read { first_body } = document.apply(Intent::Drag { at: end }, window)
        else {
            panic!("縦に動く");
        };
        assert_eq!(first_body, 99 - document.page_rows(window));
        let frame = document.frame(window, palette());
        assert!(frame.commands().iter().any(|command| matches!(
            command,
            DrawCommand::Text { text, .. } if text == "2 行目"
        )));
        // 離したら札は消え、ドラッグは終わる。
        document.apply(Intent::Release, window);
        assert_eq!(
            document.apply(Intent::Drag { at: end }, window),
            DocumentEffect::Stay
        );
    }

    #[test]
    fn pressing_the_track_pages_and_repeats_after_a_delay_until_the_thumb_arrives() {
        let window = normal();
        let mut document = tall();
        let scrollbar = vertical(&document, window);
        let page = document.page_rows(window);
        let below = PixelPoint::new(
            center(scrollbar.track()).x(),
            scrollbar.track().origin().y() + scrollbar.length() - 2,
        );
        assert_eq!(
            document.apply(pressing(below, SelectionMode::Collapse), window),
            DocumentEffect::Read { first_body: page }
        );
        assert!(document.wants_ticks());
        assert_eq!(
            document.tick(REPEAT_DELAY_TICKS, window),
            DocumentEffect::Stay
        );
        assert_eq!(
            document.tick(1, window),
            DocumentEffect::Read { first_body: page }
        );
        // つまみが指の位置まで来たら止まる（表示が末尾へ動いた後）。
        let last = table(&[&["98", "x"]]);
        document.show(99, last.clone());
        assert_eq!(document.tick(1, window), DocumentEffect::Stay);
        document.apply(Intent::Release, window);
        assert!(!document.wants_ticks());
    }

    #[test]
    fn shift_pressing_the_track_jumps_there_and_keeps_dragging() {
        let window = normal();
        let mut document = tall();
        let scrollbar = vertical(&document, window);
        let below = PixelPoint::new(
            center(scrollbar.track()).x(),
            scrollbar.track().origin().y() + scrollbar.length() - 2,
        );
        assert!(matches!(
            document.apply(pressing(below, SelectionMode::Extend), window),
            DocumentEffect::Read { .. }
        ));
        assert!(!document.wants_ticks());
        assert_eq!(
            document
                .bars(window)
                .bar(ScrollOrientation::Vertical)
                .map(|held| document.thumb_state(held)),
            Some(ThumbState::Held)
        );
    }

    #[test]
    fn hovering_the_thumb_is_reported_once() {
        let window = normal();
        let mut document = tall();
        let thumb = center(vertical(&document, window).thumb_rect(false));
        assert!(document.hover(Some(thumb), window));
        assert!(!document.hover(Some(thumb), window));
        let frame = document.frame(window, palette());
        assert!(fills(&frame, palette().scroll_thumb()).is_empty());
        assert!(document.hover(None, window));
    }

    /// 索引を作り終えた知らせで 1 度だけ、今の表示の行を読み直させる（GitHub Issue #32。開いた直後の行は索引の
    /// 無いまま読んだので、長いレコードで短く止まっていることがある）。途中の知らせ・2 度目の終わりでは読み直さない。
    #[test]
    fn finishing_the_index_rereads_the_shown_rows_once() {
        let mut document = headed();
        let progress = document.report(JobReport::Progress(
            JobProgress::new(1_000, 4_000, None).with_records(51),
        ));
        assert_eq!(progress, DocumentEffect::Stay);
        let finished = document.report(JobReport::Finished(IndexSummary::new(204, None)));
        assert_eq!(finished, DocumentEffect::Read { first_body: 0 });
        let again = document.report(JobReport::Finished(IndexSummary::new(204, None)));
        assert_eq!(again, DocumentEffect::Stay);
    }

    #[test]
    fn the_thumb_does_not_go_past_the_indexed_rows() {
        let window = normal();
        let mut document = headed();
        // 4,000 バイトのうち 1,000（レコード 51）まで索引を作った。総数は 204 と見積もる。
        document.report(JobReport::Progress(
            JobProgress::new(1_000, 4_000, None).with_records(51),
        ));
        let scrollbar = vertical(&document, window);
        assert_eq!(scrollbar.axis().total(), 203);
        document.apply(
            pressing(center(scrollbar.thumb_rect(false)), SelectionMode::Collapse),
            window,
        );
        let effect = document.apply(
            Intent::Drag {
                at: PointerPoint::new(390, 2_000),
            },
            window,
        );
        assert_eq!(
            effect,
            DocumentEffect::Read {
                first_body: 50 - document.page_rows(window)
            }
        );
    }

    #[test]
    fn the_horizontal_bar_moves_by_columns() {
        let window = view(400, 300, WindowPlacement::Normal);
        let mut document = wide();
        let scrollbar = document
            .bars(window)
            .bar(ScrollOrientation::Horizontal)
            .expect("横のスクロールバー");
        let right = PixelPoint::new(
            scrollbar.track().origin().x() + scrollbar.length() - 2,
            center(scrollbar.track()).y(),
        );
        document.apply(pressing(right, SelectionMode::Collapse), window);
        assert_eq!(document.first_column(), 1);
        document.apply(Intent::Release, window);
        // 動いた後のつまみを押す。
        let moved = document
            .bars(window)
            .bar(ScrollOrientation::Horizontal)
            .expect("横のスクロールバー");
        document.apply(
            pressing(center(moved.thumb_rect(false)), SelectionMode::Collapse),
            window,
        );
        document.apply(
            Intent::Drag {
                at: PointerPoint::new(5_000, 0),
            },
            window,
        );
        assert_eq!(document.first_column(), 9);
        document.apply(
            Intent::Drag {
                at: PointerPoint::new(-5_000, 0),
            },
            window,
        );
        assert_eq!(document.first_column(), 0);
    }

    fn dragging_to(x: i32, y: i32) -> Intent {
        Intent::Drag {
            at: PointerPoint::new(x, y),
        }
    }

    #[test]
    fn dragging_past_the_threshold_extends_and_totals_wait_for_the_release() {
        let window = normal();
        let mut document = headed();
        // 本体の 0 番目・1 列目を押して、4 画素だけ動かしても広げない。
        document.apply(pointing(47, 58, SelectionMode::Collapse), window);
        document.apply(dragging_to(51, 60), window);
        assert_eq!(document.selection(), Selection::at(CellPosition::new(0, 0)));
        // 本体の 1 番目・2 列目まで動かすと広がり、右端は「2 行 × 2 列」。合計はまだ数えない。
        document.apply(dragging_to(88, 83), window);
        assert_eq!(document.selection().cursor(), CellPosition::new(1, 1));
        assert!(document.selection().contains(0, 0));
        assert_eq!(document.right_status(), "2 行 × 2 列");
        assert_eq!(document.refresh_totals(), TotalsRequest::Unchanged);
        document.apply(Intent::Release, window);
        assert_eq!(document.refresh_totals(), TotalsRequest::Settled);
        assert_eq!(
            document.right_status(),
            "件数 4 · 合計 3 · 平均 1.5 · 数でない 2"
        );
    }

    #[test]
    fn dragging_outside_stops_at_the_edge_cells() {
        let window = normal();
        let mut document = headed();
        document.apply(pointing(47, 58, SelectionMode::Collapse), window);
        // 最後の列より右・最後の行より下の空きでは、端のセルで止まり、流さない。
        document.apply(dragging_to(380, 200), window);
        assert_eq!(document.selection().cursor(), CellPosition::new(1, 1));
        assert_eq!(document.tick(1, window), DocumentEffect::Stay);
        // 行番号の列より左と見出しの行より上は、見えている最初の行と表示の最初の列。
        document.apply(dragging_to(-30, 10), window);
        assert_eq!(document.selection().cursor(), CellPosition::new(0, 0));
    }

    #[test]
    fn dragging_below_the_table_scrolls_while_the_pointer_stays() {
        // 1 刻みの上限は 1 画面の 4 分の 1 なので、速さの違いが見える高い窓で。
        let window = view(400, 900, WindowPlacement::Normal);
        let mut document = tall();
        let page = document.page_rows(window);
        document.apply(pointing(47, 58, SelectionMode::Collapse), window);
        let bottom = i32::try_from(document.bars(window).body().size().height()).unwrap() + 28_i32;
        document.apply(dragging_to(47, bottom + 10), window);
        assert!(document.wants_ticks());
        // 縁から 11 画素なので 1 刻みに 1 行。選択は見えている最後の行まで。
        assert_eq!(
            document.tick(1, window),
            DocumentEffect::Read { first_body: 1 }
        );
        assert_eq!(document.selection().cursor().body(), page);
        // 縁から遠いほど速い（100 画素で 3 行より多い）。
        document.show(2, table(&[&["1", "a"]]));
        let effect = document.tick(1, window);
        document.apply(dragging_to(47, bottom + 100), window);
        let faster = document.tick(1, window);
        let (DocumentEffect::Read { first_body: slow }, DocumentEffect::Read { first_body: fast }) =
            (effect, faster)
        else {
            panic!("縦に流れる");
        };
        assert!(fast - 1 > slow - 1 + 2);
        document.apply(Intent::Release, window);
        assert!(!document.wants_ticks());
    }

    #[test]
    fn esc_during_a_drag_restores_the_selection_and_the_view() {
        let window = normal();
        let mut document = tall();
        document.apply(pointing(47, 58, SelectionMode::Collapse), window);
        document.apply(dragging_to(88, 2_000), window);
        document.tick(1, window);
        assert_ne!(document.selection(), Selection::at(CellPosition::new(0, 0)));
        assert_eq!(document.apply(Intent::Cancel, window), DocumentEffect::Stay);
        assert_eq!(document.selection(), Selection::at(CellPosition::new(0, 0)));
        assert!(!document.wants_ticks());
    }

    #[test]
    fn the_wheel_during_a_drag_extends_to_the_cell_under_the_pointer() {
        let window = normal();
        let mut document = tall();
        document.apply(pointing(47, 58, SelectionMode::Collapse), window);
        document.apply(dragging_to(47, 83), window);
        assert_eq!(document.selection().cursor(), CellPosition::new(1, 0));
        assert_eq!(
            document.apply(Intent::Scroll(Scroll::Down { rows: 3 }), window),
            DocumentEffect::Read { first_body: 3 }
        );
        assert_eq!(document.selection().cursor(), CellPosition::new(4, 0));
    }

    /// 見出し（id・n）と本体 3 行の数の表。
    fn numbers() -> Document {
        let rows = table(&[&["id", "n"], &["1", "10"], &["2", "20"], &["3", "30"]]);
        closed(Document::open(
            String::from("t"),
            summary(),
            Some(HeaderRows::single(0)),
            rows.clone(),
        ))
    }

    #[test]
    fn ctrl_click_adds_ranges_and_a_plain_click_starts_over() {
        let window = normal();
        let mut document = numbers();
        document.apply(pointing(47, 58, SelectionMode::Collapse), window);
        document.apply(Intent::Release, window);
        // 本体の 2 番目・2 列目を Ctrl で足す。範囲は 2 つで、カーソルは足したほう。
        document.apply(pointing(88, 108, SelectionMode::Add), window);
        document.apply(Intent::Release, window);
        assert!(document.ranges().is_multiple());
        assert_eq!(document.selection().cursor(), CellPosition::new(2, 1));
        document.refresh_totals();
        assert_eq!(
            document.right_status(),
            "件数 2 · 合計 31 · 平均 15.5 · 数でない 0"
        );
        // 範囲ごとに選択の色: セル 2 つ ＋ 行番号 2 つ ＋ 見出し 2 つ。
        let frame = document.frame(window, palette());
        assert_eq!(fills(&frame, palette().selection()).len(), 6);
        document.apply(pointing(47, 58, SelectionMode::Collapse), window);
        assert!(!document.ranges().is_multiple());
    }

    #[test]
    fn a_cell_chosen_twice_is_counted_once() {
        let window = normal();
        let mut document = numbers();
        // 1 列目の 3 行と、2 列目の 2 行目から 3 行目、さらに 2 列目の 3 行目を Ctrl で足す。
        document.apply(pointing(47, 58, SelectionMode::Collapse), window);
        document.apply(dragging_to(47, 108), window);
        document.apply(Intent::Release, window);
        document.apply(pointing(88, 83, SelectionMode::Add), window);
        document.apply(dragging_to(88, 108), window);
        document.apply(Intent::Release, window);
        document.apply(pointing(88, 108, SelectionMode::Add), window);
        document.apply(Intent::Release, window);
        document.refresh_totals();
        // 1 + 2 + 3 + 20 + 30（30 は 2 回選んだが 1 回だけ）。
        assert_eq!(
            document.right_status(),
            "件数 5 · 合計 56 · 平均 11.2 · 数でない 0"
        );
    }

    #[test]
    fn ranges_on_the_same_rows_are_copied_side_by_side_and_others_are_refused() {
        let window = normal();
        let rows = table(&[&["id", "n"], &["1", "10"], &["2", "20"], &["3", "30"]]);
        let mut document = numbers();
        // 1 行目の 1 列目と 2 列目を別々の範囲で。同じ行なので詰めて 1 行。
        document.apply(pointing(47, 58, SelectionMode::Collapse), window);
        document.apply(Intent::Release, window);
        document.apply(pointing(88, 58, SelectionMode::Add), window);
        document.apply(Intent::Release, window);
        assert_eq!(
            copy_from(&mut document, &rows).map(|text| String::from(text.as_str())),
            Some(String::from("1\t10\r\n"))
        );
        // 同じ列なら上から詰める（間の 2 行目は入れない）。
        document.apply(pointing(47, 58, SelectionMode::Collapse), window);
        document.apply(Intent::Release, window);
        document.apply(pointing(47, 108, SelectionMode::Add), window);
        document.apply(Intent::Release, window);
        assert_eq!(
            copy_from(&mut document, &rows).map(|text| String::from(text.as_str())),
            Some(String::from("1\r\n3\r\n"))
        );
        document.apply(pointing(88, 58, SelectionMode::Add), window);
        document.apply(Intent::Release, window);
        // 行も列も揃っていない範囲はコピーしない。
        assert_eq!(document.apply(Intent::Copy, window), DocumentEffect::Stay);
        assert_eq!(
            document.right_status(),
            "離れた範囲は、行か列が揃っていないとコピーできません"
        );
    }

    #[test]
    fn esc_during_a_ctrl_drag_drops_the_range_being_added() {
        let window = normal();
        let mut document = numbers();
        document.apply(pointing(47, 58, SelectionMode::Collapse), window);
        document.apply(Intent::Release, window);
        document.apply(pointing(88, 83, SelectionMode::Add), window);
        document.apply(dragging_to(88, 108), window);
        document.apply(Intent::Cancel, window);
        assert!(!document.ranges().is_multiple());
        assert_eq!(document.selection(), Selection::at(CellPosition::new(0, 0)));
    }

    #[test]
    fn the_header_selects_a_whole_column_and_the_gutter_a_whole_row() {
        let window = normal();
        let mut document = numbers();
        document.report(JobReport::Finished(IndexSummary::new(4, None)));
        // 2 列目の見出し。全部の行（ファイルの終わりまで）で、カーソルは最初の行。
        document.apply(pointing(88, 35, SelectionMode::Collapse), window);
        document.apply(Intent::Release, window);
        let column = document.selection();
        assert_eq!(column.cursor(), CellPosition::new(0, 1));
        assert_eq!(column.bottom_right(), CellPosition::new(TO_END, 1));
        document.refresh_totals();
        assert_eq!(
            document.right_status(),
            "件数 3 · 合計 60 · 平均 20 · 数でない 0"
        );
        // 行番号は行全体。Shift で前の起点の行から広げる。
        document.apply(pointing(10, 58, SelectionMode::Collapse), window);
        document.apply(Intent::Release, window);
        document.apply(pointing(10, 108, SelectionMode::Extend), window);
        assert_eq!(document.selection().top_left(), CellPosition::new(0, 0));
        assert_eq!(document.selection().bottom_right(), CellPosition::new(2, 1));
        document.apply(Intent::Release, window);
        // 左上の角と Ctrl+A はすべて。
        document.apply(pointing(10, 35, SelectionMode::Collapse), window);
        document.refresh_totals();
        assert_eq!(
            document.right_status(),
            "件数 6 · 合計 66 · 平均 11 · 数でない 0"
        );
        document.apply(moving(CursorStep::Down), window);
        document.apply(Intent::SelectAll, window);
        assert_eq!(
            document.selection().bottom_right(),
            CellPosition::new(TO_END, 1)
        );
    }

    #[test]
    fn dragging_across_headers_widens_whole_columns() {
        let window = normal();
        let mut document = numbers();
        document.apply(pointing(47, 35, SelectionMode::Collapse), window);
        document.apply(dragging_to(88, 40), window);
        assert_eq!(document.selection().top_left(), CellPosition::new(0, 0));
        assert_eq!(
            document.selection().bottom_right(),
            CellPosition::new(TO_END, 1)
        );
        // ドラッグ中の「行 × 列」は本体の行の数で切る。
        assert_eq!(document.right_status(), "3 行 × 2 列");
    }

    #[test]
    fn a_selection_to_the_end_is_recounted_when_the_index_finishes() {
        let window = normal();
        let mut document = numbers();
        document.apply(Intent::SelectAll, window);
        document.refresh_totals();
        assert_eq!(document.refresh_totals(), TotalsRequest::Unchanged);
        document.report(JobReport::Finished(IndexSummary::new(4, None)));
        assert_eq!(document.refresh_totals(), TotalsRequest::Settled);
    }

    #[test]
    fn later_reports_after_the_index_finished_do_not_recount() {
        let window = normal();
        let mut document = numbers();
        document.apply(Intent::SelectAll, window);
        document.report(JobReport::Finished(IndexSummary::new(4, None)));
        assert_eq!(document.refresh_totals(), TotalsRequest::Settled);
        document.report(JobReport::Progress(JobProgress::new(10, 10, None)));
        assert_eq!(document.refresh_totals(), TotalsRequest::Unchanged);
    }

    #[test]
    fn a_table_without_a_header_gets_a_row_of_column_numbers_to_press() {
        let window = normal();
        let rows = table(&[&["1", "10"], &["2", "20"]]);
        let mut document = closed(Document::open(
            String::from("t"),
            summary(),
            None,
            rows.clone(),
        ));
        let frame = document.frame(window, palette());
        // 列の番号の行は行番号の列を空けて「1」「2」。本体の最初の行の行番号も「1」なので、2 回書く。
        let ones = all_texts(&frame)
            .iter()
            .filter(|&&text| text == "1")
            .count();
        assert_eq!(ones, 3);
        // 2 列目の番号を押すと列全体。合計は本体だけ（番号の行は数えない）。
        document.report(JobReport::Finished(IndexSummary::new(2, None)));
        document.apply(pointing(88, 35, SelectionMode::Collapse), window);
        document.apply(Intent::Release, window);
        assert_eq!(
            document.selection().bottom_right(),
            CellPosition::new(TO_END, 1)
        );
        document.refresh_totals();
        assert_eq!(
            document.right_status(),
            "件数 2 · 合計 30 · 平均 15 · 数でない 0"
        );
        // 本体の最初の行は列の番号の行の下。
        document.apply(pointing(47, 58, SelectionMode::Collapse), window);
        assert_eq!(document.selection().cursor(), CellPosition::new(0, 0));
        assert_eq!(
            copy_from(&mut document, &rows).map(|text| String::from(text.as_str())),
            Some(String::from("1\r\n"))
        );
    }

    /// 見出し（a・b・c）と本体 2 行。2 行目は 1 列だけ（セルが無い）。
    fn three() -> Document {
        let rows = table(&[&["a", "b", "c"], &["1", "2", "3"], &["4"]]);
        closed(Document::open(
            String::from("t"),
            summary(),
            Some(HeaderRows::single(0)),
            rows.clone(),
        ))
    }

    fn header_texts(document: &Document) -> Vec<String> {
        let frame = document.frame(normal(), palette());
        all_texts(&frame)
            .into_iter()
            .filter(|text| ["a", "b", "c"].contains(text))
            .map(String::from)
            .collect()
    }

    #[test]
    fn dragging_a_selected_column_header_moves_the_column_for_display_only() {
        let window = normal();
        let rows = table(&[&["a", "b", "c"], &["1", "2", "3"], &["4"]]);
        let mut document = three();
        // 3 列目（c）を選び、もう一度押して 1 列目の左半分へ動かす。
        document.apply(pointing(140, 35, SelectionMode::Collapse), window);
        document.apply(Intent::Release, window);
        document.apply(pointing(140, 35, SelectionMode::Collapse), window);
        document.apply(dragging_to(50, 40), window);
        assert_eq!(
            document.right_status(),
            "離した所へ列を移します（Esc でやめる）"
        );
        // 落とす位置の線は行番号の列の右端（41）に。
        let frame = document.frame(window, palette());
        assert!(
            fills(&frame, palette().cursor())
                .iter()
                .any(|rect| rect.origin().x() == 1 + 41 - 1 && rect.size().width() == 3)
        );
        document.apply(Intent::Release, window);
        assert_eq!(header_texts(&document), ["c", "a", "b"]);
        // 移した列を列全体で選び直す。状態の行は元の列の番号。
        assert_eq!(document.selection().cursor(), CellPosition::new(0, 0));
        assert!(document.status().ends_with("2 行目 · 3 列目"));
        // コピーと合計は表示の順。セルが無い所は空のまま。
        document.apply(pointing(47, 58, SelectionMode::Collapse), window);
        document.apply(dragging_to(140, 83), window);
        document.apply(Intent::Release, window);
        assert_eq!(
            copy_from(&mut document, &rows).map(|text| String::from(text.as_str())),
            Some(String::from("3\t1\t2\r\n\t4\t\r\n"))
        );
        // 元に戻す。
        document.apply(Intent::ResetColumns, window);
        assert_eq!(header_texts(&document), ["a", "b", "c"]);
    }

    #[test]
    fn values_that_look_like_dates_numbers_or_genes_are_drawn_as_they_are() {
        let values = [
            "00123",
            "2026/01/02",
            "1-2",
            "3/4",
            "MARCH1",
            "SEPT2",
            "DEC1",
            "12345678901234567890",
            "1.20E+03",
            "1e5",
            "0x1F",
            "TRUE",
            "NaN",
            "-0",
            "+81-3-1234-5678",
            "001.50",
        ];
        let mut cells: Vec<&[&str]> = vec![&["value"]];
        let rows: Vec<[&str; 1]> = values.iter().map(|&value| [value]).collect();
        cells.extend(rows.iter().map(<[&str; 1]>::as_slice));
        let document = closed(Document::open(
            String::from("t"),
            summary(),
            Some(HeaderRows::single(0)),
            table(&cells),
        ));
        let frame = document.frame(view(800, 900, WindowPlacement::Normal), palette());
        let shown = all_texts(&frame);
        for value in values {
            assert!(shown.contains(&value), "{value}");
        }
    }

    #[test]
    fn duplicate_and_empty_headers_are_drawn_as_they_are_and_pointed_by_position() {
        let window = normal();
        let rows = table(&[
            &["id", "名前", "名前", "", "id"],
            &["1", "山田", "太郎", "x", "10"],
            &["2", "佐藤", "花子", "", "20"],
        ]);
        let mut document = closed(Document::open(
            String::from("t"),
            summary(),
            Some(HeaderRows::single(0)),
            rows.clone(),
        ));
        let frame = document.frame(window, palette());
        let headings: Vec<PixelRect> = frame
            .commands()
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Text { text, clip, .. } if text == "名前" => Some(*clip),
                DrawCommand::Text { .. }
                | DrawCommand::Fill { .. }
                | DrawCommand::Line { .. }
                | DrawCommand::TextSpan { .. } => None,
            })
            .collect();
        // 同じ名前の見出しは 2 つとも、改名せずにそのまま描く。
        assert_eq!(headings.len(), 2);
        assert_eq!(
            all_texts(&frame)
                .iter()
                .filter(|&&text| text == "id")
                .count(),
            2
        );
        // 2 つめの「名前」を押すと、その位置（3 列目）の列全体が選ばれる。
        // 列の境目（左右に数画素・Issue #64）を避けて、見出しの真ん中を押す。
        let second = headings[1];
        document.apply(
            pointing(
                second.origin().x() + second.size().width() / 2,
                second.origin().y() + 2,
                SelectionMode::Collapse,
            ),
            window,
        );
        document.apply(Intent::Release, window);
        assert_eq!(document.selection().cursor(), CellPosition::new(0, 2));
        // 列全体はファイルの終わりまでなので、索引を作り終えてからコピーする（GitHub Issue #39）。
        document.report(JobReport::Finished(IndexSummary::new(3, None)));
        assert_eq!(
            copy_from(&mut document, &rows).map(|text| String::from(text.as_str())),
            Some(String::from("太郎\r\n花子\r\n"))
        );
    }

    /// ファイルの終わりまでの選択は、索引を作り終えるまでコピーせず知らせる。作り終えたら全部の行を読ませる
    /// （GitHub Issue #39。直す前は読んだ 3 行だけをコピーした）。
    #[test]
    fn copying_to_the_end_waits_for_the_index() {
        let rows = table(&[&["n"], &["1"], &["2"], &["3"]]);
        let mut document = closed(Document::open(
            String::from("t"),
            summary(),
            Some(HeaderRows::single(0)),
            rows.clone(),
        ));
        document.apply(Intent::SelectColumns, normal());
        assert_eq!(document.apply(Intent::Copy, normal()), DocumentEffect::Stay);
        assert_eq!(document.notice, Some(Notice::CopyAwaitsIndex));
        document.report(JobReport::Finished(IndexSummary::new(5_001, None)));
        assert_eq!(
            document.apply(Intent::Copy, normal()),
            DocumentEffect::Copy {
                first_line: 1,
                count: 5_000
            }
        );
    }

    /// コピーする行が無ければクリップボードを替えない（見出しだけのファイル・GitHub Issue #39）。
    #[test]
    fn nothing_is_copied_from_a_header_only_file() {
        let rows = table(&[&["id", "name"]]);
        let mut document = closed(Document::open(
            String::from("t"),
            summary(),
            Some(HeaderRows::single(0)),
            rows.clone(),
        ));
        document.report(JobReport::Finished(IndexSummary::new(1, None)));
        document.apply(Intent::SelectAll, normal());
        let effect = document.apply(Intent::Copy, normal());
        if let DocumentEffect::Copy { first_line, .. } = effect {
            assert_eq!(document.copy_text(first_line, &[], &[]), None);
        }
        assert_eq!(document.notice, Some(Notice::NothingToCopy));
    }

    #[test]
    fn keys_select_whole_rows_and_columns_of_the_selection() {
        let window = normal();
        let mut document = three();
        // b1 から c2 まで選び、行全体（1〜2 行目・全部の列）。カーソルの行はそのまま。
        document.apply(moving(CursorStep::Right), window);
        document.apply(
            Intent::Move {
                step: CursorStep::Down,
                mode: SelectionMode::Extend,
            },
            window,
        );
        document.apply(
            Intent::Move {
                step: CursorStep::Right,
                mode: SelectionMode::Extend,
            },
            window,
        );
        document.apply(Intent::SelectRows, window);
        assert_eq!(document.selection().top_left(), CellPosition::new(0, 0));
        assert_eq!(document.selection().bottom_right(), CellPosition::new(1, 2));
        assert_eq!(document.selection().cursor(), CellPosition::new(1, 0));
        // 列全体はファイルの終わりまで。行全体の後なら全部の列。
        document.apply(Intent::SelectColumns, window);
        assert_eq!(document.selection().top_left(), CellPosition::new(0, 0));
        assert_eq!(
            document.selection().bottom_right(),
            CellPosition::new(TO_END, 2)
        );
        // 1 つのセルからなら、その列だけ。離れた範囲は外す。
        document.apply(Intent::GoTo(GoToTarget::parse("2,2").unwrap()), window);
        document.apply(pointing(140, 83, SelectionMode::Add), window);
        document.apply(Intent::Release, window);
        assert!(document.ranges().is_multiple());
        document.apply(Intent::SelectColumns, window);
        assert!(!document.ranges().is_multiple());
        assert_eq!(document.selection().top_left(), CellPosition::new(0, 2));
        assert_eq!(
            document.selection().bottom_right(),
            CellPosition::new(TO_END, 2)
        );
    }

    #[test]
    fn keys_move_the_selected_columns_one_step_for_display_only() {
        let window = normal();
        let mut document = three();
        // 1 列目（a）のセルから右へ 2 回。列全体で選び直される。
        document.apply(Intent::MoveColumns(ColumnShift::Right), window);
        assert_eq!(header_texts(&document), ["b", "a", "c"]);
        assert_eq!(document.selection().top_left(), CellPosition::new(0, 1));
        assert_eq!(
            document.selection().bottom_right(),
            CellPosition::new(TO_END, 1)
        );
        document.apply(Intent::MoveColumns(ColumnShift::Right), window);
        assert_eq!(header_texts(&document), ["b", "c", "a"]);
        // 右の端では動かない。
        document.apply(Intent::MoveColumns(ColumnShift::Right), window);
        assert_eq!(header_texts(&document), ["b", "c", "a"]);
        document.apply(Intent::MoveColumns(ColumnShift::Left), window);
        assert_eq!(header_texts(&document), ["b", "a", "c"]);
        // 2 列（b と a）をまとめて左へ。左の端では動かない。
        document.apply(
            Intent::Move {
                step: CursorStep::Left,
                mode: SelectionMode::Extend,
            },
            window,
        );
        document.apply(Intent::MoveColumns(ColumnShift::Left), window);
        assert_eq!(header_texts(&document), ["b", "a", "c"]);
        document.apply(Intent::MoveColumns(ColumnShift::Right), window);
        assert_eq!(header_texts(&document), ["c", "b", "a"]);
        document.apply(Intent::ResetColumns, window);
        assert_eq!(header_texts(&document), ["a", "b", "c"]);
    }

    /// 見出しの字 `text` の切り取りの幅（列の幅から余白を除いたもの）。
    fn heading_width(document: &Document, text: &str) -> u32 {
        document
            .frame(normal(), palette())
            .commands()
            .iter()
            .find_map(|command| match command {
                DrawCommand::Text {
                    text: shown, clip, ..
                } if shown == text => Some(clip.size().width()),
                DrawCommand::Text { .. }
                | DrawCommand::Fill { .. }
                | DrawCommand::Line { .. }
                | DrawCommand::TextSpan { .. } => None,
            })
            .unwrap()
    }

    /// 見出しの行（y = 35）を左からなぞって、カーソルが左右の矢印になる最初の x。
    fn first_grip(document: &mut Document) -> u32 {
        let window = normal();
        let grip = (0..400_u32)
            .find(|&x| {
                document.hover(Some(PixelPoint::new(x, 35)), window);
                document.pointer_shape() == PointerShape::ColumnResize
            })
            .unwrap();
        document.hover(None, window);
        grip
    }

    #[test]
    fn only_a_header_border_shows_the_resize_cursor() {
        let window = normal();
        let mut document = three();
        let grip = first_grip(&mut document);
        assert_eq!(document.pointer_shape(), PointerShape::Arrow);
        document.hover(Some(PixelPoint::new(grip, 60)), window);
        assert_eq!(document.pointer_shape(), PointerShape::Arrow);
        document.apply(pointing(grip, 35, SelectionMode::Collapse), window);
        assert_eq!(document.pointer_shape(), PointerShape::ColumnResize);
    }

    #[test]
    fn dragging_a_column_border_resizes_it_without_changing_the_selection() {
        let window = normal();
        let mut document = three();
        let before = heading_width(&document, "a");
        // 境目を押して右へ 5 字ぶん（半角の幅 8 × 5）。
        let grip = first_grip(&mut document);
        document.apply(pointing(grip, 35, SelectionMode::Collapse), window);
        document.apply(dragging_to(i32::try_from(grip).unwrap() + 40, 35), window);
        assert_eq!(heading_width(&document, "a"), before + 40);
        document.apply(Intent::Release, window);
        assert_eq!(document.selection(), Selection::at(CellPosition::new(0, 0)));
        // 左へ大きく動かすと下限（3 字）で止まる。Esc でドラッグの前（広げた幅）に戻る。
        let widened = first_grip(&mut document);
        document.apply(pointing(widened, 35, SelectionMode::Collapse), window);
        document.apply(dragging_to(0, 35), window);
        assert_eq!(heading_width(&document, "a"), before);
        document.apply(Intent::Cancel, window);
        assert_eq!(heading_width(&document, "a"), before + 40);
    }

    #[test]
    fn a_resized_width_follows_its_column_until_reset() {
        let window = normal();
        let mut document = three();
        let before = heading_width(&document, "a");
        let grip = first_grip(&mut document);
        document.apply(pointing(grip, 35, SelectionMode::Collapse), window);
        document.apply(dragging_to(i32::try_from(grip).unwrap() + 40, 35), window);
        document.apply(Intent::Release, window);
        document.apply(Intent::MoveColumns(ColumnShift::Right), window);
        assert_eq!(header_texts(&document), ["b", "a", "c"]);
        assert_eq!(heading_width(&document, "a"), before + 40);
        document.apply(Intent::ResetWidths, window);
        assert_eq!(heading_width(&document, "a"), before);
    }

    #[test]
    fn the_middle_of_a_header_still_selects_the_whole_column() {
        let window = normal();
        let mut document = three();
        let grip = first_grip(&mut document);
        // 境目から十分に離れた見出しの真ん中を押すと、今までどおり列全体。
        document.apply(pointing(grip - 12, 35, SelectionMode::Collapse), window);
        document.apply(Intent::Release, window);
        assert_eq!(
            document.selection().bottom_right(),
            CellPosition::new(TO_END, 0)
        );
    }

    #[test]
    fn pressing_a_selected_header_without_moving_selects_just_that_column() {
        let window = normal();
        let mut document = three();
        // 1〜3 列目を選んで、2 列目の見出しを押して離す。
        document.apply(pointing(47, 35, SelectionMode::Collapse), window);
        document.apply(dragging_to(140, 35), window);
        document.apply(Intent::Release, window);
        document.apply(pointing(100, 35, SelectionMode::Collapse), window);
        document.apply(Intent::Release, window);
        assert_eq!(document.selection().top_left(), CellPosition::new(0, 1));
        assert_eq!(
            document.selection().bottom_right(),
            CellPosition::new(TO_END, 1)
        );
        // 選んでいない見出しのドラッグは今までどおり広げる。
        document.apply(pointing(47, 35, SelectionMode::Collapse), window);
        document.apply(dragging_to(100, 35), window);
        assert_eq!(document.selection().bottom_right().column(), 1);
        // Esc で移すのをやめても何も変わらない。
        document.apply(Intent::Release, window);
        document.apply(pointing(47, 35, SelectionMode::Collapse), window);
        document.apply(dragging_to(140, 35), window);
        document.apply(Intent::Cancel, window);
        assert_eq!(header_texts(&document), ["a", "b", "c"]);
    }

    #[test]
    fn going_to_a_row_number_selects_that_cell_and_scrolls_to_it() {
        let window = normal();
        let mut document = tall();
        let page = document.page_rows(window);
        // 52 行目（レコード 51）は、見出しのレコード 0 の後なので本体の 50 番目。
        let effect = document.apply(Intent::GoTo(GoToTarget::parse("52,2").unwrap()), window);
        assert_eq!(
            document.selection(),
            Selection::at(CellPosition::new(50, 1))
        );
        assert_eq!(
            effect,
            DocumentEffect::Read {
                first_body: 50 + 1 - page
            }
        );
        assert!(document.status().ends_with("52 行目 · 2 列目"));
        // 行の数より先は最後の行。見出しの行は本体の最初の行。
        document.apply(Intent::GoTo(GoToTarget::parse("5000").unwrap()), window);
        assert_eq!(document.selection().cursor(), CellPosition::new(98, 1));
        document.apply(Intent::GoTo(GoToTarget::parse("1").unwrap()), window);
        assert_eq!(document.selection().cursor().body(), 0);
    }

    #[test]
    fn going_to_a_column_uses_the_original_column_number_after_reordering() {
        let window = normal();
        let mut document = three();
        // c 列（元の 3 列目）を先頭へ移してから、3 列目へ飛ぶ。
        document.apply(pointing(140, 35, SelectionMode::Collapse), window);
        document.apply(Intent::Release, window);
        document.apply(pointing(140, 35, SelectionMode::Collapse), window);
        document.apply(dragging_to(50, 40), window);
        document.apply(Intent::Release, window);
        document.apply(Intent::GoTo(GoToTarget::parse("2,3").unwrap()), window);
        assert_eq!(document.selection().cursor(), CellPosition::new(0, 0));
    }

    /// 見出しの無い 3 行 × 2 列の表（レコード 0〜2）。
    fn editable() -> Document {
        let rows: Vec<Vec<String>> = vec![
            vec![String::from("0012"), String::from("東京")],
            vec![String::from("b1"), String::from("b2")],
            vec![String::from("c1")],
        ];
        closed(Document::open(
            String::from("e.csv — NeNe Ledger"),
            summary(),
            None,
            cell_rows(&rows),
        ))
    }

    fn typed(document: &mut Document, text: &str) {
        for character in text.chars() {
            document.apply(Intent::Type(character), normal());
        }
    }

    fn edited(document: &Document, record: u64, column: usize) -> Option<&CellValue> {
        document
            .edits()
            .get(RowId::Source(record), ColumnId::Source(column))
    }

    #[test]
    fn typing_on_a_cell_replaces_its_value_and_enter_commits_and_moves_down() {
        let mut document = editable();
        typed(&mut document, "13");
        assert_eq!(
            document.editor().map(CellEditor::mode),
            Some(EditMode::Entering)
        );
        assert!(document.edits().is_empty());
        document.apply(Intent::Confirm, normal());
        assert_eq!(document.editor(), None);
        assert_eq!(
            edited(&document, 0, 0),
            Some(&CellValue::Text(String::from("13")))
        );
        assert_eq!(document.selection().cursor(), CellPosition::new(1, 0));
        // 原本の表示は原本のまま（派生値は描くときに重ねる）。
        assert_eq!(document.view().rows()[0].values().get(0), Some("0012"));
        let frame = document.frame(normal(), palette());
        text_at(&frame, "13");
        text_at(&frame, "● e.csv — NeNe Ledger");
        assert_eq!(
            document.window_name(),
            "● e.csv — NeNe Ledger",
            "OS の窓の名前も同じ（#167）"
        );
    }

    #[test]
    fn escape_cancels_without_an_edit_and_enter_alone_only_moves() {
        let mut document = editable();
        typed(&mut document, "x");
        document.apply(Intent::Cancel, normal());
        assert_eq!(document.editor(), None);
        assert!(document.edits().is_empty());
        document.apply(Intent::Confirm, normal());
        assert_eq!(document.selection().cursor(), CellPosition::new(1, 0));
        document.apply(Intent::Advance(CursorStep::Right), normal());
        assert_eq!(document.selection().cursor(), CellPosition::new(1, 1));
        assert!(document.edits().is_empty());
    }

    #[test]
    fn f2_continues_and_arrows_stay_inside_while_editing() {
        let mut document = editable();
        document.apply(Intent::Edit, normal());
        assert_eq!(
            document.editor().map(CellEditor::mode),
            Some(EditMode::Editing)
        );
        document.apply(moving(CursorStep::RowStart), normal());
        typed(&mut document, "A-");
        document.apply(Intent::Advance(CursorStep::Right), normal());
        assert_eq!(
            edited(&document, 0, 0),
            Some(&CellValue::Text(String::from("A-0012")))
        );
        assert_eq!(document.selection().cursor(), CellPosition::new(0, 1));
    }

    #[test]
    fn a_preedit_starts_entering_and_the_commit_goes_into_the_field() {
        let mut document = editable();
        // 組み立ての始まり（空）では編集に入らない。
        document.compose(Some(Preedit::new("", 0)));
        assert_eq!(document.editor(), None);
        document.compose(Some(Preedit::new("にほ", 2)));
        assert_eq!(
            document.editor().map(CellEditor::mode),
            Some(EditMode::Entering)
        );
        // 確定前の文字列はセルの中に描くが、差分にはまだ入らない。
        assert!(
            document
                .editing_cell()
                .is_some_and(|cell| cell.text() == "にほ"
                    && cell.caret() == 4
                    && cell.underline() == Some((0, 4)))
        );
        assert!(document.edits().is_empty());
        document.commit_text("日本");
        document.compose(None);
        document.apply(Intent::Confirm, normal());
        assert_eq!(
            edited(&document, 0, 0),
            Some(&CellValue::Text(String::from("日本")))
        );
        // 編集していないときの確定だけでも、入力の状態で入る。
        document.commit_text("語");
        document.apply(Intent::Confirm, normal());
        assert_eq!(
            edited(&document, 1, 0),
            Some(&CellValue::Text(String::from("語")))
        );
    }

    #[test]
    fn a_preedit_inside_f2_keeps_the_value_and_esc_drops_both() {
        let mut document = editable();
        document.apply(Intent::Edit, normal());
        document.compose(Some(Preedit::new("か", 1)));
        assert!(
            document
                .editing_cell()
                .is_some_and(|cell| cell.text() == "0012か" && cell.underline() == Some((4, 6)))
        );
        document.compose(None);
        document.apply(Intent::Cancel, normal());
        assert_eq!(document.editor(), None);
        assert!(document.edits().is_empty());
    }

    #[test]
    fn composing_is_ignored_while_saving_or_asking_to_close() {
        let mut document = editable();
        typed(&mut document, "x");
        document.apply(Intent::Confirm, normal());
        document.begin_saving();
        document.compose(Some(Preedit::new("に", 1)));
        assert_eq!(document.editor(), None);
        document.commit_text("に");
        assert_eq!(document.editor(), None);
        assert_eq!(document.notice, Some(Notice::StillSaving));

        let mut asking = editable();
        typed(&mut asking, "x");
        assert!(!asking.ask_close());
        asking.compose(Some(Preedit::new("す", 1)));
        asking.commit_text("す");
        assert_eq!(asking.editor(), None);
        assert_eq!(asking.close_state(), CloseState::Asking);
    }

    #[test]
    fn the_composition_area_is_the_editing_cell() {
        let mut document = editable();
        assert_eq!(document.composition_area(normal()), None);
        document.apply(Intent::Advance(CursorStep::Right), normal());
        document.apply(Intent::Confirm, normal());
        document.compose(Some(Preedit::new("と", 1)));
        let area = document.composition_area(normal()).unwrap();
        assert_eq!(
            document.cell_at(center(area), normal()),
            Some(CellPosition::new(1, 1))
        );
    }

    /// 原本の行 `source`（レコードの順）から、合成ルートと同じく区間ごとに並べて、今の縦の位置から読み直させる。
    fn reread(document: &mut Document, source: &[CellRow]) {
        let top = document.line_of(document.view().first_body());
        let mut rows: Vec<CellRow> = Vec::new();
        for run in document.runs_from(top, 50) {
            match run {
                OrderRun::Source { first, count } => {
                    let start = usize::try_from(first).unwrap();
                    let end = (start + usize::try_from(count.unwrap()).unwrap()).min(source.len());
                    rows.extend(source.get(start..end).unwrap_or_default().iter().cloned());
                }
                OrderRun::Added(added) => rows.extend(added.iter().map(|_| CellRow::empty())),
            }
        }
        document.show(top, rows.clone());
    }

    fn editable_rows() -> Vec<CellRow> {
        table(&[&["0012", "東京"], &["b1", "b2"], &["c1"]])
    }

    fn first_cells(document: &Document) -> Vec<String> {
        document
            .view()
            .rows()
            .iter()
            .map(|row| row.cells().get(0).map(String::from).unwrap_or_default())
            .collect()
    }

    #[test]
    fn rows_are_inserted_above_and_below_and_undo_takes_them_back() {
        let mut document = editable();
        document.report(JobReport::Finished(IndexSummary::new(3, None)));
        document.apply(moving(CursorStep::Down), normal());
        let effect = document.apply(Intent::InsertRows(RowPlace::Above), normal());
        assert!(matches!(effect, DocumentEffect::Read { .. }));
        reread(&mut document, &editable_rows());
        assert_eq!(first_cells(&document), ["0012", "", "b1", "c1"]);
        // カーソルは足した行（本体の 1 番目）。状態の行で知らせる。
        assert_eq!(document.selection().cursor().body(), 1);
        text_at(&document.frame(normal(), palette()), "1 行を足しました");
        // 行全体を 2 行選んで下に足すと 2 行。
        document.apply(moving(CursorStep::Down), normal());
        document.apply(
            Intent::Move {
                step: CursorStep::Down,
                mode: SelectionMode::Extend,
            },
            normal(),
        );
        document.apply(Intent::SelectRows, normal());
        document.apply(Intent::InsertRows(RowPlace::Below), normal());
        reread(&mut document, &editable_rows());
        assert_eq!(first_cells(&document), ["0012", "", "b1", "c1", "", ""]);
        document.apply(Intent::Undo, normal());
        reread(&mut document, &editable_rows());
        assert_eq!(first_cells(&document), ["0012", "", "b1", "c1"]);
        text_at(
            &document.frame(normal(), palette()),
            "元に戻しました（2 行）",
        );
        document.apply(Intent::Undo, normal());
        assert!(document.rows.is_whole());
        document.apply(Intent::Redo, normal());
        reread(&mut document, &editable_rows());
        assert_eq!(first_cells(&document), ["0012", "", "b1", "c1"]);
    }

    #[test]
    fn appending_waits_for_the_index_and_then_adds_at_the_end() {
        let mut document = editable();
        document.apply(Intent::InsertRows(RowPlace::End), normal());
        assert!(document.rows.is_whole());
        text_at(
            &document.frame(normal(), palette()),
            "表の末尾は、行の数が分かってから扱えます（索引を作っています）",
        );
        document.report(JobReport::Finished(IndexSummary::new(3, None)));
        document.apply(Intent::InsertRows(RowPlace::End), normal());
        reread(&mut document, &editable_rows());
        assert_eq!(first_cells(&document), ["0012", "b1", "c1", ""]);
        assert_eq!(document.selection().cursor().body(), 3);
        // 状態の行の行の数も、足した後の数。
        assert!(document.status().starts_with("4 行"));
        // 足した行に書いて保存の計画に渡す差分は、足した行と並び。
        typed(&mut document, "d1");
        document.apply(Intent::Confirm, normal());
        let changes = document.changes();
        assert_eq!(changes.order().len(3), 4);
        assert_eq!(changes.shape().columns(), 2);
    }

    #[test]
    fn removing_rows_keeps_their_edits_for_undo_and_saving_drops_the_history() {
        let mut document = editable();
        document.report(JobReport::Finished(IndexSummary::new(3, None)));
        document.apply(moving(CursorStep::Down), normal());
        typed(&mut document, "B");
        document.apply(Intent::Cancel, normal());
        document.apply(Intent::Edit, normal());
        typed(&mut document, "!");
        document.apply(Intent::Advance(CursorStep::Left), normal());
        document.apply(Intent::RemoveRows, normal());
        reread(&mut document, &editable_rows());
        assert_eq!(first_cells(&document), ["0012", "c1"]);
        text_at(
            &document.frame(normal(), palette()),
            "1 行を消しました（Ctrl+Z で戻す）",
        );
        // 消した行の書き換えは残り、戻せば見える。
        assert_eq!(
            edited(&document, 1, 0),
            Some(&CellValue::Text(String::from("b1!")))
        );
        document.apply(Intent::Undo, normal());
        reread(&mut document, &editable_rows());
        assert_eq!(first_cells(&document), ["0012", "b1", "c1"]);
        text_at(&document.frame(normal(), palette()), "b1!");
        // 行を足し消しした手があると、保存の後は履歴を残さない（原本のレコードの番号が変わる）。
        document.apply(Intent::Redo, normal());
        document.begin_saving();
        document.saved();
        assert!(document.rows.is_whole());
        document.apply(Intent::Undo, normal());
        text_at(
            &document.frame(normal(), palette()),
            "元に戻すものがありません",
        );
    }

    #[test]
    fn removing_every_row_leaves_an_empty_table() {
        let mut document = editable();
        document.report(JobReport::Finished(IndexSummary::new(3, None)));
        document.apply(Intent::SelectAll, normal());
        document.apply(Intent::RemoveRows, normal());
        reread(&mut document, &editable_rows());
        assert!(document.view().rows().is_empty());
        text_at(
            &document.frame(normal(), palette()),
            "3 行を消しました（Ctrl+Z で戻す）",
        );
        document.apply(Intent::Undo, normal());
        reread(&mut document, &editable_rows());
        assert_eq!(first_cells(&document), ["0012", "b1", "c1"]);
    }

    fn three_rows() -> Vec<CellRow> {
        table(&[&["a", "b", "c"], &["1", "2", "3"], &["4"]])
    }

    fn header_cells(document: &Document) -> Vec<String> {
        document
            .header()
            .first()
            .map(|row| row.cells().to_strings())
            .unwrap_or_default()
    }

    #[test]
    fn columns_are_inserted_left_and_right_and_undo_takes_them_back() {
        let mut document = three();
        document.report(JobReport::Finished(IndexSummary::new(3, None)));
        // カーソルは「b」の列。左に 1 列足す。
        document.apply(moving(CursorStep::Right), normal());
        let effect = document.apply(Intent::InsertColumns(ColumnPlace::Left), normal());
        assert!(matches!(effect, DocumentEffect::Read { .. }));
        reread(&mut document, &three_rows());
        assert_eq!(header_cells(&document), ["a", "", "b", "c"]);
        assert_eq!(
            document.view().rows()[0].cells().to_strings(),
            ["1", "", "2", "3"]
        );
        // 短い行は「セルが無い」のまま。足した列までは空のセル。
        assert_eq!(document.view().rows()[1].cells().to_strings(), ["4", ""]);
        assert_eq!(document.widths.columns().len(), 4);
        assert_eq!(document.selection().cursor().column(), 1);
        text_at(&document.frame(normal(), palette()), "1 列を足しました");
        // 足した列のセルに書く。差分は足した列の同一性に付く。
        typed(&mut document, "x");
        document.apply(Intent::Confirm, normal());
        assert_eq!(
            document.edits().get(RowId::Source(1), ColumnId::Added(0)),
            Some(&CellValue::Text(String::from("x")))
        );
        text_at(&document.frame(normal(), palette()), "x");
    }

    #[test]
    fn inserted_columns_are_undone_and_redone_back_to_the_opened_table() {
        let mut document = three();
        document.report(JobReport::Finished(IndexSummary::new(3, None)));
        document.apply(moving(CursorStep::Right), normal());
        document.apply(Intent::InsertColumns(ColumnPlace::Left), normal());
        typed(&mut document, "x");
        document.apply(Intent::Confirm, normal());
        document.apply(moving(CursorStep::Up), normal());
        // 右に足す。
        document.apply(Intent::InsertColumns(ColumnPlace::Right), normal());
        reread(&mut document, &three_rows());
        assert_eq!(header_cells(&document), ["a", "", "", "b", "c"]);
        document.apply(Intent::Undo, normal());
        reread(&mut document, &three_rows());
        assert_eq!(header_cells(&document), ["a", "", "b", "c"]);
        text_at(
            &document.frame(normal(), palette()),
            "元に戻しました（1 列）",
        );
        // 書いたセルと最初に足した列を戻すと、開いた直後の形。
        document.apply(Intent::Undo, normal());
        document.apply(Intent::Undo, normal());
        reread(&mut document, &three_rows());
        assert!(document.columns.is_whole());
        assert_eq!(header_cells(&document), ["a", "b", "c"]);
        assert_eq!(document.widths.columns().len(), 3);
    }

    #[test]
    fn columns_are_removed_with_their_widths_and_one_column_is_kept() {
        let mut document = three();
        document.report(JobReport::Finished(IndexSummary::new(3, None)));
        document.apply(Intent::RemoveColumns, normal());
        reread(&mut document, &three_rows());
        assert_eq!(header_cells(&document), ["b", "c"]);
        assert_eq!(document.view().rows()[0].cells().to_strings(), ["2", "3"]);
        // 「4」だけの行は、最初の列を消すとセルが無い。
        assert!(document.view().rows()[1].cells().is_empty());
        text_at(
            &document.frame(normal(), palette()),
            "1 列を消しました（Ctrl+Z で戻す）",
        );
        // コピーも並べ直した値。
        document.apply(Intent::SelectAll, normal());
        let rows = three_rows();
        let copied = document
            .copy_text(0, &rows, &[])
            .map(|text| String::from(text.as_str()));
        assert_eq!(copied, Some(String::from("2\t3\r\n\t\r\n")));
        // すべての列は消せない（すべてを選ぶと列全体の選択）。
        document.apply(Intent::SelectAll, normal());
        document.apply(Intent::RemoveColumns, normal());
        text_at(
            &document.frame(normal(), palette()),
            "すべての列は消せません",
        );
        document.apply(Intent::Undo, normal());
        reread(&mut document, &three_rows());
        assert_eq!(header_cells(&document), ["a", "b", "c"]);
        assert_eq!(document.widths.columns().len(), 3);
    }

    #[test]
    fn control_plus_and_minus_act_on_columns_only_when_whole_columns_are_chosen() {
        let mut document = three();
        document.report(JobReport::Finished(IndexSummary::new(3, None)));
        // セルだけを選んでいれば行（カーソルの行の上に足す・その行を消す）。
        document.apply(Intent::InsertHere, normal());
        assert!(!document.rows.is_whole());
        assert!(document.columns.is_whole());
        document.apply(Intent::RemoveHere, normal());
        assert!(document.rows.is_whole());
        // 列全体を選んでいれば列。
        document.apply(Intent::SelectColumns, normal());
        document.apply(Intent::InsertHere, normal());
        assert!(!document.columns.is_whole());
        document.apply(Intent::SelectColumns, normal());
        document.apply(Intent::RemoveHere, normal());
        assert!(document.columns.is_whole());
        // すべてを選んでいれば行。
        document.apply(Intent::SelectAll, normal());
        document.apply(Intent::RemoveHere, normal());
        text_at(
            &document.frame(normal(), palette()),
            "2 行を消しました（Ctrl+Z で戻す）",
        );
    }

    #[test]
    fn saving_after_column_changes_hands_over_the_order_and_drops_the_history() {
        let mut document = three();
        document.report(JobReport::Finished(IndexSummary::new(3, None)));
        document.apply(Intent::InsertColumns(ColumnPlace::Right), normal());
        let changes = document.changes();
        assert_eq!(changes.shape().columns(), 4);
        assert_eq!(
            changes.columns().columns(4),
            [
                ColumnId::Source(0),
                ColumnId::Added(0),
                ColumnId::Source(1),
                ColumnId::Source(2)
            ]
        );
        document.begin_saving();
        document.saved();
        assert!(document.columns.is_whole());
        // 保存したファイルの見出しは、今の列の順。
        assert_eq!(
            document.header_source,
            vec![table(&[&["a", "", "b", "c"]]).remove(0).to_strings()]
        );
        document.apply(Intent::Undo, normal());
        text_at(
            &document.frame(normal(), palette()),
            "元に戻すものがありません",
        );
    }

    #[test]
    fn rows_follow_the_row_order_and_an_added_row_can_be_written() {
        let mut document = editable();
        // 2 行目の前に 1 行足す（画面の操作は次の Issue。ここでは並びを直に変える）。
        let added = document.rows.insert(1, 1);
        assert_eq!(
            document.runs_from(0, 4),
            vec![
                OrderRun::Source {
                    first: 0,
                    count: Some(1)
                },
                OrderRun::Added(vec![0]),
                OrderRun::Source {
                    first: 1,
                    count: Some(2)
                },
            ]
        );
        // 合成ルートは区間ごとに読み、足した行はセルの無い行で渡す。
        let rows = vec![
            vec![String::from("0012"), String::from("東京")],
            Vec::new(),
            vec![String::from("b1"), String::from("b2")],
        ];
        document.show(0, cell_rows(&rows));
        let ids: Vec<RowId> = document.view().rows().iter().map(ViewRow::row).collect();
        assert_eq!(ids, vec![RowId::Source(0), added[0], RowId::Source(1)]);
        // 足した行は表の幅まで空のセル。番号は上から通し番号（ADR 0027 の決定 1）。
        assert_eq!(
            document.view().rows()[1].cells().to_strings(),
            &[String::new(), String::new()]
        );
        assert_eq!(document.view().rows()[2].line(), 2);
        document.apply(moving(CursorStep::Down), normal());
        typed(&mut document, "new");
        document.apply(Intent::Confirm, normal());
        assert_eq!(
            document.edits().get(added[0], ColumnId::Source(0)),
            Some(&CellValue::Text(String::from("new")))
        );
        // 2 行目の「b1」は原本のレコード 1 のまま（位置は 2）。
        document.apply(Intent::Edit, normal());
        typed(&mut document, "!");
        document.apply(Intent::Confirm, normal());
        assert_eq!(
            edited(&document, 1, 0),
            Some(&CellValue::Text(String::from("b1!")))
        );
        // 2 回の確定でカーソルは 4 行目（原本のレコード 2）へ。
        assert!(document.status().contains("4 行目"));
    }

    #[test]
    fn arrows_commit_and_move_while_entering() {
        let mut document = editable();
        typed(&mut document, "z");
        document.apply(moving(CursorStep::Right), normal());
        assert_eq!(document.editor(), None);
        assert_eq!(
            edited(&document, 0, 0),
            Some(&CellValue::Text(String::from("z")))
        );
        assert_eq!(document.selection().cursor(), CellPosition::new(0, 1));
    }

    #[test]
    fn typing_the_source_value_back_removes_the_edit() {
        let mut document = editable();
        typed(&mut document, "x");
        document.apply(Intent::Confirm, normal());
        document.apply(moving(CursorStep::Up), normal());
        typed(&mut document, "0012");
        document.apply(Intent::Confirm, normal());
        assert!(document.edits().is_empty());
        let frame = document.frame(normal(), palette());
        text_at(&frame, "e.csv — NeNe Ledger");
    }

    #[test]
    fn delete_empties_the_cursor_cell_and_alt_enter_breaks_the_line() {
        let mut document = editable();
        document.apply(Intent::Delete, normal());
        assert_eq!(
            edited(&document, 0, 0),
            Some(&CellValue::Text(String::new()))
        );
        document.apply(moving(CursorStep::Right), normal());
        document.apply(Intent::Edit, normal());
        document.apply(Intent::BreakLine, normal());
        typed(&mut document, "区");
        document.apply(Intent::Confirm, normal());
        assert_eq!(
            edited(&document, 0, 1),
            Some(&CellValue::Text(String::from("東京\n区")))
        );
    }

    #[test]
    fn a_missing_cell_is_written_and_a_cut_record_is_not_edited() {
        let mut document = editable();
        document.apply(
            GoToTarget::parse("3,2").map_or(Intent::Cancel, Intent::GoTo),
            normal(),
        );
        assert_eq!(document.selection().cursor(), CellPosition::new(2, 1));
        // 空のまま確定しても差分にしない（ファイルの値と同じ）。
        document.apply(Intent::Edit, normal());
        assert_eq!(document.editor().map(CellEditor::shown), Some(""));
        document.apply(Intent::Cancel, normal());
        typed(&mut document, "");
        document.apply(Intent::Delete, normal());
        assert!(document.edits().is_empty());
        // 原本の行に無いセルにも書ける（GitHub Issue #83）。表・コピーに重なる。
        typed(&mut document, "c2");
        document.apply(Intent::Confirm, normal());
        assert_eq!(
            edited(&document, 2, 1),
            Some(&CellValue::Text(String::from("c2")))
        );
        text_at(&document.frame(normal(), palette()), "c2");
        document.apply(
            GoToTarget::parse("3,1").map_or(Intent::Cancel, Intent::GoTo),
            normal(),
        );
        document.apply(Intent::SelectRows, normal());
        let rows = vec![vec![String::from("c1")]];
        assert_eq!(
            document
                .copy_text(2, &cell_rows(&rows), &[])
                .map(|text| String::from(text.as_str())),
            Some(String::from("c1\tc2\r\n"))
        );
        document.mark_cut(vec![CutRecord::new(1, None)]);
        document.apply(moving(CursorStep::Up), normal());
        document.apply(Intent::Edit, normal());
        assert_eq!(document.editor(), None);
        assert_eq!(edited(&document, 1, 0), None);
    }

    #[test]
    fn copying_takes_the_edited_values() {
        let mut document = editable();
        typed(&mut document, "新");
        document.apply(Intent::Confirm, normal());
        document.apply(moving(CursorStep::Up), normal());
        let rows: Vec<Vec<String>> = vec![vec![String::from("0012"), String::from("東京")]];
        let copied = copy_from(&mut document, &cell_rows(&rows));
        assert_eq!(copied.as_ref().map(ClipboardText::as_str), Some("新\r\n"));
    }

    #[test]
    fn the_editing_cell_draws_its_field_and_a_caret() {
        let mut document = editable();
        typed(&mut document, "ab");
        let frame = document.frame(normal(), palette());
        text_at(&frame, "ab");
        assert!(
            document
                .editing_cell()
                .is_some_and(|cell| cell.caret() == 2)
        );
        document.apply(Intent::Edit, normal());
        document.apply(moving(CursorStep::Left), normal());
        assert!(
            document
                .editing_cell()
                .is_some_and(|cell| cell.caret() == 1)
        );
    }

    #[test]
    fn saving_commits_the_open_edit_and_asks_the_composition_root_to_save() {
        let mut document = editable();
        assert_eq!(document.apply(Intent::Save, normal()), DocumentEffect::Stay);
        let frame = document.frame(normal(), palette());
        text_at(&frame, "変えた所が無いので保存しませんでした");
        typed(&mut document, "x");
        assert_eq!(document.apply(Intent::Save, normal()), DocumentEffect::Save);
        assert_eq!(document.editor(), None);
        assert_eq!(
            edited(&document, 0, 0),
            Some(&CellValue::Text(String::from("x")))
        );
    }

    #[test]
    fn while_saving_edits_are_refused_and_the_status_says_so() {
        let mut document = editable();
        typed(&mut document, "x");
        document.apply(Intent::Save, normal());
        document.begin_saving();
        assert!(document.is_saving());
        text_at(
            &document.frame(normal(), palette()),
            "保存しています…（Esc でやめる）",
        );
        typed(&mut document, "y");
        assert_eq!(document.editor(), None);
        text_at(
            &document.frame(normal(), palette()),
            "保存しています。終わるまで直せません",
        );
        // 動くことはできる。
        document.apply(moving(CursorStep::Down), normal());
        assert_eq!(document.selection().cursor(), CellPosition::new(1, 0));
    }

    #[test]
    fn a_finished_save_clears_the_edits_and_rereads_the_same_place() {
        let mut document = editable();
        typed(&mut document, "x");
        document.apply(Intent::Save, normal());
        document.begin_saving();
        assert_eq!(document.saved(), DocumentEffect::Read { first_body: 0 });
        assert!(!document.is_saving());
        assert!(document.edits().is_empty());
        assert_eq!(document.index(), JobState::Waiting);
        let frame = document.frame(normal(), palette());
        text_at(&frame, "保存しました");
        text_at(&frame, "e.csv — NeNe Ledger");
    }

    #[test]
    fn a_failed_save_keeps_the_edits_and_says_why() {
        let mut document = editable();
        typed(&mut document, "x");
        document.apply(Intent::Save, normal());
        document.begin_saving();
        document.save_failed(SaveProblem::Unrepresentable('😀'));
        assert!(!document.is_saving());
        assert!(!document.edits().is_empty());
        let frame = document.frame(normal(), palette());
        text_at(
            &frame,
            "「😀」はファイルの文字コードで表せないので保存できませんでした（直した値は残っています）",
        );
        text_at(&frame, "● e.csv — NeNe Ledger");
    }

    #[test]
    fn closing_without_edits_closes_at_once() {
        let mut document = editable();
        assert!(document.ask_close());
        assert_eq!(document.close_state(), CloseState::Finished);
    }

    #[test]
    fn closing_with_edits_asks_and_escape_keeps_the_window() {
        let mut document = editable();
        typed(&mut document, "x");
        // 編集中の値も、閉じる前に確定する。
        assert!(!document.ask_close());
        assert_eq!(document.close_state(), CloseState::Asking);
        assert!(!document.edits().is_empty());
        let frame = document.frame(normal(), palette());
        text_at(&frame, "保存しないで閉じる (D)");
        // 尋ねている間は表を動かさない。
        document.apply(moving(CursorStep::Down), normal());
        assert_eq!(document.selection().cursor(), CellPosition::new(0, 0));
        document.apply(Intent::Cancel, normal());
        assert_eq!(document.close_state(), CloseState::Open);
        assert!(!document.edits().is_empty());
    }

    #[test]
    fn choosing_discard_finishes_and_choosing_save_waits_for_the_save() {
        let mut discarding = editable();
        typed(&mut discarding, "x");
        discarding.ask_close();
        discarding.apply(Intent::Type('d'), normal());
        assert_eq!(discarding.close_state(), CloseState::Finished);

        let mut saving = editable();
        typed(&mut saving, "x");
        saving.ask_close();
        assert_eq!(
            saving.apply(Intent::Confirm, normal()),
            DocumentEffect::Save
        );
        assert_eq!(saving.close_state(), CloseState::AfterSave);
        saving.begin_saving();
        text_at(&saving.frame(normal(), palette()), "保存してから閉じます…");
        saving.saved();
        assert_eq!(saving.close_state(), CloseState::Finished);
    }

    #[test]
    fn a_failed_save_does_not_close() {
        let mut document = editable();
        typed(&mut document, "x");
        document.ask_close();
        assert_eq!(
            document.apply(Intent::Type('S'), normal()),
            DocumentEffect::Save
        );
        document.begin_saving();
        document.save_failed(SaveProblem::Locked);
        assert_eq!(document.close_state(), CloseState::Open);
        assert!(!document.edits().is_empty());
    }

    #[test]
    fn the_buttons_of_the_prompt_can_be_pressed() {
        let mut document = editable();
        typed(&mut document, "x");
        document.ask_close();
        let window = normal();
        let (_, discard) = ClosePrompt::new(window).buttons()[1];
        let at = PixelPoint::new(
            discard.origin().x() + discard.size().width() / 2,
            discard.origin().y() + discard.size().height() / 2,
        );
        document.apply(
            Intent::Point {
                at,
                mode: SelectionMode::Collapse,
            },
            window,
        );
        assert_eq!(document.close_state(), CloseState::Finished);
    }

    #[test]
    fn closing_while_saving_asks_and_save_closes_after_it() {
        let mut document = editable();
        typed(&mut document, "x");
        document.apply(Intent::Save, normal());
        document.begin_saving();
        assert!(!document.ask_close());
        assert_eq!(
            document.apply(Intent::Confirm, normal()),
            DocumentEffect::Stay
        );
        assert_eq!(document.close_state(), CloseState::AfterSave);
        document.saved();
        assert_eq!(document.close_state(), CloseState::Finished);
    }

    #[test]
    fn undo_and_redo_move_to_the_cell_and_back_to_the_file_value_removes_the_edit() {
        let mut document = editable();
        typed(&mut document, "x");
        document.apply(Intent::Advance(CursorStep::Right), normal());
        typed(&mut document, "y");
        document.apply(Intent::Confirm, normal());
        assert_eq!(document.selection().cursor(), CellPosition::new(1, 1));
        document.apply(Intent::Undo, normal());
        assert_eq!(document.selection().cursor(), CellPosition::new(0, 1));
        assert_eq!(edited(&document, 0, 1), None);
        text_at(
            &document.frame(normal(), palette()),
            "元に戻しました（1 セル）",
        );
        document.apply(Intent::Undo, normal());
        assert!(document.edits().is_empty());
        assert_eq!(document.selection().cursor(), CellPosition::new(0, 0));
        document.apply(Intent::Undo, normal());
        text_at(
            &document.frame(normal(), palette()),
            "元に戻すものがありません",
        );
        document.apply(Intent::Redo, normal());
        document.apply(Intent::Redo, normal());
        assert_eq!(
            edited(&document, 0, 1),
            Some(&CellValue::Text(String::from("y")))
        );
    }

    #[test]
    fn undo_while_editing_only_drops_the_edit() {
        let mut document = editable();
        typed(&mut document, "x");
        document.apply(Intent::Confirm, normal());
        typed(&mut document, "z");
        document.apply(Intent::Undo, normal());
        assert_eq!(document.editor(), None);
        assert_eq!(
            edited(&document, 0, 0),
            Some(&CellValue::Text(String::from("x")))
        );
    }

    #[test]
    fn undo_still_works_after_saving() {
        let mut document = editable();
        typed(&mut document, "x");
        document.apply(Intent::Save, normal());
        document.begin_saving();
        document.saved();
        assert!(document.edits().is_empty());
        // 開き直した後の表示（保存した値）。
        document.show(0, table(&[&["x", "東京"], &["b1", "b2"]]));
        document.apply(Intent::Undo, normal());
        assert_eq!(
            edited(&document, 0, 0),
            Some(&CellValue::Text(String::from("0012")))
        );
        document.apply(Intent::Redo, normal());
        assert!(document.edits().is_empty());
    }

    /// 行か列を足し消ししただけ（セルは書き換えていない）でも未保存。保存・閉じるときの確認・題名の印が効き、
    /// 戻せば未保存でなくなる（GitHub Issue #107。直す前は Ctrl+S が「保存しませんでした」で、尋ねずに閉じた）。
    #[test]
    fn adding_or_removing_rows_and_columns_alone_is_unsaved() {
        for intent in [
            Intent::InsertRows(RowPlace::Above),
            Intent::RemoveRows,
            Intent::InsertColumns(ColumnPlace::Left),
            Intent::RemoveColumns,
        ] {
            let mut document = editable();
            assert!(!document.has_unsaved());
            document.apply(intent, normal());
            assert!(document.edits().is_empty(), "{intent:?}");
            assert!(document.has_unsaved(), "{intent:?}");
            text_at(
                &document.frame(normal(), palette()),
                "● e.csv — NeNe Ledger",
            );
            let mut closing = document.clone();
            assert!(!closing.ask_close(), "{intent:?}");
            assert_eq!(closing.close_state(), CloseState::Asking);
            let mut saving = document.clone();
            assert_eq!(
                saving.apply(Intent::Save, normal()),
                DocumentEffect::Save,
                "{intent:?}"
            );
            document.apply(Intent::Undo, normal());
            assert!(!document.has_unsaved(), "{intent:?}");
            assert!(document.ask_close(), "{intent:?}");
        }
    }

    /// 新しい表は 1 行 × 3 列・見出し無し。何も打たなければ未保存でなく、Undo で消えない。打てば未保存（ADR 0028 の決定 2）。
    #[test]
    fn a_blank_table_opens_with_one_row_of_three_empty_cells() {
        let mut document = closed(Document::blank(
            String::from("無題 — NeNe Ledger"),
            summary(),
        ));
        reread(&mut document, &[]);
        assert!(document.header().is_empty());
        assert_eq!(document.view().rows().len(), 1);
        assert_eq!(document.view().rows()[0].cells().to_strings(), ["", "", ""]);
        assert_eq!(document.widths().columns().len(), 3);
        assert!(!document.has_unsaved());
        text_at(&document.frame(normal(), palette()), "無題 — NeNe Ledger");
        document.apply(Intent::Undo, normal());
        reread(&mut document, &[]);
        assert_eq!(document.view().rows().len(), 1);
        assert!(document.ask_close());
    }

    /// 新しい表に打てば未保存。列を足しても未保存で、戻せば開いた直後（ADR 0028 の決定 2）。
    #[test]
    fn a_blank_table_becomes_unsaved_only_when_changed() {
        let mut typing = closed(Document::blank(String::from("無題"), summary()));
        reread(&mut typing, &[]);
        typed(&mut typing, "x");
        typing.apply(Intent::Confirm, normal());
        assert!(typing.has_unsaved());
        assert!(!typing.ask_close());
        // 列を足せば未保存。戻せば開いた直後。
        let mut wider = closed(Document::blank(String::from("無題"), summary()));
        wider.apply(Intent::InsertColumns(ColumnPlace::Right), normal());
        assert!(wider.has_unsaved());
        wider.apply(Intent::Undo, normal());
        assert!(!wider.has_unsaved());
    }

    /// 新しい表の Ctrl+S と、どの表でも Ctrl+Shift+S は名前を付けて保存。保存して閉じるを選んで保存先を選ばずにやめれば
    /// 閉じない。名前を付けて保存した後は題名が替わり、Ctrl+S は普通の保存（ADR 0028 の決定 4・5）。
    #[test]
    fn untitled_tables_save_as_and_take_the_new_name() {
        let mut document = closed(Document::blank(String::from("無題"), summary()));
        reread(&mut document, &[]);
        assert!(document.is_untitled());
        assert_eq!(
            document.apply(Intent::Save, normal()),
            DocumentEffect::SaveAs
        );
        typed(&mut document, "x");
        assert!(!document.ask_close());
        let effect = document.apply(Intent::Type('s'), normal());
        assert_eq!(effect, DocumentEffect::SaveAs);
        assert_eq!(document.close_state(), CloseState::AfterSave);
        document.save_as_cancelled();
        assert_eq!(document.close_state(), CloseState::Open);
    }

    /// 名前を付けて保存した後は題名と改行の内訳が替わり、Ctrl+S は普通の保存（ADR 0028 の決定 5）。
    #[test]
    fn saving_as_takes_the_new_name_and_line_endings() {
        let mut document = closed(Document::blank(String::from("無題"), summary()));
        reread(&mut document, &[]);
        typed(&mut document, "x");
        assert_eq!(
            document.apply(Intent::SaveAs, normal()),
            DocumentEffect::SaveAs
        );
        document.begin_saving();
        let crlf = LineEndingCounts::tally([LineEnding::CrLf].into_iter());
        let _reread = document.saved_as(String::from("a.csv — NeNe Ledger"), crlf);
        assert_eq!(
            document.format_summary().line_endings().dominant(),
            Some(LineEnding::CrLf)
        );
        assert!(!document.is_untitled());
        assert!(!document.has_unsaved());
        assert_eq!(document.title(), "a.csv — NeNe Ledger");
        typed(&mut document, "y");
        assert_eq!(document.apply(Intent::Save, normal()), DocumentEffect::Save);
        // ファイルのある表でも Ctrl+Shift+S は名前を付けて保存（変えていなくても写しを書ける）。
        let mut opened = editable();
        assert_eq!(
            opened.apply(Intent::SaveAs, normal()),
            DocumentEffect::SaveAs
        );
    }

    /// 1 つのセルの貼り付けは、カーソルのセルの値になり（1 手として戻せる）、編集中は欄に挿す。値は変えない（CLIP-01）。
    #[test]
    fn pasting_one_cell_sets_the_cursor_cell_or_goes_into_the_field() {
        let mut document = editable();
        pasted(&mut document, "0099\r\n");
        assert_eq!(
            edited(&document, 0, 0),
            Some(&CellValue::Text(String::from("0099")))
        );
        assert_eq!(document.editor(), None);
        document.apply(Intent::Undo, normal());
        assert_eq!(edited(&document, 0, 0), None);
        // 引用した 1 つのセル（中に改行）も 1 つの値。
        pasted(&mut document, "\"a\nb\"\r\n");
        assert_eq!(
            edited(&document, 0, 0),
            Some(&CellValue::Text(String::from("a\nb")))
        );
        // 編集中は欄のカーソルに挿す。
        let mut editing = editable();
        editing.apply(Intent::Edit, normal());
        assert_eq!(editing.paste("-1\r\n"), PasteRequest::Done);
        assert!(
            editing
                .editor()
                .is_some_and(|editor| editor.field().text() == "0012-1")
        );
    }

    /// 貼り付けの 2 段を、表 `editable_rows` の原本から行を読んで済ませる（合成ルートの代わり）。
    fn pasted(document: &mut Document, text: &str) -> DocumentEffect {
        match document.paste(text) {
            PasteRequest::Done => DocumentEffect::Stay,
            PasteRequest::Read { first_line, count } => {
                let rows: Vec<CellRow> = editable_rows()
                    .into_iter()
                    .skip(usize::try_from(first_line).unwrap())
                    .take(usize::try_from(count).unwrap())
                    .collect();
                document.paste_rows(first_line, (&rows, &[]), normal())
            }
        }
    }

    fn indexed_editable() -> Document {
        let mut document = editable();
        document.report(JobReport::Finished(IndexSummary::new(3, None)));
        document
    }

    fn text_of(document: &Document, row: RowId, column: ColumnId) -> Option<&str> {
        match document.edits().get(row, column) {
            Some(CellValue::Text(value)) => Some(value),
            Some(CellValue::Absent) | None => None,
        }
    }

    /// 複数のセルは、選んだ所の左上から束のまま書き（列の数が足りない行にも書く）、貼った範囲を選ぶ。1 手で戻り、やり直せる
    /// （ADR 0029 の決定 1・3）。束の短い行は、その先のセルを書かない。
    #[test]
    fn pasting_many_cells_writes_from_the_top_left_as_one_step() {
        let mut document = indexed_editable();
        document.apply(moving(CursorStep::Down), normal());
        pasted(&mut document, "x\ty\r\nz\r\n");
        let cell = |row: u64, column: usize| (RowId::Source(row), ColumnId::Source(column));
        let (b1, b2, c1, c2) = (cell(1, 0), cell(1, 1), cell(2, 0), cell(2, 1));
        assert_eq!(text_of(&document, b1.0, b1.1), Some("x"));
        assert_eq!(text_of(&document, b2.0, b2.1), Some("y"));
        assert_eq!(text_of(&document, c1.0, c1.1), Some("z"));
        assert_eq!(text_of(&document, c2.0, c2.1), None);
        assert_eq!(document.selection().cursor(), CellPosition::new(1, 0));
        assert_eq!(document.selection().bottom_right(), CellPosition::new(2, 1));
        text_at(
            &document.frame(normal(), palette()),
            "2 行 × 2 列を貼りました",
        );
        document.apply(Intent::Undo, normal());
        assert!(document.edits().is_empty());
        assert!(!document.has_unsaved());
        text_at(
            &document.frame(normal(), palette()),
            "元に戻しました（3 セル）",
        );
        document.apply(Intent::Redo, normal());
        assert_eq!(text_of(&document, c1.0, c1.1), Some("z"));
    }

    /// 貼った値は変えない（先頭の 0・日付に見える値・指数表記・KEEP-01）。
    #[test]
    fn pasted_values_are_kept_as_text() {
        let mut document = indexed_editable();
        pasted(&mut document, "007\t2026/1/2\r\n1E5\r\n");
        let value = |row: u64, column: usize| {
            text_of(&document, RowId::Source(row), ColumnId::Source(column)).map(String::from)
        };
        assert_eq!(
            [value(0, 0), value(0, 1), value(1, 0)],
            [
                Some(String::from("007")),
                Some(String::from("2026/1/2")),
                Some(String::from("1E5"))
            ]
        );
    }

    /// 表の右下のセルから 2 行 × 2 列を貼った表（1 行と 1 列が足りない）。
    fn pasted_past_the_edges() -> Document {
        let mut document = indexed_editable();
        document.apply(moving(CursorStep::Down), normal());
        document.apply(moving(CursorStep::Down), normal());
        document.apply(moving(CursorStep::Right), normal());
        pasted(&mut document, "p\tq\r\nr\ts\r\n");
        document
    }

    /// 束が表の下と右の端を越えれば、足りない行と列を足して書き、貼った範囲を選ぶ（ADR 0029 の決定 2）。
    #[test]
    fn pasting_past_the_edges_adds_rows_and_columns() {
        let document = pasted_past_the_edges();
        assert_eq!(document.widths().columns().len(), 3);
        text_at(
            &document.frame(normal(), palette()),
            "2 行 × 2 列を貼りました（1 行と 1 列を足しました）",
        );
        let changes = document.changes();
        let added_row = changes.order().row_at(3).unwrap();
        let added_column = changes.columns().column_at(2).unwrap();
        assert!(matches!(
            (added_row, added_column),
            (RowId::Added(_), ColumnId::Added(_))
        ));
        let values = [
            text_of(&document, RowId::Source(2), ColumnId::Source(1)),
            text_of(&document, RowId::Source(2), added_column),
            text_of(&document, added_row, ColumnId::Source(1)),
            text_of(&document, added_row, added_column),
        ];
        assert_eq!(values, [Some("p"), Some("q"), Some("r"), Some("s")]);
        assert_eq!(
            (
                document.selection().cursor(),
                document.selection().bottom_right()
            ),
            (CellPosition::new(2, 1), CellPosition::new(3, 2))
        );
    }

    /// 端を越えた貼り付けは、Ctrl+Z 1 回で書き換えも足した行と列も戻り、Ctrl+Y でまた全部になる（ADR 0029 の決定 3）。
    #[test]
    fn one_undo_takes_back_a_paste_with_its_added_rows_and_columns() {
        let mut document = pasted_past_the_edges();
        assert!(document.has_unsaved());
        document.apply(Intent::Undo, normal());
        assert!(document.edits().is_empty());
        assert_eq!(document.widths().columns().len(), 2);
        assert!(!document.has_unsaved());
        // カーソルは貼った左上（消えた行と列ではない）。
        assert_eq!(document.selection().cursor(), CellPosition::new(2, 1));
        document.apply(Intent::Redo, normal());
        assert_eq!(document.widths().columns().len(), 3);
        let changes = document.changes();
        let corner = (
            changes.order().row_at(3).unwrap(),
            changes.columns().column_at(2).unwrap(),
        );
        assert_eq!(text_of(&document, corner.0, corner.1), Some("s"));
    }

    /// Ctrl+Enter の一括の入力を、貼り付けと同じく読ませた行に書く。
    fn filled(document: &mut Document) -> DocumentEffect {
        let effect = document.apply(Intent::FillSelection, normal());
        if let DocumentEffect::ReadPasted { first_line, count } = effect {
            let rows: Vec<CellRow> = editable_rows()
                .into_iter()
                .skip(usize::try_from(first_line).unwrap_or(usize::MAX))
                .take(usize::try_from(count).unwrap_or(0))
                .collect();
            return document.paste_rows(first_line, (&rows, &[]), normal());
        }
        effect
    }

    /// 範囲を選んで打ち始め、Ctrl+Enter で範囲のすべてのセルに同じ値を書く。改行を含む欄も 1 つの値のまま（分けない）。
    /// 1 手で戻る（EDIT-02・GitHub Issue #203・ADR 0029 の決定 1 と同じ道）。
    #[test]
    fn ctrl_enter_fills_the_selected_range_with_the_typed_value() {
        let mut document = indexed_editable();
        for step in [CursorStep::Down, CursorStep::Right] {
            document.apply(
                Intent::Move {
                    step,
                    mode: SelectionMode::Extend,
                },
                normal(),
            );
        }
        for intent in [Intent::Type('a'), Intent::BreakLine, Intent::Type('b')] {
            document.apply(intent, normal());
        }
        filled(&mut document);
        for (row, column) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
            assert_eq!(
                text_of(&document, RowId::Source(row), ColumnId::Source(column)),
                Some("a\nb"),
                "{row} {column}"
            );
        }
        assert_eq!(
            text_of(&document, RowId::Source(2), ColumnId::Source(0)),
            None
        );
        assert!(document.editing_cell().is_none(), "欄は閉じる");
        document.apply(Intent::Undo, normal());
        assert!(document.edits().is_empty(), "1 手で戻る");
    }

    /// 1 つのセルなら動かずに確定するだけ。編集していなければ何もしない。
    #[test]
    fn ctrl_enter_on_one_cell_commits_in_place_and_does_nothing_without_editing() {
        let mut document = indexed_editable();
        assert_eq!(filled(&mut document), DocumentEffect::Stay);
        assert!(document.edits().is_empty());
        document.apply(Intent::Type('q'), normal());
        assert_eq!(filled(&mut document), DocumentEffect::Stay);
        assert_eq!(
            text_of(&document, RowId::Source(0), ColumnId::Source(0)),
            Some("q")
        );
        assert_eq!(
            document.selection().cursor(),
            CellPosition::new(0, 0),
            "動かない"
        );
    }

    /// 書けなかったら（読めなかった）欄を戻して、打った字を失わない。書く範囲はカーソルのある今の範囲（離れた範囲の
    /// 古いほうではない）。
    #[test]
    fn ctrl_enter_keeps_the_field_when_nothing_is_written_and_fills_the_active_range() {
        let mut document = indexed_editable();
        document.apply(
            Intent::Move {
                step: CursorStep::Down,
                mode: SelectionMode::Extend,
            },
            normal(),
        );
        document.apply(Intent::Type('k'), normal());
        let DocumentEffect::ReadPasted { .. } = document.apply(Intent::FillSelection, normal())
        else {
            panic!("行を読ませる");
        };
        assert!(
            document.editing_cell().is_none(),
            "読んでいる間は欄を閉じる"
        );
        document.paste_unread();
        assert!(document.editing_cell().is_some(), "読めなければ欄を戻す");
        assert!(document.edits().is_empty());

        // 離れた範囲：先に (0,0)〜(1,0)、次に Ctrl+クリックで (0,1)〜(1,1) を選んで打つと、書くのは後の範囲。
        let mut ranges = indexed_editable();
        ranges.apply(
            Intent::Move {
                step: CursorStep::Down,
                mode: SelectionMode::Extend,
            },
            normal(),
        );
        ranges.apply(pointing(100, 58, SelectionMode::Add), normal());
        ranges.apply(Intent::Release, normal());
        ranges.apply(
            Intent::Move {
                step: CursorStep::Down,
                mode: SelectionMode::Extend,
            },
            normal(),
        );
        assert!(ranges.ranges().is_multiple());
        assert_eq!(ranges.selection().cursor(), CellPosition::new(1, 1));
        ranges.apply(Intent::Type('z'), normal());
        filled(&mut ranges);
        assert_eq!(
            text_of(&ranges, RowId::Source(0), ColumnId::Source(1)),
            Some("z")
        );
        assert_eq!(
            text_of(&ranges, RowId::Source(1), ColumnId::Source(1)),
            Some("z")
        );
        assert_eq!(
            text_of(&ranges, RowId::Source(1), ColumnId::Source(0)),
            None
        );
    }

    /// 1 つの値を広い範囲に貼れば、範囲のすべてのセルになる（Excel と同じ・ADR 0029 の決定 1）。
    #[test]
    fn pasting_one_value_over_a_range_fills_it() {
        let mut document = indexed_editable();
        document.apply(
            Intent::Move {
                step: CursorStep::Down,
                mode: SelectionMode::Extend,
            },
            normal(),
        );
        document.apply(
            Intent::Move {
                step: CursorStep::Right,
                mode: SelectionMode::Extend,
            },
            normal(),
        );
        pasted(&mut document, "z\r\n");
        for (row, column) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
            assert_eq!(
                text_of(&document, RowId::Source(row), ColumnId::Source(column)),
                Some("z")
            );
        }
        assert_eq!(
            text_of(&document, RowId::Source(2), ColumnId::Source(0)),
            None
        );
        document.apply(Intent::Undo, normal());
        assert!(document.edits().is_empty());
    }

    /// 索引を作っている間に今読める所より下へ届く貼り付け・上限を超える貼り付け・切り詰めた行への貼り付けは、何も書かずに
    /// 知らせる（ADR 0029 の決定 2・4）。空は何もしない。
    #[test]
    fn pastes_that_cannot_be_written_change_nothing() {
        let mut document = editable();
        document.apply(moving(CursorStep::Down), normal());
        document.apply(moving(CursorStep::Down), normal());
        assert_eq!(document.paste("a\r\nb\r\n"), PasteRequest::Done);
        assert!(document.edits().is_empty());
        text_at(
            &document.frame(normal(), palette()),
            "表の終わりを越えるかもしれないので、行を数え終えてから貼ってください（今は見えている所までしか数えていません）",
        );
        assert_eq!(document.paste(""), PasteRequest::Done);
        // 列全体を選んで 1 つの値を貼ると、200 万行を埋めることになる。
        let mut large = editable();
        large.report(JobReport::Finished(IndexSummary::new(2_000_000, None)));
        large.apply(Intent::SelectColumns, normal());
        assert_eq!(large.paste("z"), PasteRequest::Done);
        text_at(
            &large.frame(normal(), palette()),
            "貼り付けが大きすぎるので貼りませんでした（100 万セル・64 MiB まで）",
        );
        // 切り詰めたレコードにかかれば書かない。
        let mut cut = indexed_editable();
        let PasteRequest::Read { first_line, count } = cut.paste("a\tb\r\n") else {
            panic!("読ませるはず");
        };
        assert_eq!((first_line, count), (0, 1));
        cut.paste_rows(
            0,
            (&editable_rows()[..1], &[CutRecord::new(0, Some(1))]),
            normal(),
        );
        assert!(cut.edits().is_empty());
        // 編集中は字のまま欄に挿す（複数のセルでも）。
        let mut editing = editable();
        editing.apply(Intent::Edit, normal());
        assert_eq!(editing.paste("a\tb\r\n"), PasteRequest::Done);
        assert!(
            editing
                .editor()
                .is_some_and(|editor| editor.field().text() == "0012a\tb")
        );
    }

    /// ダブルクリックは、押したセルを編集の状態（元の値・カーソルは末尾）で直し始める。見出しの上では 1 度押したのと同じ
    /// （ADR 0026 の決定 1）。
    #[test]
    fn double_clicking_a_cell_edits_it_from_the_end() {
        let window = normal();
        let mut document = editable();
        let layout = document.layout(0, window.content_metrics());
        let body = document.bars(window).body();
        let inside = |row: usize, column: usize| {
            let rect = layout.cell_rect(row, column).unwrap();
            PixelPoint::new(
                body.origin().x() + rect.origin().x() + 3,
                body.origin().y() + rect.origin().y() + 3,
            )
        };
        // 2 行目の 2 列目（b2）を 1 度押して離し、2 度目。
        document.apply(pressing(inside(2, 1), SelectionMode::Collapse), window);
        document.apply(Intent::Release, window);
        document.apply(Intent::DoubleClick { at: inside(2, 1) }, window);
        document.apply(Intent::Release, window);
        assert_eq!(document.selection().cursor(), CellPosition::new(1, 1));
        assert!(
            document
                .editor()
                .is_some_and(|editor| editor.field().text() == "b2")
        );
        typed(&mut document, "x");
        document.apply(Intent::Confirm, window);
        assert_eq!(
            edited(&document, 1, 1),
            Some(&CellValue::Text(String::from("b2x")))
        );
        // 列の番号の行（固定した上の行）のダブルクリックは、列を選ぶだけで直し始めない。
        document.apply(Intent::DoubleClick { at: inside(0, 0) }, window);
        document.apply(Intent::Release, window);
        assert_eq!(document.editor(), None);
    }

    /// Ctrl+X の 2 段を、表 `editable_rows` の原本から行を読んで済ませ、クリップボードに置く文字列を返す。
    fn cut_text(document: &mut Document) -> String {
        let DocumentEffect::Copy { first_line, count } = document.apply(Intent::Cut, normal())
        else {
            panic!("切り取りは行を読ませるはず");
        };
        let rows: Vec<CellRow> = editable_rows()
            .into_iter()
            .skip(usize::try_from(first_line).unwrap())
            .take(usize::try_from(count).unwrap())
            .collect();
        String::from(document.copy_text(first_line, &rows, &[]).unwrap().as_str())
    }

    fn cell_text(document: &Document, row: u64, column: usize) -> Option<&str> {
        text_of(document, RowId::Source(row), ColumnId::Source(column))
    }

    fn extending(step: CursorStep) -> Intent {
        Intent::Move {
            step,
            mode: SelectionMode::Extend,
        }
    }

    /// 切り取って同じ窓で貼ると、貼った後に元が空になり、1 手で戻る（ADR 0029 の決定 5）。切り取っただけでは元は残る。
    #[test]
    fn cutting_and_pasting_in_the_window_moves_the_values_in_one_step() {
        let mut document = indexed_editable();
        document.apply(extending(CursorStep::Right), normal());
        let text = cut_text(&mut document);
        assert_eq!(text, "0012\t東京\r\n");
        assert!(document.edits().is_empty());
        assert!(document.live_cut().is_some());
        text_at(
            &document.frame(normal(), palette()),
            "1 行 × 2 列を切り取りました（貼るまで元は残ります・Esc でやめる）",
        );
        document.apply(moving(CursorStep::Left), normal());
        document.apply(moving(CursorStep::Down), normal());
        document.apply(moving(CursorStep::Down), normal());
        pasted(&mut document, &text);
        assert_eq!(
            [cell_text(&document, 2, 0), cell_text(&document, 2, 1)],
            [Some("0012"), Some("東京")]
        );
        assert_eq!(
            [cell_text(&document, 0, 0), cell_text(&document, 0, 1)],
            [Some(""), Some("")]
        );
        assert!(document.live_cut().is_none());
        text_at(
            &document.frame(normal(), palette()),
            "1 行 × 2 列を動かしました",
        );
        document.apply(Intent::Undo, normal());
        assert!(document.edits().is_empty());
    }

    /// 元と貼る先が重なれば、重なった所は貼った値になる（ADR 0029 の決定 5）。
    #[test]
    fn a_moved_range_that_overlaps_keeps_the_pasted_values() {
        let mut document = indexed_editable();
        document.apply(extending(CursorStep::Down), normal());
        let text = cut_text(&mut document);
        // 範囲を広げた後のカーソルは下の端（2 行目）。畳んで 1 つ下（2 行目）から貼る。
        document.apply(moving(CursorStep::Up), normal());
        document.apply(moving(CursorStep::Down), normal());
        pasted(&mut document, &text);
        assert_eq!(
            [
                cell_text(&document, 0, 0),
                cell_text(&document, 1, 0),
                cell_text(&document, 2, 0)
            ],
            [Some(""), Some("0012"), Some("b1")]
        );
    }

    /// Esc・直し始め・表を変える操作・ほかで置いた文字列では、貼っても元を消さない（ただのコピーとして貼る）。
    #[test]
    fn a_cut_that_was_dropped_pastes_as_a_copy() {
        let drops: [fn(&mut Document); 3] = [
            |document| {
                document.apply(Intent::Cancel, normal());
            },
            |document| {
                typed(document, "x");
                document.apply(Intent::Cancel, normal());
            },
            |document| {
                document.apply(Intent::InsertRows(RowPlace::End), normal());
            },
        ];
        for drop in drops {
            let mut document = indexed_editable();
            let text = cut_text(&mut document);
            drop(&mut document);
            assert!(document.live_cut().is_none());
            document.apply(moving(CursorStep::Top), normal());
            document.apply(moving(CursorStep::Down), normal());
            pasted(&mut document, &text);
            assert_eq!(cell_text(&document, 0, 0), None);
            assert_eq!(cell_text(&document, 1, 0), Some("0012"));
        }
        // ほかのアプリが置いた文字列（切り取りの文字列と違う）なら、枠があっても元を消さない。
        let mut document = indexed_editable();
        let _text = cut_text(&mut document);
        document.apply(moving(CursorStep::Down), normal());
        pasted(&mut document, "other\r\n");
        assert_eq!(cell_text(&document, 0, 0), None);
        assert_eq!(cell_text(&document, 1, 0), Some("other"));
    }

    /// 離れた範囲は切り取らずに知らせる（Excel と同じ）。
    #[test]
    fn many_ranges_are_not_cut() {
        let mut document = indexed_editable();
        document.others.push(document.selection);
        assert_eq!(document.apply(Intent::Cut, normal()), DocumentEffect::Stay);
        text_at(
            &document.frame(normal(), palette()),
            "離れた範囲は切り取れません（1 つの範囲を選んでください）",
        );
    }

    /// 列を足した後の切り取りは、元のセルを正しい列から読む。元が空になり、戻した後に隣の列の値を打っても書き換えが
    /// 残る（GitHub Issue #153 の 1）。
    #[test]
    fn cutting_after_a_column_insert_reads_the_right_source_column() {
        // 合成ルートと同じく、列を足し消しした後・貼った後・戻した後に見えている行を読み直す。
        let reread = |document: &mut Document| document.show(0, editable_rows());
        let mut document = indexed_editable();
        document.apply(Intent::InsertColumns(ColumnPlace::Left), normal());
        reread(&mut document);
        document.apply(moving(CursorStep::Top), normal());
        document.apply(moving(CursorStep::Right), normal());
        document.apply(moving(CursorStep::Right), normal());
        // 表は [足した列, S0, S1]。S1 の 0 行目（東京）を切り取って 2 行目へ。
        let text = cut_text(&mut document);
        assert_eq!(text, "東京\r\n");
        document.apply(moving(CursorStep::Down), normal());
        document.apply(moving(CursorStep::Down), normal());
        pasted(&mut document, &text);
        assert_eq!(cell_text(&document, 0, 1), Some(""));
        assert_eq!(cell_text(&document, 2, 1), Some("東京"));
        document.apply(Intent::Undo, normal());
        reread(&mut document);
        assert_eq!(document.selection().cursor(), CellPosition::new(0, 2));
        typed(&mut document, "0012");
        document.apply(Intent::Confirm, normal());
        assert_eq!(cell_text(&document, 0, 1), Some("0012"));
        // S0（表示の 1 列目）も、切り取って貼れば元が空になる。
        let mut first = indexed_editable();
        first.apply(Intent::InsertColumns(ColumnPlace::Left), normal());
        reread(&mut first);
        first.apply(moving(CursorStep::Top), normal());
        first.apply(moving(CursorStep::Right), normal());
        let first_text = cut_text(&mut first);
        first.apply(moving(CursorStep::Down), normal());
        first.apply(moving(CursorStep::Down), normal());
        pasted(&mut first, &first_text);
        assert_eq!(
            [cell_text(&first, 0, 0), cell_text(&first, 2, 0)],
            [Some(""), Some("0012")]
        );
    }

    /// 表示だけの列の入れ替えの後に右の端を越えて貼っても、見えていた順に p q r と並ぶ（GitHub Issue #153 の 2）。
    #[test]
    fn pasting_past_the_right_edge_keeps_the_shown_column_order() {
        let mut document = indexed_editable();
        document.apply(Intent::SelectColumns, normal());
        document.apply(Intent::MoveColumns(ColumnShift::Right), normal());
        document.apply(moving(CursorStep::Top), normal());
        document.apply(moving(CursorStep::Left), normal());
        document.apply(moving(CursorStep::Left), normal());
        pasted(&mut document, "p\tq\tr\r\n");
        let added = document.changes().columns().column_at(2).unwrap();
        // 表示の順は S1・S0・足した列のまま。
        assert_eq!(document.order, vec![1, 0, 2]);
        assert_eq!(
            [
                text_of(&document, RowId::Source(0), ColumnId::Source(1)),
                text_of(&document, RowId::Source(0), ColumnId::Source(0)),
                text_of(&document, RowId::Source(0), added),
            ],
            [Some("p"), Some("q"), Some("r")]
        );
    }

    /// 行を足した貼り付けを戻すと、カーソルは貼った左上（GitHub Issue #153 の 4）。断った貼り付けは切り取りの枠を残す（5）。
    #[test]
    fn undo_returns_to_the_pasted_corner_and_a_refused_paste_keeps_the_cut() {
        let mut document = indexed_editable();
        document.apply(moving(CursorStep::Down), normal());
        pasted(&mut document, "a\r\nb\r\nc\r\nd\r\ne\r\n");
        document.apply(Intent::Undo, normal());
        assert_eq!(document.selection().cursor(), CellPosition::new(1, 0));
        let mut cut = editable();
        let _text = cut_text(&mut cut);
        cut.apply(moving(CursorStep::Down), normal());
        cut.apply(moving(CursorStep::Down), normal());
        // 索引を作り終えていないので、表の終わりを越える貼り付けは断る。枠は残る。
        assert_eq!(cut.paste("x\r\ny\r\n"), PasteRequest::Done);
        assert!(cut.live_cut().is_some());
    }

    /// 上限：行（コピーと同じ 10 万行）・列（16,384 列）・分ける前の区切りの数（GitHub Issue #153 の 3）。
    #[test]
    fn pastes_beyond_the_row_and_column_limits_are_refused_early() {
        let mut document = indexed_editable();
        let many_columns = "x\t".repeat(MAX_PASTE_COLUMNS);
        assert_eq!(document.paste(&many_columns), PasteRequest::Done);
        let mut tall = editable();
        tall.report(JobReport::Finished(IndexSummary::new(500_000, None)));
        tall.apply(Intent::SelectColumns, normal());
        assert_eq!(tall.paste("z"), PasteRequest::Done);
        let tabs = "\t".repeat(2_000_001);
        assert_eq!(document.paste(&tabs), PasteRequest::Done);
        text_at(
            &document.frame(normal(), palette()),
            "貼り付けが大きすぎるので貼りませんでした（100 万セル・64 MiB まで）",
        );
    }

    /// 裏の検索の 1 回分を、表 `editable_rows` の原本で済ませる（合成ルートと app の `FindFold` の代わり）。
    fn finish_finding(document: &mut Document, first: DocumentEffect) -> DocumentEffect {
        let mut effect = first;
        while effect == DocumentEffect::Find {
            let (scope, pass) = document.take_find_pass().unwrap();
            let found = scan_pass(document, &scope, pass);
            effect = document.found(found, normal());
        }
        effect
    }

    /// 1 回分 `pass` の範囲のレコードを順に照らす（前へは最初、後ろへは最後の当たり）。
    fn scan_pass(document: &Document, scope: &FindScope, pass: FindPass) -> Option<(u64, usize)> {
        let rows = editable_rows();
        let end = pass
            .first()
            .saturating_add(pass.count())
            .min(u64::try_from(rows.len()).unwrap());
        let hits = (pass.first()..end)
            .filter(|&record| document.rows.position_of(RowId::Source(record)).is_some())
            .filter_map(|record| {
                let cells = rows[usize::try_from(record).unwrap()].to_strings();
                record_hit(scope, pass.bound(), record, &cells).map(|column| (record, column))
            });
        match pass.bound() {
            FindBound::After(_) => hits.min(),
            FindBound::Before(_) => hits.max(),
        }
    }

    /// レコード `record`（セル `cells`）で、境 `bound` の内側で当たる列。
    fn record_hit(
        scope: &FindScope,
        bound: FindBound,
        record: u64,
        cells: &[String],
    ) -> Option<usize> {
        let row = RowId::Source(record);
        let source = |index: usize| cells.get(index).cloned();
        let width = scope.width();
        match bound {
            FindBound::After(after) => {
                let from = after
                    .filter(|&(at, _)| at == record)
                    .map_or(0, |(_, column)| column + 1);
                scope.first_in_row(row, from..width, source)
            }
            FindBound::Before(before) => {
                let to = before
                    .filter(|&(at, _)| at == record)
                    .map_or(width, |(_, column)| column);
                scope.last_in_row(row, 0..to, source)
            }
        }
    }

    fn finding(document: &mut Document, text: &str, forward: bool) {
        let query = FindQuery::new(text, FindOptions::standard());
        let effect = document.find(query, forward);
        finish_finding(document, effect);
    }

    fn again(document: &mut Document, forward: bool) {
        let effect = document.apply(Intent::FindAgain { forward }, normal());
        finish_finding(document, effect);
    }

    /// 見つけたセルが表に重なった面（開いている検索の箱）の下に入るなら、面の下へ出るまで表を送る。面と横に重ならない
    /// 列は送らない。表の先頭で送れなければ知らせる（GitHub Issue #286）。
    #[test]
    fn a_found_cell_is_not_left_under_the_cover() {
        let mut document = tall();
        let window = normal();
        let page = document.page_rows(window);
        let body = document.bars(window).body();
        let row = document.layout(0, window.content_metrics()).row_height();
        let rows_top = body.origin().y() + row * document.fixed_rows();
        // 面は窓の全幅で、本体の上の 3 行を覆う（3 行目の途中まで）。
        let cover = PixelRect::new(
            PixelPoint::new(0, 0),
            PixelSize::new(window.size().width(), rows_top + row * 2 + 1),
        );
        document.set_cover(Some(cover));
        assert_eq!(document.covered_rows(0, window), (3, 0));
        let looking = |shown: &mut Document, record: u64| {
            let query = FindQuery::new("x", FindOptions::standard());
            assert_eq!(shown.find(query, true), DocumentEffect::Find);
            shown.found(Some((record, 0)), window)
        };
        // 表示の先頭を 40 に。本体の 41 行目（レコード 42）は面の下に入るので、面のすぐ下へ。
        document.show(41, cell_rows(&vec![vec![String::from("x")]; 20]));
        assert_eq!(document.view().first_body(), 40);
        assert_eq!(
            looking(&mut document, 42),
            DocumentEffect::Read { first_body: 38 }
        );
        assert!(!document.cover_hides_cursor(window));
        // 面の下でない行は動かさない。
        assert_eq!(looking(&mut document, 45), DocumentEffect::Stay);
        assert!(page > 4);
        // 表の先頭の行は送れないので、面の下に残ると分かる。
        document.show(1, cell_rows(&vec![vec![String::from("x")]; 20]));
        assert_eq!(looking(&mut document, 2), DocumentEffect::Stay);
        assert!(document.cover_hides_cursor(window));
    }

    /// 面と横に重ならない列は覆われず、面が無ければ覆われない（GitHub Issue #286）。
    #[test]
    fn only_columns_beside_the_cover_are_covered() {
        let mut document = tall();
        let window = normal();
        document.set_cover(Some(PixelRect::new(
            PixelPoint::new(window.size().width() - 1, 0),
            PixelSize::new(1, window.size().height()),
        )));
        assert_eq!(document.covered_rows(0, window), (0, 0));
        // 下に寄せた面は、本体の下の行を覆う。その行の当たりは面の上へ出す。
        let body = document.bars(window).body();
        let row = document.layout(0, window.content_metrics()).row_height();
        let bottom = body.origin().y() + body.size().height();
        document.set_cover(Some(PixelRect::new(
            PixelPoint::new(0, bottom - row * 2),
            PixelSize::new(window.size().width(), row * 2),
        )));
        assert_eq!(document.covered_rows(0, window), (0, 2));
        let page = document.page_rows(window);
        let query = FindQuery::new("x", FindOptions::standard());
        assert_eq!(document.find(query, true), DocumentEffect::Find);
        // 本体の page - 1 行目（面の下）は、面の上の端の行へ。
        assert_eq!(
            document.found(Some((page, 0)), window),
            DocumentEffect::Read { first_body: 2 }
        );
        // 表の末尾の行は上へ送れないので、下の面の下に残ると分かる。
        assert_eq!(
            document.find(FindQuery::new("x", FindOptions::standard()), true),
            DocumentEffect::Find
        );
        let _effect = document.found(Some((99, 0)), window);
        assert_eq!(document.selection().cursor(), CellPosition::new(98, 0));
        assert!(document.cover_hides_cursor(window));
        // 面が無ければ今までどおり。
        document.set_cover(None);
        assert_eq!(document.covered_rows(0, window), (0, 0));
    }

    /// 次を探し、F3 で続け、表の終わりから頭へ回って知らせる。前へも探せる（ADR 0030 の決定 1）。
    #[test]
    fn finding_moves_to_the_next_match_and_wraps_around() {
        let mut document = indexed_editable();
        finding(&mut document, "B", true);
        assert_eq!(document.selection().cursor(), CellPosition::new(1, 0));
        again(&mut document, true);
        assert_eq!(document.selection().cursor(), CellPosition::new(1, 1));
        again(&mut document, true);
        assert_eq!(document.selection().cursor(), CellPosition::new(1, 0));
        text_at(
            &document.frame(normal(), palette()),
            "表の端まで探して、反対の端から続けて見つけました",
        );
        again(&mut document, false);
        assert_eq!(document.selection().cursor(), CellPosition::new(1, 1));
        finding(&mut document, "zzz", true);
        assert_eq!(document.selection().cursor(), CellPosition::new(1, 1));
        text_at(&document.frame(normal(), palette()), "見つかりませんでした");
    }

    /// 書き換えた値で探し、元の値では見つからない。足した行の中も探す（ADR 0030 の決定 2）。
    #[test]
    fn finding_uses_edited_values_and_added_rows() {
        let mut document = indexed_editable();
        document.apply(moving(CursorStep::Down), normal());
        document.apply(moving(CursorStep::Down), normal());
        typed(&mut document, "edited");
        document.apply(Intent::Confirm, normal());
        document.apply(moving(CursorStep::Top), normal());
        finding(&mut document, "c1", true);
        text_at(&document.frame(normal(), palette()), "見つかりませんでした");
        finding(&mut document, "EDIT", true);
        assert_eq!(document.selection().cursor(), CellPosition::new(2, 0));
        document.apply(Intent::InsertRows(RowPlace::End), normal());
        // 合成ルートは足した行を含めて読み直す（足した行はセルの無い行）。
        let mut shown = editable_rows();
        shown.push(CellRow::empty());
        document.show(0, shown.clone());
        typed(&mut document, "tail");
        document.apply(Intent::Confirm, normal());
        document.apply(moving(CursorStep::Top), normal());
        finding(&mut document, "tai", true);
        assert_eq!(document.selection().cursor(), CellPosition::new(3, 0));
        // 前へ探しても、足した行から原本の行へ戻れる。
        finding(&mut document, "0012", false);
        assert_eq!(document.selection().cursor(), CellPosition::new(0, 0));
    }

    /// F3 はまだ探していなければ知らせ、探している間の Esc は止める。
    #[test]
    fn find_again_needs_a_query_and_escape_stops_a_search() {
        let mut document = indexed_editable();
        assert_eq!(
            document.apply(Intent::FindAgain { forward: true }, normal()),
            DocumentEffect::Stay
        );
        text_at(
            &document.frame(normal(), palette()),
            "続けて探すものがありません（Ctrl+F で探す）",
        );
        let query = FindQuery::new("b", FindOptions::standard());
        assert_eq!(document.find(query, true), DocumentEffect::Find);
        assert!(document.take_find_pass().is_some());
        assert_eq!(
            document.apply(Intent::Cancel, normal()),
            DocumentEffect::StopFinding
        );
        document.find_stopped(false);
        text_at(&document.frame(normal(), palette()), "探すのをやめました");
    }

    /// 合成ルートの裏の数えの代わり：原本の行を並びの順に照らして、置き換える書き換えを集める。
    fn counted_replacements(document: &mut Document) -> Vec<(CellChange, String)> {
        let (scope, replacement) = document.take_replace_pass().unwrap();
        assert!(document.take_replace_pass().is_none(), "1 度だけ渡す");
        editable_rows()
            .iter()
            .zip(0_u64..)
            .filter(|&(_, record)| document.rows.position_of(RowId::Source(record)).is_some())
            .flat_map(|(cells, record)| {
                let source = |index: usize| cells.get(index).map(String::from);
                (0..scope.width())
                    .filter_map(|display| {
                        scope.replaced_cell((RowId::Source(record), display), &source, &replacement)
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn replacing_all(document: &mut Document, text: &str, replacement: &str) {
        let query = FindQuery::new(text, FindOptions::standard());
        assert_eq!(
            document.replace_all(query, replacement),
            DocumentEffect::Replace
        );
        let changes = counted_replacements(document);
        document.replace_counted((changes, 0), false);
    }

    /// 全部の置換は件数を見せてから Enter で 1 手の束として書き、Ctrl+Z 1 回ですべて戻る（ADR 0030 の決定 6・強制）。
    #[test]
    fn replacing_all_shows_the_count_first_and_undoes_in_one_step() {
        let mut document = indexed_editable();
        replacing_all(&mut document, "1", "#");
        text_at(
            &document.frame(normal(), palette()),
            "3 セルを置き換えます（Enter で置き換える・Esc でやめる）",
        );
        assert_eq!(cell_text(&document, 0, 0), None, "数えただけでは書かない");
        assert_eq!(
            document.apply(Intent::Confirm, normal()),
            DocumentEffect::Stay
        );
        assert_eq!(cell_text(&document, 0, 0), Some("00#2"));
        assert_eq!(cell_text(&document, 1, 0), Some("b#"));
        assert_eq!(cell_text(&document, 2, 0), Some("c#"));
        text_at(
            &document.frame(normal(), palette()),
            "3 セルを置き換えました（Ctrl+Z で戻す）",
        );
        document.apply(Intent::Undo, normal());
        for (row, column) in [(0, 0), (1, 0), (2, 0)] {
            assert_eq!(cell_text(&document, row, column), None);
        }
        document.apply(Intent::Redo, normal());
        assert_eq!(cell_text(&document, 1, 0), Some("b#"));
    }

    /// 見つからない・置き換えなかったことは、欄の中でその場に、絞り込む条件のうちオンのものを添えて言う。パレットを開き
    /// 直せば古い知らせは消える（GitHub Issue #291）。
    #[test]
    fn nothing_found_is_said_in_place_with_the_narrowing_conditions() {
        let mut document = indexed_editable();
        finding(&mut document, "zzz", true);
        assert_eq!(document.replace_guide().as_deref(), Some("見つかりません"));
        let strict = FindOptions::standard().toggled_case().toggled_whole_cell();
        let effect = document.find(FindQuery::new("B1", strict), true);
        finish_finding(&mut document, effect);
        assert_eq!(
            document.replace_guide().as_deref(),
            Some("見つかりません（オン: 大小を区別・セル全体）")
        );
        document.end_replace_note();
        assert_eq!(document.replace_guide(), None);
        text_at(
            &document.frame(normal(), palette()),
            "Ctrl+Shift+P でコマンド",
        );
    }

    /// 件数を見せている間は、何が何に変わるかの内訳と、部分一致の注意を出す。置き換えれば消える（GitHub Issue #289）。
    #[test]
    fn replacing_all_shows_what_changes_into_what() {
        let mut document = indexed_editable();
        assert!(document.replace_detail().is_empty());
        replacing_all(&mut document, "1", "#");
        let detail = document.replace_detail();
        // 1 を # に：どのセルも一部だけ変わる（0012・b1・c1）。
        assert_eq!(
            detail,
            vec![(
                String::from("一部だけ変わる 3 セル: “0012”→“00#2” など"),
                true
            )]
        );
        document.apply(Intent::Confirm, normal());
        assert!(document.replace_detail().is_empty());
    }

    /// 置換の欄が開いている間に数えている（GitHub Issue #189）：案内の 1 行に進み具合を出し、Enter は何もせず、Esc は
    /// 数えをやめ、ほかの意図（欄に打つ）は数えをやめて欄が受け取る。全部の置換が無ければ確かめの意図は無い。
    #[test]
    fn counting_inside_the_open_field_waits_for_the_count() {
        let mut document = indexed_editable();
        let query = FindQuery::new("1", FindOptions::standard());
        assert_eq!(
            document.replace_prompt_intent(Intent::Confirm),
            None,
            "置換が無い"
        );
        assert_eq!(
            document.replace_all(query.clone(), "#"),
            DocumentEffect::Replace
        );
        assert_eq!(
            document.replace_guide().as_deref(),
            Some("置き換えるセルを数えています… · Esc やめる")
        );
        for (intent, answer) in [
            (Intent::Confirm, (DocumentEffect::Stay, true)),
            // 表を眺めても数えはやめない。
            (
                Intent::Scroll(Scroll::Down { rows: 1 }),
                (DocumentEffect::Stay, false),
            ),
            (Intent::Cancel, (DocumentEffect::StopReplacing, true)),
        ] {
            assert_eq!(
                document.replace_prompt_intent(intent),
                Some(answer),
                "{intent:?}"
            );
        }
        let mut typed = indexed_editable();
        assert_eq!(typed.replace_all(query, "#"), DocumentEffect::Replace);
        assert_eq!(
            typed.replace_prompt_intent(Intent::Type('x')),
            Some((DocumentEffect::StopReplacing, false))
        );
    }

    /// 欄で数えをやめたら、すぐに手放す。やめる前に数え終えた結果が後から届いても、古い問いの件数を見せず、置き換えない。
    /// 結果の知らせは、欄への次の意図で消える（GitHub Issue #189）。
    #[test]
    fn a_count_stopped_from_the_field_ignores_a_late_result() {
        let mut document = indexed_editable();
        let query = FindQuery::new("1", FindOptions::standard());
        assert_eq!(document.replace_all(query, "#"), DocumentEffect::Replace);
        let late = counted_replacements(&mut document);
        document.replace_prompt_intent(Intent::Type('2'));
        document.replace_counted((late, 0), false);
        assert_eq!(
            document.replace_guide().as_deref(),
            Some("置き換えをやめました")
        );
        assert_eq!(document.replace_prompt_intent(Intent::Confirm), None);
        assert_eq!(document.replace_guide(), None);
        assert!(document.edits().is_empty());
    }

    /// 置換の欄が開いている間に件数を見せる（GitHub Issue #189）：件数と確かめを案内の 1 行に出し、数えた問いに印を付け、
    /// Enter で置き換えて、置き換えた数を出す。
    #[test]
    fn replacing_all_is_confirmed_inside_the_open_field() {
        let mut document = indexed_editable();
        replacing_all(&mut document, "1", "#");
        assert_eq!(
            document.replace_guide().as_deref(),
            Some("3 セルを置き換えます · Enter 置き換える · Esc やめる")
        );
        // 件数を見せている間は、欄が開いていなくても数えた問いに印を付ける。
        let marked = document.frame_found(normal(), palette(), None);
        assert!(
            marked
                .commands()
                .iter()
                .any(|command| matches!(command, DrawCommand::TextSpan { .. }))
        );
        assert_eq!(
            document.replace_prompt_intent(Intent::Confirm),
            Some((DocumentEffect::Stay, true))
        );
        assert_eq!(cell_text(&document, 1, 0), Some("b#"));
        assert_eq!(
            document.replace_guide().as_deref(),
            Some("3 セルを置き換えました · 表で Ctrl+Z で戻す")
        );
        assert_eq!(document.replace_prompt_intent(Intent::Confirm), None);
    }

    /// 置換の欄が開いている間に件数を見せていて、Esc かほかの意図（欄に打つ）なら置き換えない。ほかの意図は欄が受け取る。
    #[test]
    fn replacing_all_inside_the_open_field_can_be_declined() {
        let mut declined = indexed_editable();
        replacing_all(&mut declined, "1", "#");
        assert_eq!(
            declined.replace_prompt_intent(Intent::Type('2')),
            Some((DocumentEffect::Stay, false))
        );
        assert_eq!(cell_text(&declined, 1, 0), None);
        assert_eq!(
            declined.replace_guide().as_deref(),
            Some("置き換えをやめました")
        );
        let mut escaped = indexed_editable();
        replacing_all(&mut escaped, "1", "#");
        assert_eq!(
            escaped.replace_prompt_intent(Intent::Cancel),
            Some((DocumentEffect::Stay, true))
        );
        assert_eq!(cell_text(&escaped, 1, 0), None);
    }

    /// 件数を見せている間のほかの意図はやめ、数えている間の Esc は数えをやめる。数えている間に表が変われば書かない。
    #[test]
    fn replacing_all_can_be_declined_stopped_or_outdated() {
        let mut declined = indexed_editable();
        replacing_all(&mut declined, "1", "#");
        declined.apply(moving(CursorStep::Down), normal());
        assert_eq!(
            declined.selection().cursor(),
            CellPosition::new(1, 0),
            "下へはそのまま動く"
        );
        declined.apply(Intent::Confirm, normal());
        assert_eq!(cell_text(&declined, 0, 0), None);
        let mut escaped = indexed_editable();
        replacing_all(&mut escaped, "1", "#");
        assert_eq!(
            escaped.apply(Intent::Cancel, normal()),
            DocumentEffect::Stay
        );
        text_at(
            &escaped.frame(normal(), palette()),
            "置き換えるのをやめました",
        );

        let mut stopped = indexed_editable();
        let query = FindQuery::new("1", FindOptions::standard());
        assert_eq!(
            stopped.replace_all(query.clone(), "#"),
            DocumentEffect::Replace
        );
        assert_eq!(
            stopped.apply(Intent::Cancel, normal()),
            DocumentEffect::StopReplacing
        );
        stopped.replace_stopped(false);
        text_at(
            &stopped.frame(normal(), palette()),
            "置き換えるのをやめました",
        );

        let mut changed = indexed_editable();
        assert_eq!(
            changed.replace_all(query.clone(), "#"),
            DocumentEffect::Replace
        );
        let outdated = counted_replacements(&mut changed);
        typed(&mut changed, "x");
        changed.apply(Intent::Confirm, normal());
        changed.replace_counted((outdated, 0), false);
        text_at(
            &changed.frame(normal(), palette()),
            "数えている間に表が変わったので置き換えませんでした（もう一度数えてください）",
        );
        assert_eq!(cell_text(&changed, 1, 0), None);

        let mut many = indexed_editable();
        assert_eq!(many.replace_all(query, "#"), DocumentEffect::Replace);
        let _taken = many.take_replace_pass();
        many.replace_counted((Vec::new(), 2), true);
        text_at(
            &many.frame(normal(), palette()),
            "100 万セルを越えるので置き換えませんでした（範囲や条件を狭めてください）",
        );
    }

    /// 足した行の書き換えた値も置き換え、当たらなければ飛ばした行の数を添えて知らせる。
    #[test]
    fn replacing_all_reaches_added_rows_and_reports_nothing_found() {
        let mut document = indexed_editable();
        document.apply(Intent::InsertRows(RowPlace::End), normal());
        let mut shown = editable_rows();
        shown.push(CellRow::empty());
        document.show(0, shown.clone());
        typed(&mut document, "tail1");
        document.apply(Intent::Confirm, normal());
        replacing_all(&mut document, "1", "#");
        text_at(
            &document.frame(normal(), palette()),
            "4 セルを置き換えます（Enter で置き換える・Esc でやめる）",
        );
        document.apply(Intent::Confirm, normal());
        let added = document.rows.row_at(3).unwrap();
        assert_eq!(
            text_of(&document, added, ColumnId::Source(0)),
            Some("tail#")
        );

        let mut nothing = indexed_editable();
        let query = FindQuery::new("zzz", FindOptions::standard());
        assert_eq!(nothing.replace_all(query, "#"), DocumentEffect::Replace);
        let _taken = nothing.take_replace_pass();
        nothing.replace_counted((Vec::new(), 2), false);
        text_at(
            &nothing.frame(normal(), palette()),
            "置き換えるセルがありませんでした・1 MiB で切り詰めた 2 行は値の全体が見えないので飛ばしました",
        );
    }

    /// 1 つずつの置換：カーソルのセルが当たれば置き換えて次へ、当たらなければ次を探すだけ（ADR 0030 の決定 6）。
    #[test]
    fn replacing_one_rewrites_the_cursor_cell_then_finds_the_next() {
        let mut document = indexed_editable();
        let query = FindQuery::new("B", FindOptions::standard());
        // カーソルの「0012」は当たらないので、置き換えずに次の「b1」へ。
        let skipped = document.replace_one(query.clone(), "x");
        finish_finding(&mut document, skipped);
        assert_eq!(cell_text(&document, 0, 0), None);
        assert_eq!(document.selection().cursor(), CellPosition::new(1, 0));
        let first = document.replace_one(query.clone(), "x");
        finish_finding(&mut document, first);
        assert_eq!(cell_text(&document, 1, 0), Some("x1"));
        assert_eq!(document.selection().cursor(), CellPosition::new(1, 1));
        let second = document.replace_one(query, "x");
        finish_finding(&mut document, second);
        assert_eq!(cell_text(&document, 1, 1), Some("x2"));
        text_at(&document.frame(normal(), palette()), "見つかりませんでした");
        document.apply(Intent::Undo, normal());
        assert_eq!(cell_text(&document, 1, 1), None, "1 つずつ戻る");
        assert_eq!(cell_text(&document, 1, 0), Some("x1"));
    }

    /// 読めない正規表現では探さず、置き換えず、そう知らせる（ADR 0031）。読める正規表現は探す。
    #[test]
    fn a_broken_regex_neither_finds_nor_replaces() {
        let mut document = indexed_editable();
        let options = FindOptions::standard().toggled_regex();
        let broken = FindQuery::new("(b", options);
        assert_eq!(document.find(broken.clone(), true), DocumentEffect::Stay);
        text_at(
            &document.frame(normal(), palette()),
            "正規表現の式が読めないので探しませんでした（Alt+R で文字どおりに）",
        );
        assert_eq!(
            document.replace_one(broken.clone(), "x"),
            DocumentEffect::Stay
        );
        assert_eq!(document.replace_all(broken, "x"), DocumentEffect::Stay);
        finding(&mut document, "b", true);
        let effect = document.find(FindQuery::new(r"^c\d$", options), true);
        finish_finding(&mut document, effect);
        assert_eq!(document.selection().cursor(), CellPosition::new(2, 0));
    }

    /// 編集中の貼り付けは字のまま（引用を解かない・レビューで見つけた）。
    #[test]
    fn pasting_into_the_field_keeps_quotes() {
        let mut editing = editable();
        editing.apply(Intent::Edit, normal());
        assert_eq!(editing.paste("\"abc\"\r\n"), PasteRequest::Done);
        assert!(
            editing
                .editor()
                .is_some_and(|editor| editor.field().text() == "0012\"abc\"")
        );
    }

    /// 索引を作り終えたら、状態の行の改行の内訳はファイル全体で数えたもの（先頭の標本が LF だけでも、後ろに CRLF が
    /// あれば混在と出す・OPEN-08）。
    #[test]
    fn the_whole_file_line_endings_replace_the_sample() {
        let mut document = editable();
        let mixed = LineEndingCounts::new(5, 1_000_000, 0);
        document.report(JobReport::Finished(
            IndexSummary::new(1_000_005, None).with_line_endings(mixed),
        ));
        assert_eq!(document.format_summary().line_endings(), mixed);
        assert!(document.format_summary().line_endings().mixed());
    }

    /// 保存の途中は割合と、Esc でやめられることを出す。Esc は保存をやめる効き目を返し、やめたら書き換えは残る（SAVE-02）。
    #[test]
    fn saving_shows_progress_and_esc_stops_it() {
        let mut document = editable();
        typed(&mut document, "x");
        assert_eq!(document.apply(Intent::Save, normal()), DocumentEffect::Save);
        document.begin_saving();
        text_at(
            &document.frame(normal(), palette()),
            "保存しています…（Esc でやめる）",
        );
        document.save_progress(450, 1_000);
        text_at(
            &document.frame(normal(), palette()),
            "保存しています… 45%（Esc でやめる）",
        );
        document.save_progress(2_000, 1_000);
        text_at(
            &document.frame(normal(), palette()),
            "保存しています… 99%（Esc でやめる）",
        );
        assert_eq!(
            document.apply(Intent::Cancel, normal()),
            DocumentEffect::StopSaving
        );
        document.save_failed(SaveProblem::Cancelled);
        text_at(
            &document.frame(normal(), palette()),
            "保存をやめました。ファイルは元のままです（直した値は残っています）",
        );
        assert!(document.has_unsaved());
        assert!(!document.is_saving());
    }

    /// 末尾に足した行を戻すと、カーソルは表の最後の行に戻る。読み直す前の古い表示の行の数を使わない（GitHub Issue #147）。
    #[test]
    fn undoing_rows_added_at_the_end_leaves_the_cursor_on_a_real_row() {
        let mut document = indexed_editable();
        document.apply(moving(CursorStep::Down), normal());
        document.apply(moving(CursorStep::Down), normal());
        document.apply(Intent::InsertRows(RowPlace::End), normal());
        // 合成ルートは足した行を含めて読み直す（足した行はセルの無い行）。
        let mut shown = editable_rows();
        shown.push(CellRow::empty());
        document.show(0, shown.clone());
        assert_eq!(document.selection().cursor(), CellPosition::new(3, 0));
        document.apply(Intent::Undo, normal());
        assert_eq!(document.selection().cursor(), CellPosition::new(2, 0));
    }

    /// 種を固定した xorshift（テストは乱数の名前に頼らない・ARC-007）。
    struct Xorshift(u64);

    impl Xorshift {
        fn below(&mut self, bound: usize) -> usize {
            self.0 ^= self.0 << 13_u32;
            self.0 ^= self.0 >> 7_u32;
            self.0 ^= self.0 << 17_u32;
            usize::try_from(self.0 % u64::try_from(bound).unwrap()).unwrap()
        }
    }

    /// 無作為の 1 行（`row` 行目・列の数はばらばら・空のセルと改行を含むセルも）。
    fn ragged_row(rng: &mut Xorshift, row: usize) -> Vec<String> {
        const PIECES: [&str; 6] = ["", "a", "東京", "x\ny", "0012", "long-value"];
        (0..rng.below(90))
            .map(|column| format!("{row}-{column}{}", PIECES[rng.below(PIECES.len())]))
            .collect()
    }

    /// 列の数のばらばらな無作為の表。
    fn ragged_table(rng: &mut Xorshift) -> Vec<Vec<String>> {
        (0..=rng.below(30))
            .map(|row| ragged_row(rng, row))
            .collect()
    }

    /// 原本の 1 行 `cells` の、列の範囲 `spans` だけを持つ窓の行（合成ルートの `read_columns` と同じ形）。
    fn held_row(cells: &[String], spans: &[Range<usize>]) -> CellRow {
        let mut row = CellRow::empty();
        let held = spans
            .iter()
            .flat_map(|span| span.start.min(cells.len())..span.end.min(cells.len()));
        for column in held {
            row.push_with(|text| text.push_str(&cells[column]));
        }
        row.holding(cells.len(), spans)
    }

    /// 原本の行 `source` の、列の範囲 `columns` だけを持つ窓の行（`None` なら全部）。
    fn held_rows(source: &[Vec<String>], columns: Option<&[Range<usize>]>) -> Vec<CellRow> {
        source
            .iter()
            .map(|cells| {
                columns.map_or_else(|| CellRow::new(cells), |spans| held_row(cells, spans))
            })
            .collect()
    }

    /// 合成ルートと同じく、窓の行を `columns_to_read` の列だけで読み直す（行の並びは変えていない表）。読んだ後も列が
    /// 足りなければ、今の表示の先頭から読み直す（`columns_reread`）。
    fn reread_held(document: &mut Document, source: &[Vec<String>], first_body: u64) {
        read_held(document, source, first_body);
        if let Some(body) = document.columns_reread() {
            read_held(document, source, body);
        }
    }

    /// 本体の `body` 番目からの窓の行を、`columns_to_read` の列だけで読んで差し替える。
    fn read_held(document: &mut Document, source: &[Vec<String>], body: u64) {
        let first = usize::try_from(document.line_of(body)).unwrap();
        let end = first.saturating_add(50).min(source.len());
        let columns = document.columns_to_read();
        let part = source.get(first.min(end)..end).unwrap_or_default();
        document.show(document.line_of(body), held_rows(part, columns.as_deref()));
    }

    /// 見えている列（画面の配置から数える・実装の `held_columns` に頼らない）：固定した列と、最初の列から本体の幅に入る
    /// 列と、カーソルの列。
    fn shown_columns(document: &Document, window: WindowView) -> Vec<usize> {
        let layout = document.layout(document.first_column, window.content_metrics());
        let width = document.bars(window).body().size().width();
        layout
            .visible_columns()
            .filter(|&column| layout.column_left(column).is_some_and(|left| left < width))
            .chain(core::iter::once(document.selection().cursor().column()))
            .collect()
    }

    /// 窓の行の見えているセルが、原本 `source` と同じ（持っていない列が無い）。確かめたセルの数。
    fn assert_shown_cells_match(
        document: &Document,
        source: &[Vec<String>],
        window: WindowView,
    ) -> usize {
        let shown = shown_columns(document, window);
        let cells = document.view().rows().iter().flat_map(|row| {
            let line = &source[usize::try_from(row.line()).unwrap()];
            shown.iter().map(move |&column| (row, line, column))
        });
        let mut checked = 0_usize;
        for (row, line, column) in cells {
            assert!(row.values().holds(column), "{column}");
            assert_eq!(
                row.values().get(column),
                line.get(column).map(String::as_str)
            );
            checked += 1;
        }
        checked
    }

    /// 無作為の横と縦の動き。
    const WINDOW_STEPS: [Intent; 10] = [
        Intent::Move {
            step: CursorStep::Right,
            mode: SelectionMode::Collapse,
        },
        Intent::Move {
            step: CursorStep::Left,
            mode: SelectionMode::Collapse,
        },
        Intent::Move {
            step: CursorStep::RowEnd,
            mode: SelectionMode::Collapse,
        },
        Intent::Move {
            step: CursorStep::RowStart,
            mode: SelectionMode::Collapse,
        },
        Intent::Move {
            step: CursorStep::Down,
            mode: SelectionMode::Collapse,
        },
        Intent::Move {
            step: CursorStep::Up,
            mode: SelectionMode::Collapse,
        },
        Intent::Scroll(Scroll::Right { columns: 7 }),
        Intent::Scroll(Scroll::Left { columns: 5 }),
        Intent::FreezeColumns,
        Intent::UnfreezeColumns,
    ];

    /// 左から 3 列だけを持つ窓の行で開いた表（見出しは全部の列）と、全部の列を持つ表。
    fn held_and_whole(source: &[Vec<String>], header: Option<HeaderRows>) -> (Document, Document) {
        let first = 0..3_usize;
        let mut opened = held_rows(source, Some(core::slice::from_ref(&first)));
        if let (Some(_), Some(cells)) = (header, source.first()) {
            opened[0] = CellRow::new(cells);
        }
        (
            Document::open(String::from("t"), summary(), header, opened),
            Document::open(String::from("t"), summary(), header, cell_rows(source)),
        )
    }

    /// 2 つの表に同じ意図を渡し、読ませた窓の行を読む（一部の列だけを持つ表 `held` は `columns_to_read` の列だけ）。
    fn step_both(
        (held, whole): (&mut Document, &mut Document),
        source: &[Vec<String>],
        (intent, window): (Intent, WindowView),
    ) {
        if let DocumentEffect::Read { first_body } = held.apply(intent, window) {
            reread_held(held, source, first_body);
        }
        if let DocumentEffect::Read { .. } = whole.apply(intent, window) {
            reread(whole, &cell_rows(source));
        }
        assert_eq!(held.selection().cursor(), whole.selection().cursor());
        assert!(held.holds_shown_columns(window), "{intent:?}");
    }

    /// カーソルの行が両方に見えていれば、詳細ペインとカーソルのセルの置換の答えが同じ。
    fn assert_cursor_cells_match(held: &Document, whole: &Document, query: &FindQuery) {
        let body = held.selection().cursor().body();
        if held.view_row(body).is_none() || whole.view_row(body).is_none() {
            return;
        }
        assert_eq!(held.detail_content(), whole.detail_content());
        assert_eq!(
            held.replaced_at_cursor(query, "#"),
            whole.replaced_at_cursor(query, "#")
        );
    }

    /// 窓の行が見えている列の範囲だけを持っても、無作為の横と縦の動きの後に、見えているセル・詳細ペイン・カーソルの
    /// セルの置換の答えが、全部の列を持つ表と同じ（ADR 0033 の決定 4・GitHub Issue #22）。
    #[test]
    fn holding_only_the_shown_columns_answers_like_holding_them_all() {
        let mut rng = Xorshift(0x2026_1108_0022);
        let window = normal();
        let query = FindQuery::new("a", FindOptions::standard());
        let mut checked = 0_usize;
        for _ in 0..40_u32 {
            let source = ragged_table(&mut rng);
            let (mut held, mut whole) = held_and_whole(
                &source,
                (rng.below(2) == 0).then_some(HeaderRows::single(0)),
            );
            for _ in 0..60_u32 {
                let intent = WINDOW_STEPS[rng.below(WINDOW_STEPS.len())];
                step_both((&mut held, &mut whole), &source, (intent, window));
                checked += assert_shown_cells_match(&held, &source, window);
                assert_cursor_cells_match(&held, &whole, &query);
            }
        }
        assert!(checked > 10_000, "{checked}");
    }

    /// 表の中身の写し（差分・行の並び・列の並び・履歴の版）。読み取り専用の間に変わらないことを比べる。
    fn table_state(document: &Document) -> (CellEdits, Vec<OrderRun>, usize, u64) {
        (
            document.edits().clone(),
            document.runs_from(0, 10),
            document.widths.columns().len(),
            document.history.revision(),
        )
    }

    /// 読み取り専用の間に止める意図（EDIT-13）。
    const READ_ONLY_REFUSED: [Intent; 17] = [
        Intent::Type('x'),
        Intent::Edit,
        Intent::Delete,
        Intent::BreakLine,
        Intent::FillSelection,
        Intent::Save,
        Intent::Paste,
        Intent::Cut,
        Intent::ReplaceAll,
        Intent::Undo,
        Intent::Redo,
        Intent::InsertRows(RowPlace::Above),
        Intent::RemoveRows,
        Intent::InsertColumns(ColumnPlace::Left),
        Intent::RemoveColumns,
        Intent::InsertHere,
        Intent::RemoveHere,
    ];

    /// 読み取り専用の間に、意図の一覧と、貼り付け・字の確定・置換・ダブルクリックのどれでも表が変わらず、直し始めず、
    /// 知らせが出る。
    fn assert_refuses_every_change(document: &mut Document, window: WindowView) {
        let before = table_state(document);
        for intent in READ_ONLY_REFUSED {
            document.apply(intent, window);
            assert_eq!(table_state(document), before, "{intent:?}");
            assert!(document.editor.is_none(), "{intent:?}");
            assert_eq!(
                document.notice,
                Some(Notice::ReadOnly(ReadOnlyNotice::Refused)),
                "{intent:?}"
            );
        }
        assert_eq!(document.paste("p\tq"), PasteRequest::Done);
        document.commit_text("あ");
        let query = FindQuery::new("b", FindOptions::standard());
        assert_eq!(
            document.replace_one(query.clone(), "#"),
            DocumentEffect::Stay
        );
        assert_eq!(document.replace_all(query, "#"), DocumentEffect::Stay);
        let layout = document.layout(0, window.content_metrics());
        let body = document.bars(window).body();
        let rect = layout.cell_rect(1, 1).unwrap();
        let at = PixelPoint::new(
            body.origin().x() + rect.origin().x() + 3,
            body.origin().y() + rect.origin().y() + 3,
        );
        document.apply(pressing(at, SelectionMode::Collapse), window);
        document.apply(Intent::Release, window);
        document.apply(Intent::DoubleClick { at }, window);
        document.apply(Intent::Release, window);
        assert!(document.editor.is_none());
        assert_eq!(table_state(document), before);
    }

    /// 読み取り専用の間は、表の値・行・列を変える操作のどれでも表が変わらず、知らせが出る。見る・動くはできる。外せば
    /// また直せる（EDIT-13・GitHub Issue #238）。
    #[test]
    fn a_read_only_table_refuses_every_change_until_released() {
        let window = normal();
        let mut document = editable();
        // 戻せる書き換えを 1 つ作っておく（Undo も止まることを見る）。
        typed(&mut document, "z");
        document.apply(Intent::Confirm, window);
        document.apply(moving(CursorStep::Up), window);
        document.apply(Intent::ToggleReadOnly, window);
        assert!(document.is_read_only());
        assert_eq!(document.notice, Some(Notice::ReadOnly(ReadOnlyNotice::On)));
        assert_eq!(
            document.window_name(),
            "● e.csv（読み取り専用） — NeNe Ledger"
        );
        assert_refuses_every_change(&mut document, window);
        // 見る・動く・コピーはできる。
        document.apply(moving(CursorStep::Down), window);
        assert_eq!(document.selection().cursor(), CellPosition::new(1, 1));
        assert!(matches!(
            document.apply(Intent::Copy, window),
            DocumentEffect::Copy { .. }
        ));
        // 外せば直せる。
        document.apply(Intent::ToggleReadOnly, window);
        assert_eq!(document.notice, Some(Notice::ReadOnly(ReadOnlyNotice::Off)));
        assert_eq!(document.window_name(), "● e.csv — NeNe Ledger");
        typed(&mut document, "y");
        document.apply(Intent::Confirm, window);
        assert_eq!(
            edited(&document, 1, 1),
            Some(&CellValue::Text(String::from("y")))
        );
    }

    /// 書けないファイルは読み取り専用で始まり、理由を知らせ、直せない。理由が無ければそのまま（FILE-04・GitHub Issue #240）。
    #[test]
    fn a_blocked_file_starts_read_only_and_says_why() {
        let window = normal();
        let mut plain = editable();
        plain.start_read_only_if_blocked(None);
        assert!(!plain.is_read_only() && plain.notice.is_none());
        for (blocked, notice) in [
            (ReplaceFailure::Locked, ReadOnlyNotice::OpenedLocked),
            (ReplaceFailure::Denied, ReadOnlyNotice::OpenedDenied),
        ] {
            let mut document = editable();
            document.start_read_only_if_blocked(Some(blocked));
            assert!(document.is_read_only());
            assert_eq!(document.notice, Some(Notice::ReadOnly(notice)));
            assert_eq!(
                document.window_name(),
                "e.csv（読み取り専用） — NeNe Ledger"
            );
            let before = table_state(&document);
            typed(&mut document, "x");
            assert_eq!(table_state(&document), before);
            assert!(document.editor.is_none());
            document.apply(Intent::ToggleReadOnly, window);
            assert!(!document.is_read_only());
        }
    }

    /// JSON のオブジェクトか配列のセルは、詳細ペインで整形して見せ、補助の行に印を添える。値（文字数とバイト数）は元の
    /// まま。JSON でないセルは今までどおり（VIEW-07・GitHub Issue #247）。
    #[test]
    fn the_pane_lays_out_a_json_cell() {
        let rows = table(&[
            &["id", "data"],
            &["1", r#"{"a":1,"b":[true,"x"]}"#],
            &["2", "{a:1}"],
        ]);
        let mut document = opened_with(&rows);
        let window = view(400, 400, WindowPlacement::Normal);
        document.apply(moving(CursorStep::Right), window);
        let frame = document.frame(window, palette());
        let texts = all_texts(&frame);
        for wanted in [
            "2 行目　2 列目　22 文字　UTF-8 で 22 バイト　JSON（整形して表示）",
            "{",
            "  \"a\": 1,",
            "  \"b\": [",
            "    true,",
            // 詳細ペインの本文は 5 行（DETAIL_LINES）。残りは今までの長い値と同じく行の数で知らせる。
            "〈あと 3 行〉",
        ] {
            assert!(texts.contains(&wanted), "{wanted} が無い: {texts:?}");
        }
        document.apply(moving(CursorStep::Down), window);
        let next = document.frame(window, palette());
        let plain = all_texts(&next);
        assert!(
            plain.contains(&"3 行目　2 列目　5 文字　UTF-8 で 5 バイト"),
            "{plain:?}"
        );
        assert!(plain.contains(&"{a:1}"), "{plain:?}");
    }

    /// 数式に見えるセルは評価せず、表とコピーで字のまま（FR-031・SEC-01・GitHub Issue #249）。
    #[test]
    fn formula_like_cells_stay_as_text() {
        const FORMULAS: [&str; 6] = [
            "=1+1",
            "=HYPERLINK(\"http://example.com\",\"x\")",
            "+cmd|' /C calc'!A0",
            "@SUM(A1:A2)",
            "-2+3",
            "=WEBSERVICE(\"http://example.com\")",
        ];
        let rows: Vec<CellRow> = FORMULAS.iter().map(|&cell| CellRow::new(&[cell])).collect();
        let mut document = closed(Document::open(
            String::from("t"),
            summary(),
            None,
            rows.clone(),
        ));
        let shown: Vec<Option<&str>> = document
            .view()
            .rows()
            .iter()
            .map(|row| row.values().get(0))
            .collect();
        assert_eq!(shown, FORMULAS.map(Some));
        for _ in 1..FORMULAS.len() {
            document.apply(
                Intent::Move {
                    step: CursorStep::Down,
                    mode: SelectionMode::Extend,
                },
                normal(),
            );
        }
        let copied = copy_from(&mut document, &rows).unwrap();
        // 引用符の無い値は丸ごと、引用符を含む値はタブ区切りの引用の規則で囲まれるので、引用符の前までが字のまま。
        for formula in FORMULAS {
            let head = formula.split('"').next().unwrap_or(formula);
            assert!(copied.as_str().contains(head), "{formula} → {copied:?}");
        }
        assert!(copied.as_str().contains("+cmd|' /C calc'!A0"));
    }

    /// 見出しの行しかない表では、詳細ペインは「見えている範囲の外」でなく「表に行が無い」と出し、行の番号を言わない。
    /// 索引を作り終える前は今までどおり（GitHub Issue #256）。
    #[test]
    fn a_header_only_table_says_it_has_no_rows() {
        let mut document = opened_with(&table(&[&["only", "header"]]));
        let window = view(400, 400, WindowPlacement::Normal);
        let frame = document.frame(window, palette());
        assert!(all_texts(&frame).contains(&"〈カーソルの行は見えている範囲の外〉"));
        document.report(JobReport::Finished(IndexSummary::new(1, None)));
        let finished = document.frame(window, palette());
        let texts = all_texts(&finished);
        assert!(
            texts.contains(&"〈表に行が無い（見出しの行だけ）〉"),
            "{texts:?}"
        );
        assert!(
            !texts
                .iter()
                .any(|text| text.contains("見えている範囲の外") || text.contains("2 行目　")),
            "{texts:?}"
        );
    }

    /// 左の列を固定して狭い窓で右端の列へ動いた後に窓を広げると、表の右の端が収まる所まで、左へ隠れた列を戻す。収まらない
    /// 幅では戻しすぎない（GitHub Issue #265）。
    #[test]
    fn widening_the_window_brings_back_hidden_columns() {
        let cells = [
            "value-0000",
            "value-1111",
            "value-2222",
            "value-3333",
            "value-4444",
            "value-5555",
        ];
        let mut document = opened_with(&table(&[&cells, &cells]));
        let narrow = view(300, 400, WindowPlacement::Normal);
        document.apply(moving(CursorStep::Right), narrow);
        document.apply(Intent::FreezeColumns, narrow);
        let frozen = document.frozen;
        assert!(frozen >= 1, "{frozen}");
        document.apply(moving(CursorStep::RowEnd), narrow);
        let far = document.first_column;
        assert!(far > 3, "{far}");
        // 狭いままなら、最後の列を見せる所から動かさない。
        document.fit_columns(narrow);
        assert_eq!(document.first_column, far);
        // 広げると、全部が収まるので固定した列の次まで戻る。
        document.fit_columns(view(1600, 400, WindowPlacement::Normal));
        assert_eq!(document.first_column, frozen);
        // 少しだけ広げたときは、最後の列が収まるいちばん左まで。
        document.apply(moving(CursorStep::RowEnd), narrow);
        let middle = view(600, 400, WindowPlacement::Normal);
        document.fit_columns(middle);
        let layout = document.layout(document.first_column, middle.content_metrics());
        let width = document.bars(middle).body().size().width();
        assert!(layout.column_right(5).is_some_and(|right| right <= width));
        assert!(document.first_column > frozen && document.first_column < far);
    }

    /// 前書きの行（見出しの行より上）は字と行番号を薄く描き、データの行はふつう。開いたときのカーソルはデータの 1 行目
    /// （OPEN-11・GitHub Issue #273）。
    #[test]
    fn preamble_rows_are_faded_and_the_cursor_starts_on_data() {
        let rows = table(&[&["売上集計"], &["作成日"], &["地域"], &["東京"]]);
        let document = closed(Document::open(
            String::from("t"),
            summary(),
            Some(HeaderRows::single(2)),
            rows,
        ));
        assert_eq!(document.selection().cursor(), CellPosition::new(2, 0));
        let frame = document.frame(normal(), palette());
        let ink_of = |wanted: &str| {
            frame.commands().iter().find_map(|command| match command {
                DrawCommand::Text { text, color, .. } if text == wanted => Some(*color),
                DrawCommand::Fill { .. }
                | DrawCommand::Text { .. }
                | DrawCommand::Line { .. }
                | DrawCommand::TextSpan { .. } => None,
            })
        };
        // 前書きの字は、字の色をそのセルの背景（カーソルの列なら帯の色）へ寄せた色で、ふつうの字の色ではない。
        let text = Some(palette().text());
        assert!(ink_of("売上集計").is_some() && ink_of("売上集計") != text);
        assert!(ink_of("作成日").is_some() && ink_of("作成日") != text);
        assert_eq!(ink_of("東京"), Some(palette().text()));
        assert_eq!(ink_of("地域"), Some(palette().text()));
    }
}
