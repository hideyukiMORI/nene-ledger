//! `ReplaceFileW` による置き換え（ADR 0025 の決定 3・4・施主の判断 1・3）。

use std::path::Path;
use std::time::Duration;

use nene_ledger_application::replace_failure::ReplaceFailure;
use nene_ledger_application::replacing::Replacing;
use windows::Win32::Foundation::{
    ERROR_ACCESS_DENIED, ERROR_LOCK_VIOLATION, ERROR_SHARING_VIOLATION,
    ERROR_UNABLE_TO_MOVE_REPLACEMENT, ERROR_UNABLE_TO_MOVE_REPLACEMENT_2,
    ERROR_UNABLE_TO_REMOVE_REPLACED, WIN32_ERROR,
};
use windows::Win32::Storage::FileSystem::{REPLACEFILE_IGNORE_MERGE_ERRORS, ReplaceFileW};
use windows::core::{HRESULT, HSTRING, PCWSTR};

/// 最初の再試行までの待ち。倍にしながら `RETRIES` 回まで試す（合わせて約 1.6 秒・施主の判断 3）。
const FIRST_WAIT: Duration = Duration::from_millis(50);
/// 再試行の回数。
const RETRIES: u32 = 5;

/// `ReplaceFileW` で置き換える。原本の ACL・属性・名前つきストリームを引き継ぐ（`WRITE_DAC` が無ければ ACL は
/// 引き継がずに成功する）。共有違反・ロックの違反・アクセスの拒否・1175 は、ウイルス対策や検索の索引付けが一時的に
/// 掴んでいることが多いので、待って再試行する。🔴 Windows の実機での確かめはまだ（QLT-013）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileReplacing;

impl Replacing for FileReplacing {
    type Location = Path;

    fn replace(&self, replacement: &Path, target: &Path) -> Result<(), ReplaceFailure> {
        let replaced = HSTRING::from(target);
        let replacing = HSTRING::from(replacement);
        let mut wait = FIRST_WAIT;
        let mut attempt = 0_u32;
        loop {
            let Err(code) = replace_once(&replaced, &replacing) else {
                return Ok(());
            };
            let failure = failure_of(code);
            if failure != ReplaceFailure::Locked || attempt == RETRIES {
                return Err(failure);
            }
            std::thread::sleep(wait);
            wait = wait.saturating_mul(2);
            attempt = attempt.saturating_add(1);
        }
    }
}

#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の ReplaceFileW を呼ぶ（ADR 0002 の 4・ADR 0025 の決定 4）"
)]
fn replace_once(replaced: &HSTRING, replacing: &HSTRING) -> Result<(), HRESULT> {
    // SAFETY: 2 つの名前は呼び出しの間生きている HSTRING（ヌルで終わる UTF-16）。backup は渡さない（NULL）。
    // lpexclude と lpreserved は文書どおり NULL。
    unsafe {
        ReplaceFileW(
            replaced,
            replacing,
            PCWSTR::null(),
            REPLACEFILE_IGNORE_MERGE_ERRORS,
            None,
            None,
        )
    }
    .map_err(|error| error.code())
}

/// 失敗の種類。再試行する価値のあるもの（一時的なロック）は `Locked`。
fn failure_of(code: HRESULT) -> ReplaceFailure {
    let is = |error: WIN32_ERROR| code == HRESULT::from_win32(error.0);
    if is(ERROR_SHARING_VIOLATION)
        || is(ERROR_LOCK_VIOLATION)
        || is(ERROR_ACCESS_DENIED)
        || is(ERROR_UNABLE_TO_REMOVE_REPLACED)
    {
        ReplaceFailure::Locked
    } else if is(ERROR_UNABLE_TO_MOVE_REPLACEMENT) || is(ERROR_UNABLE_TO_MOVE_REPLACEMENT_2) {
        ReplaceFailure::Interrupted
    } else {
        ReplaceFailure::Other
    }
}
