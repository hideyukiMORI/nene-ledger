//! パレットの候補。

use nene_ledger_core::dialect_candidate::DialectCandidate;

use crate::command_id::CommandId;
use crate::key_guide::KeyGuide;
use crate::reading_choice::ReadingChoice;

/// パレットの候補 1 つ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteItem {
    /// 操作。
    Command(CommandId),
    /// 配色（組み込みの表の中の位置）。
    Theme(usize),
    /// 読み方の候補（Issue #52）。
    Reading(ReadingChoice),
    /// パレットに無いキーの操作（キーの一覧の後半・読むだけ・Issue #63）。
    Key(KeyGuide),
    /// 判別の候補の方言（点数の順・Issue #3）。
    Candidate(DialectCandidate),
}
