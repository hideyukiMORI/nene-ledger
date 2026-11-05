//! IME の確定前の文字列（UTF-16 の単位の列と、単位で数えたカーソル）を core の [`Preedit`] に直す（ADR 0026 の決定 4）。
//! Win32 を呼ばないので Linux でもテストする。出発点は `NeNeDocket` の同じ名の部品。

use nene_ledger_core::preedit::Preedit;

/// 単位の列を文字列に、単位のカーソルを文字の添字にする。壊れたサロゲートは置換文字として 1 文字に数える。
#[must_use]
pub(crate) fn preedit_of(units: &[u16], cursor_units: usize) -> Preedit {
    let text = String::from_utf16_lossy(units);
    let cursor = char::decode_utf16(units.iter().copied().take(cursor_units)).count();
    Preedit::new(&text, cursor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_in_utf16_units_becomes_a_character_index() {
        let units: Vec<u16> = "に😀ほ".encode_utf16().collect();
        assert_eq!(units.len(), 4);
        let preedit = preedit_of(&units, 3);
        assert_eq!(preedit.text(), "に😀ほ");
        assert_eq!(preedit.cursor(), 2);
        assert_eq!(preedit_of(&units, 9).cursor(), 3);
        assert_eq!(preedit_of(&[], 0).text(), "");
    }

    #[test]
    fn a_stray_surrogate_counts_as_one_replacement_character() {
        let preedit = preedit_of(&[0xD83D, 0x3042], 2);
        assert_eq!(preedit.text(), "\u{fffd}あ");
        assert_eq!(preedit.cursor(), 2);
    }
}
