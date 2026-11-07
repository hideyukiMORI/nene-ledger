//! 設定のファイルの形（ARC-009）。
//!
//! 版 2: `version = 2`・`theme = system|<名前>`・`zoom = <百分率>`・`detail = open|closed`・`find = <区別する条件の並び>`
//! （`case`・`whole`・`width`・`kana`・`regex` を空白で区切る・ADR 0030 の決定 3・GitHub Issue #213）。1 行に 1 つの
//! `鍵 = 値`。版 1（`find` の無い版）も読む。
//! 形はここ（純関数）、読み書きはアダプタ。知らない版・知らない鍵・読めない値・同じ鍵の重なりは `None` で、黙って既定値へ
//! 落とさない。無い鍵は初期値で補う。

use alloc::format;
use alloc::string::String;

use nene_ledger_core::find_options::FindOptions;
use nene_ledger_core::theme_choice::ThemeChoice;

use crate::detail_pane::DetailPane;
use crate::settings::Settings;
use crate::zoom_level::ZoomLevel;

/// いま書く版。
const VERSION: &str = "2";
/// 読める古い版（`find` の鍵が無い）。
const OLD_VERSION: &str = "1";

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
    let find: alloc::vec::Vec<&str> = FIND_FLAGS
        .iter()
        .filter(|(_, on, _)| on(settings.find()))
        .map(|(name, _, _)| *name)
        .collect();
    format!(
        "version = {VERSION}\ntheme = {}\nzoom = {}\ndetail = {detail}\nfind = {}\n",
        settings.theme().keyword(),
        settings.zoom().percent(),
        find.join(" ")
    )
}

/// 検索の条件の鍵の言葉・入っているか・入れ替える関数。
type FindFlag = (
    &'static str,
    fn(FindOptions) -> bool,
    fn(FindOptions) -> FindOptions,
);

/// `find` の鍵に書く、区別する条件（入っている条件の名前を並べる）。
const FIND_FLAGS: [FindFlag; 5] = [
    ("case", FindOptions::match_case, FindOptions::toggled_case),
    (
        "whole",
        FindOptions::whole_cell,
        FindOptions::toggled_whole_cell,
    ),
    (
        "width",
        FindOptions::match_width,
        FindOptions::toggled_width,
    ),
    ("kana", FindOptions::match_kana, FindOptions::toggled_kana),
    ("regex", FindOptions::regex, FindOptions::toggled_regex),
];

/// `find` の値（条件の名前の並び）を読む。知らない名前・同じ名前の重なりは `None`。
fn parse_find(value: &str) -> Option<FindOptions> {
    let names: alloc::vec::Vec<&str> = value.split_whitespace().collect();
    for (index, name) in names.iter().enumerate() {
        let known = FIND_FLAGS.iter().any(|(flag, _, _)| flag == name);
        if !known || names.get(..index)?.contains(name) {
            return None;
        }
    }
    Some(
        FIND_FLAGS
            .iter()
            .fold(FindOptions::standard(), |options, (name, on, toggle)| {
                if on(options) == names.contains(name) {
                    options
                } else {
                    toggle(options)
                }
            }),
    )
}

/// 読み取り中の値。鍵ごとに、まだ読んでいなければ `None`。
struct Reading {
    version: Option<()>,
    theme: Option<ThemeChoice>,
    zoom: Option<ZoomLevel>,
    detail: Option<DetailPane>,
    find: Option<FindOptions>,
}

impl Reading {
    fn default_values() -> Self {
        Self {
            version: None,
            theme: None,
            zoom: None,
            detail: None,
            find: None,
        }
    }

    /// 1 つの `鍵 = 値` を取り込む。知らない鍵・読めない値・2 度目の鍵は `None`。
    fn take(&mut self, key: &str, value: &str) -> Option<()> {
        match key {
            "version" if self.version.is_none() && (value == VERSION || value == OLD_VERSION) => {
                self.version = Some(());
            }
            "find" if self.find.is_none() => self.find = Some(parse_find(value)?),
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
                .with_detail(self.detail.unwrap_or(initial.detail()))
                .with_find(self.find.unwrap_or(initial.find())),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zoom_step::ZoomStep;
    use nene_ledger_core::find_options::FindOptions;

    #[test]
    fn settings_round_trip() {
        let settings = Settings::initial()
            .with_theme(ThemeChoice::from_keyword("solarized-dark").unwrap())
            .with_zoom(ZoomLevel::hundred().stepped(ZoomStep::In))
            .with_detail(DetailPane::Closed);
        let text = encode(&settings);
        assert_eq!(
            text,
            "version = 2\ntheme = solarized-dark\nzoom = 110\ndetail = closed\nfind = kana\n"
        );
        assert_eq!(parse(&text), Some(settings));
        assert_eq!(
            parse(&encode(&Settings::initial())),
            Some(Settings::initial())
        );
    }

    /// 検索の条件は区別する条件の名前の並びで書き、読み戻すと同じ。版 1 のファイル（`find` の無い版）は既定の条件。
    /// 空の並びは「どれも区別しない」（GitHub Issue #213・ADR 0030 の決定 3）。
    #[test]
    fn find_options_round_trip_and_version_one_files_still_read() {
        let standard = FindOptions::standard();
        let every = [
            standard,
            standard.toggled_case().toggled_regex(),
            standard.toggled_kana(),
            standard.toggled_width().toggled_whole_cell(),
        ];
        for find in every {
            let settings = Settings::initial().with_find(find);
            assert_eq!(parse(&encode(&settings)), Some(settings), "{find:?}");
        }
        let none = parse("version = 2\nfind =").map(Settings::find);
        assert_eq!(none, Some(standard.toggled_kana()));
        let old = parse("version = 1\ntheme = system\nzoom = 100\ndetail = open\n");
        assert_eq!(old, Some(Settings::initial()));
        assert_eq!(
            parse("version = 2\nfind = regex case kana").map(Settings::find),
            Some(standard.toggled_regex().toggled_case())
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
            "version = 3\n",
            "version = 2\nfind = case case\n",
            "version = 2\nfind = loose\n",
            "version = 2\nfind = kana\nfind = case\n",
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
