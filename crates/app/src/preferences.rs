//! 起動をまたいで残す見た目（配色・文字の大きさ・詳細ペインの開閉）。読み・解決し・書く。

#[cfg(windows)]
use nene_ledger_adapters_windows::appearance;
#[cfg(windows)]
use nene_ledger_adapters_windows::saved_settings::SettingsStore;
#[cfg(not(windows))]
use nene_ledger_adapters_x11::appearance;
#[cfg(not(windows))]
use nene_ledger_adapters_x11::saved_settings::SettingsStore;
use nene_ledger_application::detail_pane::DetailPane;
use nene_ledger_application::settings::Settings;
use nene_ledger_application::settings_failure::SettingsFailure;
use nene_ledger_application::settings_port::SettingsPort;
use nene_ledger_application::theme_state::ThemeState;
use nene_ledger_application::theme_step::ThemeStep;
use nene_ledger_application::zoom_level::ZoomLevel;
use nene_ledger_core::find_options::FindOptions;
use nene_ledger_core::theme_choice::ThemeChoice;

/// 起動をまたいで残す見た目。設定（application）と、今の配色と、書く先（adapters-x11）を結ぶだけ（ARC-006）。
///
/// 設定のファイルが読めない形なら、書く先を持たない（利用者のファイルを上書きしない）。そのときの変更は、この起動の間だけ。
#[derive(Debug)]
pub(crate) struct Preferences {
    settings: Settings,
    theme: ThemeState,
    store: Option<SettingsStore>,
}

impl Preferences {
    /// 設定を読み、OS の明暗と高コントラストを 1 度観測して配色を決める（ADR 0012 の決定 5・A11Y-02）。`override_theme` は `--theme` の指定で、
    /// この起動の間だけ設定より優先する（設定は変えない）。設定を読めなければ初期値で始め、利用者に知らせる文を返す。
    pub(crate) fn load(override_theme: Option<ThemeState>) -> (Self, Option<String>) {
        let (settings, store, warning) = match SettingsStore::in_config_directory() {
            Ok(store) => match store.load() {
                Ok(settings) => (settings, Some(store), None),
                Err(failure) => (
                    Settings::initial(),
                    None,
                    Some(format!(
                        "設定を読めなかったので既定で開く（設定のファイルは書き換えない）: {failure:?}"
                    )),
                ),
            },
            Err(failure) => (
                Settings::initial(),
                None,
                Some(format!("設定の置き場が無いので保存しない: {failure:?}")),
            ),
        };
        let theme = override_theme.unwrap_or_else(|| {
            ThemeState::of(
                settings
                    .theme()
                    .resolve(appearance::read(), appearance::high_contrast()),
            )
        });
        (
            Self {
                settings,
                theme,
                store,
            },
            warning,
        )
    }

    /// 今の配色。
    pub(crate) fn theme(&self) -> ThemeState {
        self.theme
    }

    /// 残した設定。
    pub(crate) fn settings(&self) -> Settings {
        self.settings
    }

    /// 配色を `step` の向きに替えて残す（希望は替えた名前になる）。
    pub(crate) fn step_theme(&mut self, step: ThemeStep) -> Result<(), SettingsFailure> {
        self.theme = self.theme.stepped(step);
        self.settings = self
            .settings
            .with_theme(ThemeChoice::Named(self.theme.seed()));
        self.save()
    }

    /// 配色 `theme` をその場で試す（設定は変えない）。パレットの配色の段で選んだとき・Esc で戻すとき。
    pub(crate) fn preview_theme(&mut self, theme: ThemeState) {
        self.theme = theme;
    }

    /// 配色 `theme` に決めて残す。
    pub(crate) fn commit_theme(&mut self, theme: ThemeState) -> Result<(), SettingsFailure> {
        self.theme = theme;
        self.settings = self.settings.with_theme(ThemeChoice::Named(theme.seed()));
        self.save()
    }

    /// 文字の大きさを残す。
    pub(crate) fn set_zoom(&mut self, zoom: ZoomLevel) -> Result<(), SettingsFailure> {
        self.settings = self.settings.with_zoom(zoom);
        self.save()
    }

    /// 詳細ペインの開閉を残す。変わっていなければ書かない。
    pub(crate) fn set_detail(&mut self, detail: DetailPane) -> Result<(), SettingsFailure> {
        if self.settings.detail() == detail {
            return Ok(());
        }
        self.settings = self.settings.with_detail(detail);
        self.save()
    }

    /// 検索の条件を替えて書く（変わらなければ書かない・ADR 0030 の決定 3・GitHub Issue #213）。
    pub(crate) fn set_find(&mut self, find: FindOptions) -> Result<(), SettingsFailure> {
        if self.settings.find() == find {
            return Ok(());
        }
        self.settings = self.settings.with_find(find);
        self.save()
    }

    fn save(&mut self) -> Result<(), SettingsFailure> {
        match &mut self.store {
            Some(store) => store.save(&self.settings),
            None => Err(SettingsFailure::Unwritable),
        }
    }
}
