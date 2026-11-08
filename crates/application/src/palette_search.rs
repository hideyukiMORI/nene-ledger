//! パレットの絞り込み（純関数）。

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use nene_ledger_core::dialect_candidate::DialectCandidate;
use nene_ledger_core::header_rows::HeaderRows;
use nene_ledger_core::theme_catalog::SEEDS;

use crate::command_id::CommandId;
use crate::key_guide::KeyGuide;
use crate::palette_item::PaletteItem;
use crate::reading_aspect::ReadingAspect;
use crate::reading_choice::ReadingChoice;
use crate::romaji::{fold, romanize};

/// 操作の候補。何も打たなければ固定の順で全部。打てば合う度合い（名前の読みの頭・英語の検索語の頭・どこかに含む）の順。
#[must_use]
pub fn commands(query: &str) -> Vec<PaletteItem> {
    let raw = query.trim().to_ascii_lowercase();
    let folded = fold(query);
    let mut ranked: Vec<(u8, usize, CommandId)> = CommandId::ALL
        .iter()
        .enumerate()
        .filter_map(|(order, &command)| {
            command_rank(command, &raw, &folded).map(|rank| (rank, order, command))
        })
        .collect();
    ranked.sort_by_key(|&(rank, order, _)| (rank, order));
    ranked
        .into_iter()
        .map(|(_, _, command)| PaletteItem::Command(command))
        .collect()
}

/// 操作の候補に、最近使った操作 `recent`（新しい順）を足したもの。何も打っていなければ最近使った操作を上に並べ、残りは
/// 固定の順（同じ操作は 1 度だけ・GitHub Issue #309）。打てば [`commands`] のまま（合う度合いの順）。
#[must_use]
pub fn commands_with_recent(query: &str, recent: &[CommandId]) -> Vec<PaletteItem> {
    let all = commands(query);
    if !query.trim().is_empty() || recent.is_empty() {
        return all;
    }
    recent
        .iter()
        .map(|&command| PaletteItem::Command(command))
        .chain(all.into_iter().filter(|item| match item {
            PaletteItem::Command(command) => !recent.contains(command),
            PaletteItem::Theme(_)
            | PaletteItem::Reading(_)
            | PaletteItem::Key(_)
            | PaletteItem::About(_)
            | PaletteItem::Candidate(_) => true,
        }))
        .collect()
}

/// 操作 `command` が検索 `raw`（小文字）・`folded`（寄せた形）に合う度合い（小さいほど上）。合わなければ `None`。
fn command_rank(command: CommandId, raw: &str, folded: &str) -> Option<u8> {
    if raw.is_empty() {
        return Some(0);
    }
    let reading = fold(&romanize(command.reading()));
    let keywords = command.keywords();
    if !folded.is_empty() && reading.starts_with(folded) {
        Some(0)
    } else if keywords.iter().any(|keyword| keyword.starts_with(raw)) {
        Some(1)
    } else if (!folded.is_empty() && reading.contains(folded))
        || keywords.iter().any(|keyword| keyword.contains(raw))
    {
        Some(2)
    } else {
        None
    }
}

/// キーの一覧の候補（Issue #63）。キーのあるパレットの操作（固定の順）の後に、パレットに無いキーの操作。打てば、キーの
/// 表記に含むか、名前の読み（ローマ字）か操作の検索語に合うものを、一覧の順のまま。
#[must_use]
pub fn key_list(query: &str) -> Vec<PaletteItem> {
    let raw = query.trim().to_ascii_lowercase();
    let folded = fold(query);
    let in_keys = |keys: &str| keys.to_ascii_lowercase().contains(&raw);
    let commands = CommandId::ALL
        .iter()
        .filter(|command| {
            command.keys().is_some_and(|keys| {
                in_keys(keys) || command_rank(**command, &raw, &folded).is_some()
            })
        })
        .map(|&command| PaletteItem::Command(command));
    let guides = KeyGuide::ALL
        .iter()
        .filter(|guide| {
            in_keys(guide.keys())
                || (!folded.is_empty() && fold(&romanize(guide.reading())).contains(&folded))
        })
        .map(|&guide| PaletteItem::Key(guide));
    commands.chain(guides).collect()
}

