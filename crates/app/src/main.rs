//! `NeNe Ledger` の合成ルート。引数のファイルを開き、読み取り（scan）と画面（ui）を結ぶ。
//!
//! 🔴 端末への出力・引数・終了コードを扱ってよい唯一の場所（ARC-006・RS-014）。判断は持たない。
//! Windows の exe はコンソールを出さない GUI の形で、失敗はメッセージの窓で知らせる（Issue #56）。

#![cfg_attr(windows, windows_subsystem = "windows")]

mod dialect_doubt;
mod dumping;
mod find_fold;
mod finding;
mod guessed;
mod open_failure;
mod open_palette;
mod opening;
mod preferences;
mod reader;
mod replace_fold;
mod rows;
mod saving;
mod scan_bytes;
mod scan_lookup;
mod totals_fold;
mod viewer;
mod workers;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use nene_ledger_application::about_line::AboutLine;
use nene_ledger_application::file_stamp::FileStamp;
use nene_ledger_application::theme_state::ThemeState;
use nene_ledger_scan::source::Source;

use crate::open_failure::OpenFailure;

fn main() -> ExitCode {
    let arguments: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();
    // 版を書いて終わる（`Cargo.toml` の版・ADR 0032・DEVELOPMENT_WORKFLOW 第 10 節）。
    if let [flag] = arguments.as_slice()
        && flag == "--version"
    {
        return version_command();
    }
    if let [flag, file] = arguments.as_slice()
        && flag == "--dump"
    {
        return dump_command(&PathBuf::from(file));
    }
    // 測定用: 頭から探し、最初に当たったセルを書く（ADR 0030 の決定 4・QLT-014）。見つからなければ終了コード 1。
    if let [flag, text, file] = arguments.as_slice()
        && flag == "--find"
    {
        return find_command(&text.to_string_lossy(), &PathBuf::from(file));
    }
    // 測定用: 開いて何も変えずに保存する（KEEP-02 の往復の実測・BIG-14・GitHub Issue #58）。
    if let [flag, file] = arguments.as_slice()
        && flag == "--resave"
    {
        return match saving::save_edited(
            &PathBuf::from(file),
            &nene_ledger_core::cell_edits::CellEdits::empty(),
        ) {
            Ok(()) => ExitCode::SUCCESS,
            Err(failure) => {
                report(&saving::describe(&failure));
                ExitCode::FAILURE
            }
        };
    }
    // `--theme <名前>` は、この起動の間だけ組み込みの配色を選ぶ（設定は変えない・無ければ設定の希望・ADR 0012）。
    let (theme, rest) = match arguments.as_slice() {
        [flag, name, rest @ ..] if flag == "--theme" => {
            let Some(theme) = name.to_str().and_then(ThemeState::named) else {
                report(&format!(
                    "配色 {} は無い（組み込みの 125 の名前から選ぶ）",
                    name.to_string_lossy()
                ));
                return ExitCode::from(2_u8);
            };
            (Some(theme), rest)
        }
        rest => (None, rest),
    };
    // `--new` は原本の無い新しい表を開く（ADR 0028 の決定 1・2）。
    if let [flag] = rest
        && flag == "--new"
    {
        return show_blank(theme);
    }
    let Some(path) = rest.first().map(PathBuf::from).or_else(chosen_file) else {
        return ExitCode::from(2_u8);
    };
    open_and_show(&path, theme)
}

/// `--version`：名前と版を標準出力に書く。
fn version_command() -> ExitCode {
    // コンソールの無い Windows の exe は、起動したコマンドプロンプトに結んで書く。
    #[cfg(windows)]
    nene_ledger_ui_win32::desktop::attach_console();
    let mut out = std::io::stdout().lock();
    match writeln!(out, "{}", AboutLine::Version.text()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(_failure) => ExitCode::FAILURE,
    }
}

/// `--dump`：`path` を表として読み、タブ区切りで標準出力に書く。
fn dump_command(path: &Path) -> ExitCode {
    // コンソールの無い Windows の exe は、起動したコマンドプロンプトに結んで書く。
    #[cfg(windows)]
    nene_ledger_ui_win32::desktop::attach_console();
    match dumping::dump(path) {
        Ok(cuts) => {
            if let Some((count, first)) = cuts.summary() {
                warn(&format!(
                    "1 MiB を超えるレコード {count} 件は先頭の 1 MiB だけを書いた（最初は {} 行目）",
                    first.saturating_add(1)
                ));
            }
            ExitCode::SUCCESS
        }
        Err(failure) => {
            report(&describe(&failure));
            ExitCode::FAILURE
        }
    }
}

