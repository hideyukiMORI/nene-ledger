//! 設定のファイルの形（ARC-009）。
//!
//! 版 1: `version = 1`・`theme = system|<名前>`・`zoom = <百分率>`・`detail = open|closed`。1 行に 1 つの `鍵 = 値`。
//! 形はここ（純関数）、読み書きはアダプタ。知らない版・知らない鍵・読めない値・同じ鍵の重なりは `None` で、黙って既定値へ
//! 落とさない。無い鍵は初期値で補う。

use alloc::format;
use alloc::string::String;

use nene_ledger_core::theme_choice::ThemeChoice;

use crate::detail_pane::DetailPane;
use crate::settings::Settings;
use crate::zoom_level::ZoomLevel;

/// いま書く版。
const VERSION: &str = "1";

/// 文字列から設定を読む。
#[must_use]
pub fn parse(text: &str) -> Option<Settings> {
    let mut reading = Reading::default_values();
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
        let (key, value) = line.split_once('=')?;
        reading.take(key.trim(), value.trim())?;
    }
    reading.into_settings()
}

/// 設定を文字列に書く（いまの版）。
#[must_use]
pub fn encode(settings: &Settings) -> String {
    let detail = match settings.detail() {
        DetailPane::Open => "open",
        DetailPane::Closed => "closed",
    };
    format!(
        "version = {VERSION}\ntheme = {}\nzoom = {}\ndetail = {detail}\n",
        settings.theme().keyword(),
        settings.zoom().percent()
    )
}

/// 読み取り中の値。鍵ごとに、まだ読んでいなければ `None`。
struct Reading {
    version: Option<()>,
    theme: Option<ThemeChoice>,
    zoom: Option<ZoomLevel>,
    detail: Option<DetailPane>,
}

impl Reading {
    fn default_values() -> Self {
        Self {
            version: None,
            theme: None,
            zoom: None,
            detail: None,
        }
    }

    /// 1 つの `鍵 = 値` を取り込む。知らない鍵・読めない値・2 度目の鍵は `None`。
    fn take(&mut self, key: &str, value: &str) -> Option<()> {
        match key {
            "version" if self.version.is_none() && value == VERSION => self.version = Some(()),
            "theme" if self.theme.is_none() => self.theme = Some(ThemeChoice::from_keyword(value)?),
            "zoom" if self.zoom.is_none() => {
                self.zoom = Some(ZoomLevel::from_percent(value.parse().ok()?)?);
            }
            "detail" if self.detail.is_none() => {
                self.detail = Some(match value {
                    "open" => DetailPane::Open,
                    "closed" => DetailPane::Closed,
                    _ => return None,
                });
            }
            _ => return None,
        }
        Some(())
    }

    /// 版が書いてあれば設定にする。無い鍵は初期値で補う。
    fn into_settings(self) -> Option<Settings> {
        self.version?;
        let initial = Settings::initial();
        Some(
            initial
                .with_theme(self.theme.unwrap_or(initial.theme()))
                .with_zoom(self.zoom.unwrap_or(initial.zoom()))
                .with_detail(self.detail.unwrap_or(initial.detail())),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zoom_step::ZoomStep;

    #[test]
    fn settings_round_trip() {
        let settings = Settings::initial()
            .with_theme(ThemeChoice::from_keyword("solarized-dark").unwrap())
            .with_zoom(ZoomLevel::hundred().stepped(ZoomStep::In))
            .with_detail(DetailPane::Closed);
        let text = encode(&settings);
        assert_eq!(
            text,
            "version = 1\ntheme = solarized-dark\nzoom = 110\ndetail = closed\n"
        );
        assert_eq!(parse(&text), Some(settings));
        assert_eq!(
            parse(&encode(&Settings::initial())),
            Some(Settings::initial())
        );
    }

    #[test]
    fn missing_keys_take_initial_values_and_blank_lines_are_ignored() {
        assert_eq!(parse("version = 1\n\n"), Some(Settings::initial()));
        assert_eq!(
            parse("version=1\nzoom=150")
                .map(Settings::zoom)
                .map(ZoomLevel::percent),
            Some(150)
        );
    }

    #[test]
    fn unreadable_files_are_refused_rather_than_defaulted() {
        for text in [
            "",
            "theme = system\n",
            "version = 2\n",
            "version = 1\ncolor = red\n",
            "version = 1\ntheme = no-such-theme\n",
            "version = 1\nzoom = 123\n",
            "version = 1\nzoom = big\n",
            "version = 1\ndetail = half\n",
            "version = 1\nzoom = 100\nzoom = 110\n",
            "version = 1\nnot a pair\n",
        ] {
            assert_eq!(parse(text), None, "{text:?}");
        }
    }
}