/// 配色の候補。何も打たなければ表の順で全部。打てば名前に含むもの（頭から合うものが先）。
#[must_use]
pub fn themes(query: &str) -> Vec<PaletteItem> {
    let wanted = compact(query);
    let mut ranked: Vec<(bool, usize)> = SEEDS
        .iter()
        .enumerate()
        .filter_map(|(position, seed)| {
            let name = compact(seed.name());
            name.contains(&wanted)
                .then(|| (!name.starts_with(&wanted), position))
        })
        .collect();
    ranked.sort_unstable();
    ranked
        .into_iter()
        .map(|(_, position)| PaletteItem::Theme(position))
        .collect()
}

/// 読み方の面 `aspect` の候補。何も打たなければ決まった順で全部。打てば英字の検索語か名前に含むもの。
/// 区切りの面で英数字でない半角の字を 1 つだけ打てば、その字で区切る候補 1 つ（Issue #67）。
#[must_use]
pub fn readings(aspect: ReadingAspect, query: &str) -> Vec<PaletteItem> {
    if aspect == ReadingAspect::Delimiter
        && let Some(byte) = typed_delimiter(query)
    {
        return vec![PaletteItem::Reading(ReadingChoice::Delimiter(byte))];
    }
    if aspect == ReadingAspect::Header
        && let Some(choices) = header_range(query)
    {
        return choices.into_iter().map(PaletteItem::Reading).collect();
    }
    let wanted = compact(query);
    aspect
        .options()
        .into_iter()
        .filter(|choice| {
            wanted.is_empty()
                || compact(choice.keyword()).contains(&wanted)
                || compact(&choice.name()).contains(&wanted)
        })
        .map(PaletteItem::Reading)
        .collect()
}