/// `--find`：`path` を頭から `text` で探し、最初に当たったセルを標準出力に書く。見つからなければ終了コード 1。
fn find_command(text: &str, path: &Path) -> ExitCode {
    // コンソールの無い Windows の exe は、起動したコマンドプロンプトに結んで書く。
    #[cfg(windows)]
    nene_ledger_ui_win32::desktop::attach_console();
    match finding::find_first(path, text) {
        Ok(Some((record, column))) => {
            let mut out = std::io::stdout().lock();
            match writeln!(out, "{record} 行目 {column} 列目") {
                Ok(()) => ExitCode::SUCCESS,
                Err(_failure) => ExitCode::FAILURE,
            }
        }
        Ok(None) => ExitCode::from(1_u8),
        Err(failure) => {
            report(&describe(&failure));
            ExitCode::FAILURE
        }
    }
}

/// `path` を開いて窓に出す。文字のファイルでないようなら、開くかを尋ねる。
fn open_and_show(path: &Path, theme: Option<ThemeState>) -> ExitCode {
    // 🔴 印は開く前に取る。開いた後で外から変われば、保存の前に違いが分かる（FILE-01・ADR 0025）。
    let stamp = nene_ledger_store::stamping::stamp_of(path).ok();
    match opening::open(path) {
        Ok(opened) if opened.looks_binary() && !open_anyway(path) => ExitCode::SUCCESS,
        Ok(opened) => show(path, opened, (theme, stamp)),
        Err(failure) => {
            report(&describe(&failure));
            ExitCode::FAILURE
        }
    }
}

/// 原本の無い新しい表（1 行 × 3 列・ADR 0028）を窓に出す。
fn show_blank(theme: Option<ThemeState>) -> ExitCode {
    let summary = opening::blank_summary();
    let document = nene_ledger_application::document::Document::blank(
        String::from(opening::BLANK_TITLE),
        summary,
    );
    let start = Start {
        document,
        format: summary.format(),
        path: None,
        stamp: None,
    };
    match run_window(start, || Ok(opening::blank_source()), theme) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            report(&message);
            ExitCode::FAILURE
        }
    }
}

/// 窓に出す表と、その原本の読み方・道筋（新しい表は無い）・開く前に取った印。
struct Start {
    document: nene_ledger_application::document::Document,
    format: nene_ledger_core::text_format::TextFormat,
    path: Option<PathBuf>,
    stamp: Option<FileStamp>,
}

/// 窓を開き、その起こす口で裏の索引の仕事を始め、閉じられるまで描く。開いた先頭の行は画面を作ったら手放す（窓が開いている
/// 間ずっと、セルごとの `String` の写しを持たない・GitHub Issue #22）。
fn show(
    path: &Path,
    opened: opening::Opened,
    (theme, stamp): (Option<ThemeState>, Option<FileStamp>),
) -> ExitCode {
    let start = Start {
        document: first_document(&opened),
        format: opened.summary().format(),
        path: Some(path.to_path_buf()),
        stamp,
    };
    drop(opened);
    let open_source =
        || Source::open(path).map_err(|failure| format!("ファイルを読めなかった: {failure:?}"));
    match run_window(start, open_source, theme) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            report(&message);
            ExitCode::FAILURE
        }
    }
}

/// 窓を開いて画面を結ぶ。窓（ui）と時計は OS ごとに取り替え、画面の状態と仕事の結び方は 1 つ（Issue #55）。
/// 原本は `open_source` で開く（索引の仕事と画面のスレッドとで別に開く）。
fn run_window(
    start: Start,
    open_source: impl Fn() -> Result<Source, String>,
    theme: Option<ThemeState>,
) -> Result<(), String> {
    use nene_ledger_application::ticker::Ticker;
    use nene_ledger_scan::block_size::BlockSize;
    use nene_ledger_scan::index_job::IndexJob;
    #[cfg(windows)]
    use nene_ledger_ui_win32::ui::Ui;
    #[cfg(not(windows))]
    use nene_ledger_ui_x11::ui::Ui;

    let Start {
        document,
        format,
        path,
        stamp,
    } = start;
    let ui =
        Ui::open(document.title()).map_err(|failure| format!("窓を出せなかった: {failure:?}"))?;
    let job = IndexJob::spawn(open_source()?, format, BlockSize::MEBIBYTE, ui.wakeup())
        .map_err(|failure| format!("索引の仕事を始められなかった: {failure:?}"))?;
    // 画面のスレッドが範囲を読むための原本は、索引の仕事とは別に開く（位置を指定して読むので共有しなくてよい）。
    let reader = reader::Reader::new((open_source()?, path, stamp), format, ui.wakeup());
    // 押し続けとドラッグの時計。X11 はアダプタのスレッド、Windows は窓のタイマー（SetTimer）。
    #[cfg(not(windows))]
    let ticker: Box<dyn Ticker> = Box::new(
        nene_ledger_adapters_x11::thread_ticker::ThreadTicker::spawn(ui.wakeup())
            .map_err(|failure| format!("時計を始められなかった: {failure:?}"))?,
    );
    #[cfg(windows)]
    let ticker: Box<dyn Ticker> = ui.ticker();
    let (preferences, warning) = preferences::Preferences::load(theme);
    if let Some(message) = warning {
        report(&message);
    }
    let mut viewer = viewer::Viewer::new(
        document,
        workers::Workers::new(job, ticker),
        reader,
        preferences,
    );
    ui.run(&mut viewer)
        .map_err(|failure| format!("窓が止まった: {failure:?}"))
}

