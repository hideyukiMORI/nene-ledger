//! 窓の外の Windows の画面の部品（Issue #56）。ファイルを選ぶ画面・失敗を知らせるメッセージの窓・親のコンソールへの結び付け。

use std::ffi::OsString;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};

use windows::Win32::System::Console::{ATTACH_PARENT_PROCESS, AttachConsole};
use windows::Win32::UI::Controls::Dialogs::{
    GetOpenFileNameW, GetSaveFileNameW, OFN_EXPLORER, OFN_FILEMUSTEXIST, OFN_NOCHANGEDIR,
    OFN_OVERWRITEPROMPT, OFN_PATHMUSTEXIST, OPENFILENAMEW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    IDYES, MB_DEFBUTTON2, MB_ICONERROR, MB_ICONWARNING, MB_OK, MB_YESNO, MessageBoxW,
};
use windows::core::{HSTRING, PCWSTR, PWSTR, w};

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
    chosen.then(|| path_in(&buffer)).flatten()
}

/// 「名前を付けて保存」の画面で、保存先を選ばせる（ADR 0028 の決定 4）。初めはフォルダ `folder`・名前 `name`。
/// 既にあるファイルを選べば、OS の画面が上書きしてよいかを尋ねる（`OFN_OVERWRITEPROMPT`・決定 5）。やめれば `None`。
/// 拡張子を打たなければ `.csv` を付ける。
#[must_use]
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の GetSaveFileNameW を呼ぶ（ADR 0002 の 4・ADR 0028 の決定 4）"
)]
pub fn choose_save_file(folder: Option<&Path>, name: &str) -> Option<PathBuf> {
    let mut buffer = vec![0_u16; PATH_LIMIT];
    for (slot, unit) in buffer.iter_mut().zip(name.encode_utf16()) {
        *slot = unit;
    }
    let initial: Option<Vec<u16>> = folder.map(|folder| {
        folder
            .as_os_str()
            .encode_wide()
            .chain(core::iter::once(0_u16))
            .collect()
    });
    let mut dialog = OPENFILENAMEW {
        lStructSize: u32::try_from(size_of::<OPENFILENAMEW>()).ok()?,
        lpstrFilter: w!(
            "CSV (*.csv)\0*.csv\0TSV・テキスト (*.tsv;*.txt)\0*.tsv;*.txt\0すべてのファイル (*.*)\0*.*\0"
        ),
        lpstrFile: PWSTR(buffer.as_mut_ptr()),
        nMaxFile: u32::try_from(buffer.len()).ok()?,
        lpstrInitialDir: initial
            .as_ref()
            .map_or(PCWSTR::null(), |initial| PCWSTR(initial.as_ptr())),
        lpstrTitle: w!("名前を付けて保存 — NeNe Ledger"),
        lpstrDefExt: w!("csv"),
        Flags: OFN_EXPLORER | OFN_OVERWRITEPROMPT | OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR,
        ..OPENFILENAMEW::default()
    };
    // SAFETY: dialog はこの関数の局所。初めの名前と選んだ道は buffer（長さ nMaxFile・ヌルで終わる）、初めのフォルダは
    // initial（ヌルで終わる）。どれも呼び出しの間生きている。
    let chosen = unsafe { GetSaveFileNameW(&raw mut dialog) }.as_bool();
    chosen.then(|| path_in(&buffer)).flatten()
}

/// 画面が `buffer` に書いた道（最初のヌルまで）。
fn path_in(buffer: &[u16]) -> Option<PathBuf> {
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
