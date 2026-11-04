//! `NeNe Ledger` の合成ルート。引数のファイルを開き、読み取り（scan）と画面（ui）を結ぶ。
//!
//! 🔴 端末への出力・引数・終了コードを扱ってよい唯一の場所（ARC-006・RS-014）。判断は持たない。
//! Windows の exe はコンソールを出さない GUI の形で、失敗はメッセージの窓で知らせる（Issue #56）。

#![cfg_attr(windows, windows_subsystem = "windows")]

mod dumping;
mod guessed;
mod open_failure;
mod open_palette;
mod opening;
mod preferences;
mod reader;
mod rows;
mod totals_fold;
mod viewer;
mod workers;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use nene_ledger_application::theme_state::ThemeState;

use crate::open_failure::OpenFailure;

fn main() -> ExitCode {
    let arguments: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();
    if let [flag, file] = arguments.as_slice()
        && flag == "--dump"
    {
        // コンソールの無い Windows の exe は、起動したコマンドプロンプトに結んで書く。
        #[cfg(windows)]
        nene_ledger_ui_win32::desktop::attach_console();
        return match dumping::dump(&PathBuf::from(file)) {
            Ok(()) => ExitCode::SUCCESS,
            Err(failure) => {
                report(&describe(&failure));
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
    let Some(path) = rest.first().map(PathBuf::from).or_else(chosen_file) else {
        return ExitCode::from(2_u8);
    };
    match opening::open(&path) {
        Ok(opened) if opened.looks_binary() && !open_anyway(&path) => ExitCode::SUCCESS,
        Ok(opened) => show(&path, &opened, theme),
        Err(failure) => {
            report(&describe(&failure));
            ExitCode::FAILURE
        }
    }
}

/// 窓を開き、その起こす口で裏の索引の仕事を始め、閉じられるまで描く。
fn show(path: &Path, opened: &opening::Opened, theme: Option<ThemeState>) -> ExitCode {
    match run_window(path, opened, theme) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            report(&message);
            ExitCode::FAILURE
        }
    }
}

/// 窓を開いて画面を結ぶ。窓（ui）と時計は OS ごとに取り替え、画面の状態と仕事の結び方は 1 つ（Issue #55）。
fn run_window(
    path: &Path,
    opened: &opening::Opened,
    theme: Option<ThemeState>,
) -> Result<(), String> {
    use nene_ledger_application::document::Document;
    use nene_ledger_application::ticker::Ticker;
    use nene_ledger_scan::block_size::BlockSize;
    use nene_ledger_scan::index_job::IndexJob;
    use nene_ledger_scan::source::Source;
    #[cfg(windows)]
    use nene_ledger_ui_win32::ui::Ui;
    #[cfg(not(windows))]
    use nene_ledger_ui_x11::ui::Ui;

    let ui =
        Ui::open(opened.title()).map_err(|failure| format!("窓を出せなかった: {failure:?}"))?;
    let open_source =
        || Source::open(path).map_err(|failure| format!("ファイルを読めなかった: {failure:?}"));
    let format = opened.summary().format();
    let job = IndexJob::spawn(open_source()?, format, BlockSize::MEBIBYTE, ui.wakeup())
        .map_err(|failure| format!("索引の仕事を始められなかった: {failure:?}"))?;
    // 画面のスレッドが範囲を読むための原本は、索引の仕事とは別に開く（位置を指定して読むので共有しなくてよい）。
    let reader = reader::Reader::new(open_source()?, format, ui.wakeup());
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
        {
            let mut document = Document::open(
                opened.title().to_owned(),
                opened.summary(),
                opened.header(),
                opened.rows(),
            );
            if opened.looks_binary() {
                document.notify(nene_ledger_application::notice::Notice::LooksBinary);
            }
            document
        },
        workers::Workers::new(job, ticker),
        reader,
        preferences,
    );
    ui.run(&mut viewer)
        .map_err(|failure| format!("窓が止まった: {failure:?}"))
}

/// 開けなかった理由を、利用者に向けた文にする。🔴 表示言語の文言の表（ADR 0011）を入れるまでの仮の日本語。
fn describe(failure: &OpenFailure) -> String {
    match failure {
        OpenFailure::Scan(scan) => format!("ファイルを読めなかった: {scan:?}"),
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
            "使い方: nene-ledger [--theme <配色>] <CSV か TSV のファイル> | nene-ledger --dump <ファイル>",
        );
        None
    }
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
