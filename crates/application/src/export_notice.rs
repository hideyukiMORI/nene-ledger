//! 書き出しの知らせ（ADR 0036）。

use nene_ledger_core::text_encoding::TextEncoding;

use crate::export_format::ExportFormat;
use crate::save_problem::SaveProblem;

/// 書き出しの知らせ。どれも、今のファイルと窓の書き換えは変えていない。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportNotice {
    /// 書き出した。形式を言う。
    Written(ExportFormat),
    /// 書き出しの途中なので、直すことと保存を受けない。
    Busy,
    /// 選んだ文字コードで表せない字のあるセルがあるので、書き出さなかった（何も書いていない・決定 5）。
    Unrepresentable {
        /// 選んだ文字コード。
        encoding: TextEncoding,
        /// セルの数。
        cells: u64,
    },
    /// 書き出す先が今のファイルと同じなので、書き出さなかった（決定 2）。
    SameFile,
    /// 書き出せなかった（理由は保存と同じ分け方）。
    Failed(SaveProblem),
}
