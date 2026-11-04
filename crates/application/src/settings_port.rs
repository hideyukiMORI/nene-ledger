//! 設定を残す口。

use crate::settings::Settings;
use crate::settings_failure::SettingsFailure;

/// 設定を残す口（port）。読み書きはアダプタが行い、application は形（`settings_file`）と中身だけを知る。
pub trait SettingsPort {
    /// 設定を丸ごと書く。
    ///
    /// # Errors
    ///
    /// 設定の置き場に書けないとき。
    fn save(&mut self, settings: &Settings) -> Result<(), SettingsFailure>;
}
