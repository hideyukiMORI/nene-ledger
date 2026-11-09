//! 設定のファイルの形（ARC-009）。
//!
//! 版 2: `version = 2`・`theme = system|<名前>`・`zoom = <百分率>`・`detail = open|closed`・`find = <区別する条件の並び>`
//! （`case`・`whole`・`width`・`kana`・`regex` を空白で区切る・ADR 0030 の決定 3・GitHub Issue #213）。1 行に 1 つの
//! `鍵 = 値`。版 1（`find` の無い版）も読む。
//! 版 3: 版 2 に `detail_lines = <詳細ペインの本文の行の数>` を足す（GitHub Issue #452）。既定の行の数なら版 2 以下で書く
//! （古い exe でも読める）。
//! 形はここ（純関数）、読み書きはアダプタ。知らない版・知らない鍵・読めない値・同じ鍵の重なりは `None` で、黙って既定値へ
//! 落とさない。無い鍵は初期値で補う。

use alloc::format;
use alloc::string::String;

use nene_ledger_core::find_options::FindOptions;
use nene_ledger_core::theme_choice::ThemeChoice;

use crate::client_bands::{DETAIL_LINES, MAX_DETAIL_LINES, MIN_DETAIL_LINES};
use crate::detail_pane::DetailPane;
use crate::settings::Settings;
use crate::zoom_level::ZoomLevel;

