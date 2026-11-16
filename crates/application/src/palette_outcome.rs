//! パレットに意図を渡した結果、合成ルートがすること。

use crate::column_task::ColumnTask;
use crate::export_format::ExportFormat;
use crate::intent::Intent;
use crate::reading_aspect::ReadingAspect;
use crate::reading_choice::ReadingChoice;

/// パレットに意図を渡した結果、合成ルートがすること。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteOutcome {
    /// 開いたまま（描き直すだけ）。
    Stay,
    /// 閉じる。
    Close,
    /// 閉じて、この意図を実行する。
    Run(Intent),
    /// 開いたまま、この位置の配色をその場で試す（設定は変えない）。
    Preview(usize),
    /// 閉じて、この位置の配色に決める（設定に残す）。
    Commit(usize),
    /// 閉じて、この位置の配色（開いたときの配色）に戻す。
    Revert(usize),
    /// 読み方の面を選ぶ段を開く（合成ルートが今の読み方を渡して `CommandPalette::enter_reading` を呼ぶ）。
    OpenReading(ReadingAspect),
    /// 候補の読み方で、先頭の行を読み直して試す（保存しない）。
    PreviewReading(ReadingChoice),
    /// 候補の読み方で開き直す（索引を作り直す）。
    CommitReading(ReadingChoice),
    /// 試した読み方をやめて元に戻す。
    RevertReading,
    /// 判別の候補を選ぶ段を開く（合成ルートが今の文字コードで候補を作り `CommandPalette::enter_candidates` を呼ぶ・Issue #3）。
    OpenCandidates,
    /// 打った道筋に名前を付けて保存する（合成ルートが `CommandPalette::query` を読む）。`overwrite` は、上書きを尋ねて
    /// もう一度 Enter を受けたか。`false` で道筋のファイルが既にあれば、合成ルートは `CommandPalette::ask_overwrite` を呼ぶ。
    SaveAs {
        /// 上書きを確かめたか。
        overwrite: bool,
        /// 書き出す先なら、その形式（ADR 0036 の決定 2）。名前を付けて保存なら `None`。
        export: Option<ExportFormat>,
    },
    /// 書き出しの形式を選ぶ段を開く（合成ルートが今の形式を渡して `CommandPalette::enter_export` を呼ぶ・ADR 0036）。
    OpenExport,
    /// 閉じて、この形式で書き出す先を選ばせる（ADR 0036 の決定 2）。
    Export(ExportFormat),
    /// 列の分割か結合の段を開く（合成ルートが、できるかを確かめて `CommandPalette::enter_columns` を呼ぶ・ADR 0037）。
    OpenColumns(ColumnTask),
    /// 打った文字列で、列を分ける・つなぐ（合成ルートが `CommandPalette::query` を読む）。
    Columns(ColumnTask),
    /// 開いたまま、打った文字を今の条件で探す（合成ルートが `CommandPalette::query` と `find_options` を読む）。
    /// `forward` なら次（Enter・F3）、そうでなければ前（Shift+Enter・Shift+F3）。
    Find {
        /// 次へ探すか。
        forward: bool,
    },
    /// 開いたまま、カーソルのセルを置き換えて次を探す（`all` なら閉じて全部を数える）。合成ルートが
    /// `CommandPalette::query`・`replacement`・`find_options` を読む（ADR 0030 の決定 6）。
    Replace {
        /// 全部を置き換えるか（Alt+A）。
        all: bool,
    },
}
