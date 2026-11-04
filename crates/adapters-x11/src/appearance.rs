//! OS の明暗（ADR 0012 の決定 5）。
//!
//! X11 には標準の観測が無いので GTK の慣習から読む: 環境変数 `GTK_THEME` が `:dark` / `:light` で終わればそれ、無ければ
//! `gtk-3.0/settings.ini` の `gtk-application-prefer-dark-theme`。どちらも無ければ分からない（`system` はライト）。起動時に 1 度だけ。
//! `NeNeDocket` の同じ名前のモジュールを写した。

use std::env;
use std::fs;

use nene_ledger_core::appearance::Appearance;

use crate::config_directory;

/// 環境変数。`GTK_THEME=Adwaita:dark` の形。
const VARIABLE: &str = "GTK_THEME";
/// GTK 3 の設定のファイル（設定の根から）。
const SETTINGS_PATH: [&str; 2] = ["gtk-3.0", "settings.ini"];
/// 設定のファイルの鍵。
const PREFER_DARK: &str = "gtk-application-prefer-dark-theme";

/// OS の明暗。`GTK_THEME`、無ければ `settings.ini`、どちらも無ければ `None`。
#[must_use]
pub fn read() -> Option<Appearance> {
    env::var(VARIABLE)
        .ok()
        .and_then(|value| from_gtk_theme(&value))
        .or_else(|| {
            let path = config_directory::base()?
                .join(SETTINGS_PATH[0])
                .join(SETTINGS_PATH[1]);
            from_settings_ini(&fs::read_to_string(path).ok()?)
        })
}

/// `GTK_THEME` の値から。`:dark` で終わればダーク、`:light` ならライト、ほかは分からない。
#[must_use]
pub fn from_gtk_theme(value: &str) -> Option<Appearance> {
    if value.ends_with(":dark") {
        Some(Appearance::Dark)
    } else if value.ends_with(":light") {
        Some(Appearance::Light)
    } else {
        None
    }
}

/// `settings.ini` の文から。`gtk-application-prefer-dark-theme` が `1` / `true` ならダーク、`0` / `false` ならライト、無ければ分からない。
#[must_use]
pub fn from_settings_ini(text: &str) -> Option<Appearance> {
    text.lines()
        .filter_map(|line| line.split_once('='))
        .find(|(key, _)| key.trim() == PREFER_DARK)
        .and_then(|(_, value)| match value.trim() {
            "1" | "true" => Some(Appearance::Dark),
            "0" | "false" => Some(Appearance::Light),
            _ => None,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_gtk_theme_suffix_and_the_settings_key_decide() {
        assert_eq!(from_gtk_theme("Adwaita:dark"), Some(Appearance::Dark));
        assert_eq!(from_gtk_theme("Adwaita:light"), Some(Appearance::Light));
        assert_eq!(from_gtk_theme("Adwaita"), None);
        assert_eq!(
            from_settings_ini("[Settings]\ngtk-application-prefer-dark-theme=1\n"),
            Some(Appearance::Dark)
        );
        assert_eq!(
            from_settings_ini("gtk-application-prefer-dark-theme = false\n"),
            Some(Appearance::Light)
        );
        assert_eq!(
            from_settings_ini("[Settings]\ngtk-theme-name=Adwaita\n"),
            None
        );
        assert_eq!(
            from_settings_ini("gtk-application-prefer-dark-theme=maybe\n"),
            None
        );
    }
}