/// 見出しの行の段で打った行の番号（GitHub Issue #315・専門家ペルソナの 3 巡目）。`2` は 2 行目から始まる見出し（1〜3 行）、
/// `2-3`（`〜` も）は 2〜3 行目の見出しだけ。数でなければ `None`（今までどおり名前で絞る）。範囲が見出しにできない
/// （0 行目・逆・4 行以上）なら空。
fn header_range(query: &str) -> Option<Vec<ReadingChoice>> {
    let text = query.trim();
    let (from, to) = match text.split_once(['-', '〜', '~']) {
        Some((from, to)) => (from.trim(), Some(to.trim())),
        None => (text, None),
    };
    let number = |part: &str| -> Option<u64> {
        (!part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
            .then(|| part.parse::<u64>().ok())
            .flatten()
    };
    let first = number(from)?.checked_sub(1);
    let Some(first) = first else {
        return Some(Vec::new());
    };
    let Some(to) = to else {
        return Some(
            (1..=HeaderRows::MAX)
                .filter_map(|count| HeaderRows::new(first, count))
                .map(|rows| ReadingChoice::Header(Some(rows)))
                .collect(),
        );
    };
    let last = number(to)?;
    Some(
        last.checked_sub(first)
            .and_then(|count| HeaderRows::new(first, count))
            .map(|rows| ReadingChoice::Header(Some(rows)))
            .into_iter()
            .collect(),
    )
}

/// 判別の候補 `list`（点数の順）。何も打たなければ全部をその順で。打てば区切りの英字の検索語か名前に含むもの（Issue #3）。
#[must_use]
pub fn candidates(list: &[DialectCandidate], query: &str) -> Vec<PaletteItem> {
    let wanted = compact(query);
    list.iter()
        .copied()
        .filter(|candidate| {
            let choice = ReadingChoice::dialect(candidate.dialect());
            wanted.is_empty()
                || compact(choice.keyword()).contains(&wanted)
                || compact(&choice.name()).contains(&wanted)
        })
        .map(PaletteItem::Candidate)
        .collect()
}

/// 区切りとして打った字。英数字でない半角の字（空白を含む）がちょうど 1 つなら、そのバイト。
fn typed_delimiter(query: &str) -> Option<u8> {
    let mut characters = query.chars();
    let only = characters.next()?;
    if characters.next().is_some() || only.is_ascii_alphanumeric() {
        return None;
    }
    u8::try_from(u32::from(only))
        .ok()
        .filter(|byte| (0x20..=0x7E).contains(byte))
}

/// 名前を比べる形（小文字の英数字だけ）。
fn compact(text: &str) -> String {
    text.chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|character| character.to_ascii_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use nene_ledger_core::theme_catalog::{COUNT, position_of};

    fn ids(items: &[PaletteItem]) -> Vec<CommandId> {
        items
            .iter()
            .filter_map(|item| match item {
                PaletteItem::Command(command) => Some(*command),
                PaletteItem::Theme(_)
                | PaletteItem::Reading(_)
                | PaletteItem::Key(_)
                | PaletteItem::About(_)
                | PaletteItem::Candidate(_) => None,
            })
            .collect()
    }

    #[test]
    fn the_key_list_has_keyed_commands_then_the_other_keys_and_filters() {
        let all = key_list("");
        let keyed = CommandId::ALL
            .iter()
            .filter(|command| command.keys().is_some())
            .count();
        assert_eq!(all.len(), keyed + KeyGuide::ALL.len());
        assert!(
            all.iter()
                .take(keyed)
                .all(|item| matches!(item, PaletteItem::Command(_)))
        );
        assert_eq!(
            all.get(keyed..),
            Some(KeyGuide::ALL.map(PaletteItem::Key).as_slice())
        );
        // キーの表記で（大文字と小文字は問わない）。
        let shift = key_list("Shift+");
        assert!(shift.contains(&PaletteItem::Command(CommandId::ToggleDetail)));
        assert!(shift.contains(&PaletteItem::Key(KeyGuide::ExtendClick)));
        assert!(!shift.contains(&PaletteItem::Command(CommandId::Copy)));
        // 名前の読み（ローマ字）と、操作の検索語で。
        assert_eq!(ids(&key_list("kopi")), vec![CommandId::Copy]);
        assert_eq!(
            key_list("haisyoku"),
            vec![
                PaletteItem::Key(KeyGuide::NextTheme),
                PaletteItem::Key(KeyGuide::PreviousTheme)
            ]
        );
        assert!(key_list("xyz").is_empty());
    }

    #[test]
    fn nothing_typed_lists_every_command_in_the_fixed_order() {
        assert_eq!(ids(&commands("")), CommandId::ALL.to_vec());
        assert_eq!(ids(&commands("  ")), CommandId::ALL.to_vec());
    }

    #[test]
    fn english_and_romaji_find_the_same_commands() {
        assert_eq!(ids(&commands("theme")), vec![CommandId::ChooseTheme]);
        assert_eq!(ids(&commands("haishoku")), vec![CommandId::ChooseTheme]);
        assert_eq!(ids(&commands("haisyoku")), vec![CommandId::ChooseTheme]);
        assert_eq!(ids(&commands("Copy")), vec![CommandId::Copy]);
        assert_eq!(ids(&commands("kopi")), vec![CommandId::Copy]);
        // 「しょうさい」は shousai でも syosai でも。
        assert_eq!(ids(&commands("shousai")), vec![CommandId::ToggleDetail]);
        assert_eq!(ids(&commands("syosai")), vec![CommandId::ToggleDetail]);
        assert!(commands("xyz").is_empty());
    }

    #[test]
    fn closer_matches_come_first() {
        // 「もじ」で始まる読みが 4 つ（大きく・小さく・戻す・文字コード）。英語の font は 3 つ。
        assert_eq!(
            ids(&commands("moji")),
            vec![
                CommandId::ZoomIn,
                CommandId::ZoomOut,
                CommandId::ZoomReset,
                CommandId::ChooseEncoding
            ]
        );
        assert_eq!(ids(&commands("zoom")).len(), 3);
        // 「ookiku」は読みの途中（もじを・おおきく）に含む。
        assert_eq!(ids(&commands("okiku")), vec![CommandId::ZoomIn]);
    }

    #[test]
    fn themes_filter_by_name_with_prefix_matches_first() {
        assert_eq!(themes("").len(), COUNT);
        let solarized = themes("solar");
        assert_eq!(
            solarized,
            vec![
                PaletteItem::Theme(position_of("solarized-dark").unwrap()),
                PaletteItem::Theme(position_of("solarized-light").unwrap()),
            ]
        );
        // 頭から合う github-light が、途中に含む名前より先。
        let light = themes("github_light");
        assert_eq!(
            light.first(),
            Some(&PaletteItem::Theme(position_of("github-light").unwrap()))
        );
        assert!(themes("zzzz").is_empty());
    }

    #[test]
    fn candidates_keep_the_score_order_and_filter_by_the_delimiter_word() {
        use nene_ledger_core::delimiter::Delimiter;
        use nene_ledger_core::dialect::Dialect;
        use nene_ledger_core::quoting::Quoting;
        let comma = DialectCandidate::new(Dialect::rfc4180(), 900, 5);
        let tab = DialectCandidate::new(
            Dialect::new(Delimiter::TAB, Quoting::Plain, b'"').unwrap(),
            400,
            2,
        );
        let list = [comma, tab];
        assert_eq!(
            candidates(&list, ""),
            vec![PaletteItem::Candidate(comma), PaletteItem::Candidate(tab)]
        );
        assert_eq!(candidates(&list, "tab"), vec![PaletteItem::Candidate(tab)]);
        assert_eq!(
            candidates(&list, "RFC"),
            vec![PaletteItem::Candidate(comma)]
        );
        assert!(candidates(&list, "xyz").is_empty());
    }

    /// 何も打っていない一覧は、最近使った操作を上に並べ、残りは固定の順で 1 度ずつ。打てば合う度合いの順のまま
    /// （GitHub Issue #309）。
    #[test]
    fn recent_commands_come_first_only_before_typing() {
        let recent = [CommandId::Replace, CommandId::ChooseTheme];
        let listed = ids(&commands_with_recent("", &recent));
        assert_eq!(&listed[..2], &recent);
        assert_eq!(listed.len(), CommandId::ALL.len());
        assert_eq!(
            ids(&commands_with_recent("kensaku", &recent)),
            ids(&commands("kensaku"))
        );
        assert_eq!(ids(&commands_with_recent("", &[])), ids(&commands("")));
    }

    /// 見出しの行の段は、行の番号を打てばそこから始まる見出しに、`2-3` と打てばその範囲だけに絞る（GitHub Issue #315）。
    #[test]
    fn header_rows_are_picked_by_their_line_numbers() {
        let names = |query: &str| -> Vec<String> {
            readings(ReadingAspect::Header, query)
                .into_iter()
                .filter_map(|item| match item {
                    PaletteItem::Reading(choice) => Some(choice.name()),
                    PaletteItem::Command(_)
                    | PaletteItem::Theme(_)
                    | PaletteItem::Key(_)
                    | PaletteItem::About(_)
                    | PaletteItem::Candidate(_) => None,
                })
                .collect()
        };
        assert_eq!(
            names("2"),
            vec!["2 行目が見出し", "2〜3 行目が見出し", "2〜4 行目が見出し"]
        );
        assert_eq!(names("2-3"), vec!["2〜3 行目が見出し"]);
        assert_eq!(names(" 2〜3 "), vec!["2〜3 行目が見出し"]);
        assert!(names("3-2").is_empty() && names("1-5").is_empty() && names("0").is_empty());
        // 数でなければ今までどおり名前で絞る。
        assert_eq!(names("header").len(), ReadingAspect::Header.options().len());
    }
}