/// 開いた先頭の行の画面。方言の判別に確信が無ければそれを知らせ（Issue #3）、文字のファイルでないようならそちらを知らせる
/// （知らせは 1 つなので、後のほうが残る）。
fn first_document(opened: &opening::Opened) -> nene_ledger_application::document::Document {
    use nene_ledger_application::notice::Notice;
    let mut document = nene_ledger_application::document::Document::open(
        opened.title().to_owned(),
        opened.summary(),
        opened.header(),
        opened.rows(),
    );
    document.mark_cut(opened.cuts().to_vec());
    if let Some(doubt) = opened.doubt() {
        document.notify(Notice::DialectUnsure {
            top: doubt.top(),
            second: doubt.second(),
        });
    }
    if opened.looks_binary() {
        document.notify(Notice::LooksBinary);
    }
    document
}

/// 開けなかった理由を、利用者に向けた文にする。🔴 表示言語の文言の表（ADR 0011）を入れるまでの仮の日本語。
fn describe(failure: &OpenFailure) -> String {
    match failure {
        OpenFailure::Scan(scan) => format!("ファイルを読めなかった: {scan:?}"),
        OpenFailure::Write(error) => format!("書き出せなかった: {error}"),
    }
}

/// 文字のファイルでないようなファイル `path` を開くか。Windows は尋ねる（既定は「いいえ」）。
#[cfg(windows)]
fn open_anyway(path: &Path) -> bool {
    nene_ledger_ui_win32::desktop::confirm(&format!(
        "{} は文字のファイルではないようです（NUL のバイトがあります）。\n表として開きますか？（ファイルは変えません）",
        path.display()
    ))
}

/// 文字のファイルでないようなファイルを開くか。X11 は尋ねる画面が無いので開く（状態の行で知らせる）。
#[cfg(not(windows))]
fn open_anyway(_path: &Path) -> bool {
    true
}

/// ファイルを渡されなかったとき。Windows は「ファイルを開く」画面で選ばせ（やめれば `None`）、X11 は使い方を知らせる。
fn chosen_file() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        nene_ledger_ui_win32::desktop::choose_file()
    }
    #[cfg(not(windows))]
    {
        report(
            "使い方: nene-ledger [--theme <配色>] <CSV か TSV のファイル> | nene-ledger [--theme <配色>] --new | nene-ledger --dump <ファイル>",
        );
        None
    }
}

/// `--dump` の注意を標準エラーに書く（どの OS でも端末に・Windows は `--dump` でコンソールに結んである）。
#[expect(
    clippy::print_stderr,
    reason = "RS-014: 合成ルートは --dump の注意を端末に出してよい（測定で値の欠けを知らせる・GitHub Issue #36）"
)]
pub(crate) fn warn(message: &str) {
    eprintln!("nene-ledger: {message}");
}

/// 利用者への失敗の知らせ。X11 は端末に、Windows はメッセージの窓に（コンソールが無い）。
#[cfg_attr(
    not(windows),
    expect(
        clippy::print_stderr,
        reason = "RS-014: 合成ルートは利用者への失敗の知らせを端末に出してよい唯一の場所"
    )
)]
fn report(message: &str) {
    #[cfg(windows)]
    nene_ledger_ui_win32::desktop::show_failure(&format!("nene-ledger: {message}"));
    #[cfg(not(windows))]
    eprintln!("nene-ledger: {message}");
}
