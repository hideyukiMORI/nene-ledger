//! 保存の失敗（ADR 0025 の決定 3）。

use std::io::ErrorKind;
use std::path::PathBuf;

use nene_ledger_application::replace_failure::ReplaceFailure;

/// 保存の失敗。🔴 どの失敗でも原本は書いていない。置き換えの失敗だけは一時ファイルが残っている。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveFailure {
    /// 書き換えた値に、原本の文字コードで表せない字がある（SAVE-04）。値をそのまま持つ。
    Unrepresentable(String),
    /// 原本の道筋を解けない・原本の情報を読めない。
    Target(ErrorKind),
    /// 原本にハードリンクがある（リンクの数）。置き換えるとリンクが切れるので、利用者に選ばせる（施主の判断 2）。
    HardLinked(u64),
    /// 一時ファイルを作れない・書けない・同期できない（一時ファイルは消した）。
    Write(ErrorKind),
    /// 原本を読めない（一時ファイルは消した）。
    Read(ErrorKind),
    /// 原本が計画より短い。外から変えられた（一時ファイルは消した）。
    SourceShrank,
    /// 置き換えられなかった。一時ファイルは `temporary` に残っている（再試行・別名で保存・やめるを選ばせる）。
    Replace {
        /// 失敗の種類。
        failure: ReplaceFailure,
        /// 残っている一時ファイル。
        temporary: PathBuf,
    },
}
