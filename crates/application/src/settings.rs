//! 設定。

use nene_ledger_core::find_options::FindOptions;
use nene_ledger_core::theme_choice::ThemeChoice;

use crate::detail_pane::DetailPane;
use crate::zoom_level::ZoomLevel;

/// 起動をまたいで残す設定（ARC-009）。配色の希望・表と詳細ペインの文字の大きさ・詳細ペインの開閉・検索の条件
/// （ADR 0030 の決定 3・GitHub Issue #213）。
/// 形（`settings_file`）と読み書き（アダプタ）は別に持つ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    theme: ThemeChoice,
    zoom: ZoomLevel,
    detail: DetailPane,
    find: FindOptions,
}

impl Settings {
    /// 初めて起動したときの設定。配色は OS の明暗に従い、文字は 100%、詳細ペインは開く。検索の条件は既定（ADR 0030 の
    /// 決定 3）。
    #[must_use]
    pub fn initial() -> Self {
        Self {
            theme: ThemeChoice::System,
            zoom: ZoomLevel::hundred(),
            detail: DetailPane::Open,
            find: FindOptions::standard(),
        }
    }

    /// 配色の希望。
    #[must_use]
    pub fn theme(self) -> ThemeChoice {
        self.theme
    }

    /// 文字の大きさ。
    #[must_use]
    pub fn zoom(self) -> ZoomLevel {
        self.zoom
    }

    /// 詳細ペインの開閉。
    #[must_use]
    pub fn detail(self) -> DetailPane {
        self.detail
    }

    /// 検索の条件。
    #[must_use]
    pub fn find(self) -> FindOptions {
        self.find
    }

    /// 検索の条件を替えた設定。
    #[must_use]
    pub fn with_find(self, find: FindOptions) -> Self {
        Self { find, ..self }
    }

    /// 配色の希望を替えた設定。
    #[must_use]
    pub fn with_theme(self, theme: ThemeChoice) -> Self {
        Self { theme, ..self }
    }

    /// 文字の大きさを替えた設定。
    #[must_use]
    pub fn with_zoom(self, zoom: ZoomLevel) -> Self {
        Self { zoom, ..self }
    }

    /// 詳細ペインの開閉を替えた設定。
    #[must_use]
    pub fn with_detail(self, detail: DetailPane) -> Self {
        Self { detail, ..self }
    }
}