/// 詳細ペインの行の数まで書く版（GitHub Issue #452）。
const LINES_VERSION: &str = "3";
/// 検索の条件まで書く版。
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
    let head = format!(
        "theme = {}\nzoom = {}\ndetail = {detail}\n",
        settings.theme().keyword(),
        settings.zoom().percent()
    );
    // 検索の条件と詳細ペインの行の数が既定なら版 1 のまま書く（版 1 しか読めない古い exe でも、配色と文字の大きさを失わない）。
    if settings.find() == FindOptions::standard() && settings.detail_lines() == DETAIL_LINES {
        return format!("version = {OLD_VERSION}\n{head}");
    }
    let find: alloc::vec::Vec<&str> = FIND_FLAGS
        .iter()
        .filter(|(_, on, _)| on(settings.find()))
        .map(|(name, _, _)| *name)
        .collect();
    let find = if find.is_empty() {
        String::from("find =")
    } else {
        format!("find = {}", find.join(" "))
    };
    if settings.detail_lines() == DETAIL_LINES {
        return format!("version = {VERSION}\n{head}{find}\n");
    }
    format!(
        "version = {LINES_VERSION}\n{head}{find}\ndetail_lines = {}\n",
        settings.detail_lines()
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

/// まだ読んでいない鍵 `slot` に、読めた値 `value` を入れる。2 度目の鍵か、読めない値なら `None`。
fn once<T>(slot: &mut Option<T>, value: Option<T>) -> Option<()> {
    if slot.is_some() {
        return None;
    }
    *slot = Some(value?);
    Some(())
}

/// `detail_lines` の値を読む（上限と下限に収める）。数でなければ `None`。
fn lines_of(value: &str) -> Option<u32> {
    let lines: u32 = value.parse().ok()?;
    Some(lines.clamp(MIN_DETAIL_LINES, MAX_DETAIL_LINES))
}

/// `detail` の値を読む。
fn pane_of(value: &str) -> Option<DetailPane> {
    match value {
        "open" => Some(DetailPane::Open),
        "closed" => Some(DetailPane::Closed),
        _ => None,
    }
}

/// `version` の値を読む（1・2・3）。知らない版は `None`。
fn version_of(value: &str) -> Option<u8> {
    match value {
        OLD_VERSION => Some(1),
        VERSION => Some(2),
        LINES_VERSION => Some(3),
        _ => None,
    }
}

/// `find` の値（条件の名前の並び）を読む。どれも区別しない条件から、名前の条件を 1 つずつ入れる。知らない名前・同じ名前の
/// 重なりは `None`。
fn parse_find(value: &str) -> Option<FindOptions> {
    // 既定はひらがなカタカナだけを区別するので、それを外すとどれも区別しない。
    let nothing = FindOptions::standard().toggled_kana();
    value.split_whitespace().try_fold(nothing, |options, name| {
        let (_, on, toggle) = FIND_FLAGS.iter().find(|(flag, _, _)| *flag == name)?;
        (!on(options)).then(|| toggle(options))
    })
}

/// 読み取り中の値。鍵ごとに、まだ読んでいなければ `None`。
struct Reading {
    /// 読んだ版（1・2・3）。
    version: Option<u8>,
    theme: Option<ThemeChoice>,
    zoom: Option<ZoomLevel>,
    detail: Option<DetailPane>,
    find: Option<FindOptions>,
    detail_lines: Option<u32>,
}

impl Reading {
    fn default_values() -> Self {
        Self {
            version: None,
            theme: None,
            zoom: None,
            detail: None,
            find: None,
            detail_lines: None,
        }
    }

    /// 1 つの `鍵 = 値` を取り込む。知らない鍵・読めない値・2 度目の鍵は `None`。
    fn take(&mut self, key: &str, value: &str) -> Option<()> {
        match key {
            "version" => once(&mut self.version, version_of(value)),
            "detail_lines" => once(&mut self.detail_lines, lines_of(value)),
            "find" => once(&mut self.find, parse_find(value)),
            "theme" => once(&mut self.theme, ThemeChoice::from_keyword(value)),
            "zoom" => once(
                &mut self.zoom,
                value.parse().ok().and_then(ZoomLevel::from_percent),
            ),
            "detail" => once(&mut self.detail, pane_of(value)),
            _ => None,
        }
    }

    /// 版が書いてあれば設定にする。無い鍵は初期値で補う。
    fn into_settings(self) -> Option<Settings> {
        // 版 1 には `find` の鍵が無く、版 2 以下には `detail_lines` の鍵が無い（あれば読めない版として断る）。
        let version = self.version?;
        if (version < 2 && self.find.is_some()) || (version < 3 && self.detail_lines.is_some()) {
            return None;
        }
        let initial = Settings::initial();
        Some(
            initial
                .with_theme(self.theme.unwrap_or(initial.theme()))
                .with_zoom(self.zoom.unwrap_or(initial.zoom()))
                .with_detail(self.detail.unwrap_or(initial.detail()))
                .with_find(self.find.unwrap_or(initial.find()))
                .with_detail_lines(self.detail_lines.unwrap_or(initial.detail_lines())),
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
            "version = 1\ntheme = solarized-dark\nzoom = 110\ndetail = closed\n"
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
        // 既定の条件なら版 1 のまま書く（古い exe でも読める）。そうでなければ版 2。どれも区別しないなら値は空（後ろに空白を
        // 残さない）。
        assert!(encode(&Settings::initial()).starts_with("version = 1\n"));
        let nothing = encode(&Settings::initial().with_find(standard.toggled_kana()));
        assert!(nothing.starts_with("version = 2\n") && nothing.ends_with("\nfind =\n"));
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
            "version = 4\n",
            "version = 2\ndetail_lines = 8\n",
            "version = 3\ndetail_lines = many\n",
            "version = 3\ndetail_lines = 8\ndetail_lines = 9\n",
            "version = 2\nfind = case case\n",
            "version = 2\nfind = loose\n",
            "version = 2\nfind = kana\nfind = case\n",
            "version = 1\nfind = case\n",
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

    /// 詳細ペインの行の数を変えたときだけ版 3 で書き、読み戻せる。既定なら今までの版のまま（古い exe でも読める・
    /// GitHub Issue #452）。上限と下限に収めて読む。
    #[test]
    fn detail_lines_are_written_only_when_changed() {
        let plain = Settings::initial();
        assert!(encode(&plain).starts_with("version = 1\n"));
        let tall = plain.with_detail_lines(12);
        let text = encode(&tall);
        assert_eq!(
            text,
            "version = 3\ntheme = system\nzoom = 100\ndetail = open\nfind = kana\ndetail_lines = 12\n"
        );
        assert_eq!(parse(&text), Some(tall));
        assert_eq!(
            parse("version = 3\ndetail_lines = 999\n").map(Settings::detail_lines),
            Some(MAX_DETAIL_LINES)
        );
        assert_eq!(
            parse("version = 3\ndetail_lines = 0\n").map(Settings::detail_lines),
            Some(MIN_DETAIL_LINES)
        );
    }
}
