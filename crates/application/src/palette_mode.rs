//! パレットの段。

use crate::column_task::ColumnTask;
use crate::export_format::ExportFormat;
use crate::reading_aspect::ReadingAspect;
use crate::reading_choice::ReadingChoice;

/// パレットの段。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteMode {
    /// 操作の一覧。
    Commands,
    /// 飛ぶ先の行（と列）の番号を打つ段（Issue #54）。
    GoTo,
    /// 読み方の面 `aspect` を選ぶ段（Issue #52）。`current` は今の読み方（印を付け、Esc で戻す先）。
    Reading {
        /// 選ぶ面。
        aspect: ReadingAspect,
        /// 今の読み方。
        current: ReadingChoice,
    },
    /// キーの一覧（Issue #63）。
    Keys,
    /// 「このアプリについて」（版・著作権・ライセンス・ADR 0032）。
    About,
    /// 判別の候補を選ぶ段（Issue #3）。候補そのものは `CommandPalette` が持つ。`current` は今の読み方（印を付け、Esc で戻す先）。
    Candidates {
        /// 今の読み方（方言）。
        current: ReadingChoice,
    },
    /// 名前を付けて保存の保存先の道筋を打つ段（X11・ADR 0028 の決定 4）。`overwrite` は、打った道筋のファイルが既にあり、
    /// 上書きしてよいかを尋ねている間（もう一度 Enter で上書き・決定 5）。
    SaveAs {
        /// 上書きを尋ねているか。
        overwrite: bool,
        /// 書き出す先を打っているなら、その形式（ADR 0036 の決定 2）。名前を付けて保存なら `None`。
        export: Option<ExportFormat>,
    },
    /// 書き出しの形式を選ぶ段（ADR 0036 の決定 1）。`format` は今選んでいる形式（初めは今のファイルの形式）。
    Export {
        /// 選んでいる形式。
        format: ExportFormat,
    },
    /// 列を分ける区切り・つなぐ区切りを打つ段（ADR 0037 の決定 1・4）。候補の所に、見えている行の分け方の見本を出す
    /// （見本そのものは `CommandPalette` が持つ）。
    Columns {
        /// 仕事（分割なら分け方の種類・結合）。
        task: ColumnTask,
    },
    /// 探す文字を打つ段（Ctrl+F・ADR 0030 の決定 1）。条件は `CommandPalette::find_options`。
    Find,
    /// 探す文字と置き換える文字を打つ段（Ctrl+H・ADR 0030 の決定 6）。`with` は置き換える欄を打っている間（Tab で替える）。
    /// 置き換える文字は `CommandPalette::replacement`。
    Replace {
        /// 置き換える欄を打っているか。
        with: bool,
    },
    /// 配色の一覧。`original` は開いたときの配色の、表の中の位置（Esc で戻す先）。
    Themes {
        /// 開いたときの配色の位置。
        original: usize,
    },
}

impl PaletteMode {
    /// 選んでいる候補をその場で試す段か（配色・読み方・判別の候補）。試した表が見えるように、窓の下へ寄せる。
    #[must_use]
    pub fn tries_the_selection(self) -> bool {
        match self {
            Self::Themes { .. } | Self::Reading { .. } | Self::Candidates { .. } => true,
            Self::Commands
            | Self::GoTo
            | Self::Find
            | Self::Replace { .. }
            | Self::Keys
            | Self::About
            | Self::Export { .. }
            | Self::Columns { .. }
            | Self::SaveAs { .. } => false,
        }
    }

    /// 欄の下に案内の行を持つ段か（その場で試す段と、値を替える書き出しの形式の段）。
    #[must_use]
    pub fn has_hint(self) -> bool {
        self.tries_the_selection() || matches!(self, Self::Export { .. } | Self::Columns { .. })
    }
}
