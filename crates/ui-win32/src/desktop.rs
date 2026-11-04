//! 窓の外の Windows の画面の部品（Issue #56）。ファイルを選ぶ画面・失敗を知らせるメッセージの窓・親のコンソールへの結び付け。

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;

use windows::Win32::System::Console::{ATTACH_PARENT_PROCESS, AttachConsole};
use windows::Win32::UI::Controls::Dialogs::{
    GetOpenFileNameW, OFN_EXPLORER, OFN_FILEMUSTEXIST, OFN_PATHMUSTEXIST, OPENFILENAMEW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    IDYES, MB_DEFBUTTON2, MB_ICONERROR, MB_ICONWARNING, MB_OK, MB_YESNO, MessageBoxW,
};
use windows::core::{HSTRING, PWSTR, w};

/// 選ぶファイルの道の長さの上限（UTF-16 の単位・長い道も入る）。
const PATH_LIMIT: usize = 32_768;

/// 「ファイルを開く」画面で、開くファイルを選ばせる（CSV・TSV・テキスト・すべてのファイル）。やめれば `None`。
#[must_use]
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の GetOpenFileNameW を呼ぶ（ADR 0002 の 4）"
)]
pub fn choose_file() -> Option<PathBuf> {
    let mut buffer = vec![0_u16; PATH_LIMIT];
    let mut dialog = OPENFILENAMEW {
        lStructSize: u32::try_from(size_of::<OPENFILENAMEW>()).ok()?,
        lpstrFilter: w!(
            "CSV・TSV・テキスト (*.csv;*.tsv;*.txt)\0*.csv;*.tsv;*.txt\0すべてのファイル (*.*)\0*.*\0"
        ),
        lpstrFile: PWSTR(buffer.as_mut_ptr()),
        nMaxFile: u32::try_from(buffer.len()).ok()?,
        lpstrTitle: w!("開くファイルを選ぶ — NeNe Ledger"),
        Flags: OFN_EXPLORER | OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST,
        ..OPENFILENAMEW::default()
    };
    // SAFETY: dialog はこの関数の局所で、選んだ道を buffer（長さ nMaxFile）に書く。どちらも呼び出しの間生きている。
    let chosen = unsafe { GetOpenFileNameW(&raw mut dialog) }.as_bool();
    if !chosen {
        return None;
    }
    let length = buffer
        .iter()
        .position(|&unit| unit == 0)
        .unwrap_or(buffer.len());
    let path = OsString::from_wide(buffer.get(..length)?);
    Some(PathBuf::from(path))
}

/// 失敗を知らせるメッセージの窓（コンソールの無い Windows の exe で、利用者に見える唯一の知らせ）。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の MessageBoxW を呼ぶ（ADR 0002 の 4）"
)]
pub fn show_failure(message: &str) {
    let text = HSTRING::from(message);
    // SAFETY: 文と題名はこの関数の間生きている。持ち主の窓は無い。
    let _answer = unsafe { MessageBoxW(None, &text, w!("NeNe Ledger"), MB_OK | MB_ICONERROR) };
}

/// はい / いいえで尋ねるメッセージの窓（既定は「いいえ」）。「はい」なら `true`。
#[must_use]
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の MessageBoxW を呼ぶ（ADR 0002 の 4）"
)]
pub fn confirm(message: &str) -> bool {
    let text = HSTRING::from(message);
    // SAFETY: 文と題名はこの関数の間生きている。持ち主の窓は無い。
    let answer = unsafe {
        MessageBoxW(
            None,
            &text,
            w!("NeNe Ledger"),
            MB_YESNO | MB_ICONWARNING | MB_DEFBUTTON2,
        )
    };
    answer == IDYES
}

/// 親のコンソール（起動したコマンドプロンプト）に結ぶ。`--dump` の書き出しをそこへ出す。親が無ければ何もしない。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の AttachConsole を呼ぶ（ADR 0002 の 4）"
)]
pub fn attach_console() {
    // SAFETY: 値だけを渡す。親が無い・既に結んでいれば失敗するだけ。
    let _attached: Result<(), _> = unsafe { AttachConsole(ATTACH_PARENT_PROCESS) };
}
