//! 文字コードで表せない字があって保存できなかった知らせ（SAVE-04・GitHub Issue #393）。

use nene_ledger_core::text_encoding::TextEncoding;

/// 書き換えた値に、ファイルの文字コードで表せない字があり、保存できなかった。どのセルか（表示の行と列・0 始まり）と、
/// その字のある書き換えたセルの数を言う（取り込み先で「?」になる前に直せるように・業界のペルソナの相談）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnrepresentableNotice {
    /// 表せない字（保存が最初に出会ったもの）。
    pub character: char,
    /// ファイルの文字コード。
    pub encoding: TextEncoding,
    /// 最初のセルの行（表示の位置）。
    pub line: u64,
    /// 最初のセルの列（表示の位置）。
    pub column: usize,
    /// その字のある、書き換えたセルの数。
    pub cells: u64,
}
