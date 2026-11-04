//! パレットの絞り込み（純関数）。

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

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
                PaletteItem::Theme(_) | PaletteItem::Reading(_) | PaletteItem::Key(_) => None,
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
}
