//! 新しい表を新しい窓で開く（ADR 0028 の決定 1）。自分の exe を `--new` で起こし直す。

use std::process::Command;
use std::thread::Builder;

use crate::adapter_failure::AdapterFailure;

/// 子のプロセスの終わりを待つスレッドの名前。
const THREAD_NAME: &str = "nene-ledger-new-window";

/// 自分の exe を `--new` で起こす。今の窓は残る（Excel・メモ帳と同じ）。
///
/// 子の終わりは裏のスレッドで待つ（待たないと、子が閉じた後も親が閉じるまでゾンビとして残る）。親が先に閉じれば、
/// 子は init に引き取られる。
///
/// # Errors
///
/// 自分の exe の道筋を取れない・起こせない・待つスレッドを作れないとき。
pub fn open_new_window() -> Result<(), AdapterFailure> {
    let exe = std::env::current_exe()?;
    let mut child = Command::new(exe).arg("--new").spawn()?;
    let _waiter = Builder::new()
        .name(String::from(THREAD_NAME))
        .spawn(move || drop(child.wait()))?;
    Ok(())
}
