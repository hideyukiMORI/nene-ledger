//! 貼り付けた文字列と IME の確定した文字列を、セルの値として挿せる形に整える（ADR 0026 の決定 2）。

use alloc::string::String;

/// `\r\n` と `\r` は `\n` に揃える（保存でその行の終わり方に揃え直す・ADR 0024）。
/// タブはセルの値の字として残す（区切りではない）。ほかの制御文字は捨てる。
#[must_use]
pub fn pasted_text(text: &str) -> String {
    let mut tidied = String::with_capacity(text.len());
    let mut after_return = false;
    for character in text.chars() {
        let carried = after_return;
        after_return = character == '\r';
        match character {
            '\n' if carried => {}
            '\r' | '\n' => tidied.push('\n'),
            '\t' => tidied.push('\t'),
            control if control.is_control() => {}
            printable => tidied.push(printable),
        }
    }
    tidied
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_breaks_are_normalized_and_control_characters_dropped() {
        assert_eq!(pasted_text("日本\r\n語\rx\ny"), "日本\n語\nx\ny");
        assert_eq!(pasted_text("a\tb\u{1}c\u{7f}"), "a\tbc");
        assert_eq!(pasted_text("\r\n\r\n"), "\n\n");
        assert_eq!(pasted_text("\r\r\n"), "\n\n");
        assert_eq!(pasted_text(""), "");
    }
}
