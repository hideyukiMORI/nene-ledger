//! JSON のオブジェクトか配列の値を、詳細ペインで読みやすく整形する規則（VIEW-07・GitHub Issue #247）。値は変えない。

use alloc::string::String;

/// 整形する入れ子の深さの上限（深い入れ子で手間と記憶が膨らまないように・SEC-04）。
const MAX_DEPTH: usize = 128;
/// 整形する値の大きさの上限（バイト）。
const MAX_BYTES: usize = 1 << 20;
/// 1 段の字下げ。
const INDENT: &str = "  ";

/// `text` が（前後の空白を除いて）JSON のオブジェクトか配列として正しく読めるなら、2 字下げで整形した文字列。読めない・
/// オブジェクトでも配列でもない・深すぎる・大きすぎるなら `None`。文字列（逃がしを含む）と数は元の字のまま写す。空の
/// `{}`・`[]` は 1 行のまま。
#[must_use]
pub fn pretty_json(text: &str) -> Option<String> {
    let trimmed = text.trim_matches(is_json_space);
    if trimmed.len() > MAX_BYTES || !(trimmed.starts_with('{') || trimmed.starts_with('[')) {
        return None;
    }
    let mut layout = Layout {
        text: trimmed,
        at: 0,
        out: String::with_capacity(trimmed.len().saturating_mul(2)),
    };
    layout.value(0)?;
    layout.skip_space();
    (layout.at == trimmed.len()).then_some(layout.out)
}

/// JSON の空白（RFC 8259：空白・タブ・LF・CR）か。
const fn is_json_space(character: char) -> bool {
    matches!(character, ' ' | '\t' | '\n' | '\r')
}

/// 読み進める位置と、書き出す整形した文字列。
struct Layout<'text> {
    text: &'text str,
    at: usize,
    out: String,
}

impl Layout<'_> {
    /// 今の位置のバイト。
    fn peek(&self) -> Option<u8> {
        self.text.as_bytes().get(self.at).copied()
    }

    /// 空白を読み飛ばす。
    fn skip_space(&mut self) {
        while self
            .peek()
            .is_some_and(|byte| is_json_space(char::from(byte)))
        {
            self.at = self.at.saturating_add(1);
        }
    }

    /// 改行して、`depth` 段の字下げを書く。
    fn new_line(&mut self, depth: usize) {
        self.out.push('\n');
        for _ in 0..depth {
            self.out.push_str(INDENT);
        }
    }

    /// 1 つの値（深さ `depth`）を読んで書く。読めなければ `None`。
    fn value(&mut self, depth: usize) -> Option<()> {
        self.skip_space();
        match self.peek()? {
            b'{' => self.container(depth, (b'{', b'}'), true),
            b'[' => self.container(depth, (b'[', b']'), false),
            b'"' => self.string(),
            b't' => self.literal("true"),
            b'f' => self.literal("false"),
            b'n' => self.literal("null"),
            b'-' | b'0'..=b'9' => self.number(),
            _ => None,
        }
    }

    /// オブジェクト（`keyed`）か配列を読んで書く。中身があれば 1 つずつ改行して字下げする。
    fn container(&mut self, depth: usize, (open, close): (u8, u8), keyed: bool) -> Option<()> {
        let inner = depth.checked_add(1).filter(|&inner| inner <= MAX_DEPTH)?;
        self.at = self.at.saturating_add(1);
        self.out.push(char::from(open));
        self.skip_space();
        if self.peek()? == close {
            self.at = self.at.saturating_add(1);
            self.out.push(char::from(close));
            return Some(());
        }
        loop {
            self.new_line(inner);
            if keyed {
                self.skip_space();
                (self.peek()? == b'"').then_some(())?;
                self.string()?;
                self.skip_space();
                (self.peek()? == b':').then_some(())?;
                self.at = self.at.saturating_add(1);
                self.out.push_str(": ");
            }
            self.value(inner)?;
            self.skip_space();
            let next = self.peek()?;
            self.at = self.at.saturating_add(1);
            if next == b',' {
                self.out.push(',');
            } else if next == close {
                self.new_line(depth);
                self.out.push(char::from(close));
                return Some(());
            } else {
                return None;
            }
        }
    }

    /// 文字列（`"` から `"` まで）を確かめて、元の字のまま写す。制御文字と知らない逃がしは読めない。
    fn string(&mut self) -> Option<()> {
        let start = self.at;
        self.at = self.at.saturating_add(1);
        loop {
            let byte = self.peek()?;
            self.at = self.at.saturating_add(1);
            match byte {
                b'"' => break,
                b'\\' => self.escape()?,
                0x00..=0x1F => return None,
                _ => {}
            }
        }
        self.out.push_str(self.text.get(start..self.at)?);
        Some(())
    }

    /// `\` の後の 1 つの逃がし（`\u` は 16 進の 4 字）。
    fn escape(&mut self) -> Option<()> {
        let kind = self.peek()?;
        self.at = self.at.saturating_add(1);
        match kind {
            b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => Some(()),
            b'u' => {
                let digits = self
                    .text
                    .as_bytes()
                    .get(self.at..self.at.saturating_add(4))?;
                digits.iter().all(u8::is_ascii_hexdigit).then_some(())?;
                self.at = self.at.saturating_add(4);
                Some(())
            }
            _ => None,
        }
    }

    /// `true`・`false`・`null` の 1 つ。
    fn literal(&mut self, word: &str) -> Option<()> {
        let end = self.at.saturating_add(word.len());
        (self.text.get(self.at..end)? == word).then_some(())?;
        self.at = end;
        self.out.push_str(word);
        Some(())
    }

    /// 数（RFC 8259 の文法）を確かめて、元の字のまま写す。
    fn number(&mut self) -> Option<()> {
        let start = self.at;
        if self.peek() == Some(b'-') {
            self.at = self.at.saturating_add(1);
        }
        match self.peek()? {
            b'0' => self.at = self.at.saturating_add(1),
            b'1'..=b'9' => self.digits(),
            _ => return None,
        }
        if self.peek() == Some(b'.') {
            self.at = self.at.saturating_add(1);
            self.peek().filter(u8::is_ascii_digit)?;
            self.digits();
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.at = self.at.saturating_add(1);
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.at = self.at.saturating_add(1);
            }
            self.peek().filter(u8::is_ascii_digit)?;
            self.digits();
        }
        self.out.push_str(self.text.get(start..self.at)?);
        Some(())
    }

    /// 続く 10 進の数字を読み飛ばす。
    fn digits(&mut self) {
        while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
            self.at = self.at.saturating_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// オブジェクトと配列は 2 字下げで 1 つずつ改行する。文字列と数は元の字のまま。空の `{}`・`[]` は 1 行のまま。
    #[test]
    fn objects_and_arrays_are_indented() {
        assert_eq!(
            pretty_json(r#" {"名前":"東京é\"x","n":[1,-2.5e+3,true,null],"空":{},"a":[]} "#)
                .as_deref(),
            Some(
                "{\n  \"名前\": \"東京é\\\"x\",\n  \"n\": [\n    1,\n    -2.5e+3,\n    true,\n    null\n  ],\n  \"空\": {},\n  \"a\": []\n}"
            )
        );
        assert_eq!(pretty_json("[]").as_deref(), Some("[]"));
    }

    /// オブジェクトでも配列でもない値と、読めない JSON は整形しない。
    #[test]
    fn values_that_are_not_valid_containers_are_left_alone() {
        for text in [
            "",
            "123",
            "\"x\"",
            "true",
            "{",
            "[1,]",
            "{\"a\":1,}",
            "{\"a\" 1}",
            "{a:1}",
            "[01]",
            "[1.]",
            "[1e]",
            "[-]",
            "[\"a\\x\"]",
            "[\"a\\u12g4\"]",
            "[\"tab\there\"]",
            "[tru]",
            "[1] x",
            "[1][2]",
        ] {
            assert_eq!(pretty_json(text), None, "{text:?}");
        }
    }

    /// 深すぎる入れ子と大きすぎる値は整形しない（落ちない・SEC-04）。
    #[test]
    fn deep_or_huge_values_are_left_alone() {
        let deep = "[".repeat(MAX_DEPTH + 1) + &"]".repeat(MAX_DEPTH + 1);
        assert_eq!(pretty_json(&deep), None);
        let fits = "[".repeat(MAX_DEPTH) + &"]".repeat(MAX_DEPTH);
        assert!(pretty_json(&fits).is_some());
        let huge = String::from("[") + &"1,".repeat(MAX_BYTES / 2) + "1]";
        assert_eq!(pretty_json(&huge), None);
        let very_deep = "[".repeat(1_000_000);
        assert_eq!(pretty_json(&very_deep), None);
    }

    /// 種を固定した xorshift（テストは乱数の名前に頼らない・ARC-007）。
    struct Xorshift(u64);

    impl Xorshift {
        fn below(&mut self, bound: usize) -> usize {
            self.0 ^= self.0 << 13_u32;
            self.0 ^= self.0 >> 7_u32;
            self.0 ^= self.0 << 17_u32;
            usize::try_from(self.0 % u64::try_from(bound).unwrap()).unwrap()
        }
    }

    /// 区切りの前後に混ぜる空白。
    const SPACE: [&str; 4] = ["", " ", "\n", "\t "];

    /// 無作為の JSON の値（深さ `depth` まで）。区切りの前後に空白を混ぜる。
    fn random_value(rng: &mut Xorshift, depth: usize, out: &mut String) {
        const LEAVES: [&str; 9] = [
            "0",
            "-12.5e3",
            "true",
            "null",
            "\"a b\"",
            "\"東\\n\\\"\"",
            "\"\"",
            "3",
            "\"\\u00e9\"",
        ];
        let kind = if depth == 0 { 2 } else { rng.below(3) };
        match kind {
            0 => random_container(rng, depth, ('{', '}', true), out),
            1 => random_container(rng, depth, ('[', ']', false), out),
            _ => out.push_str(LEAVES[rng.below(LEAVES.len())]),
        }
    }

    /// 無作為のオブジェクト（`keyed`）か配列。
    fn random_container(
        rng: &mut Xorshift,
        depth: usize,
        (open, close, keyed): (char, char, bool),
        out: &mut String,
    ) {
        out.push(open);
        for index in 0..rng.below(4) {
            out.push_str(if index > 0 { "," } else { "" });
            out.push_str(SPACE[rng.below(SPACE.len())]);
            out.push_str(if keyed { "\"k\"" } else { "" });
            out.push_str(if keyed {
                SPACE[rng.below(SPACE.len())]
            } else {
                ""
            });
            out.push_str(if keyed { ":" } else { "" });
            random_value(rng, depth - 1, out);
            out.push_str(SPACE[rng.below(SPACE.len())]);
        }
        out.push(close);
    }

    /// 文字列の外の空白を除いた文字列（整形の前後で同じになるはず）。
    fn without_space(text: &str) -> String {
        let mut out = String::new();
        // 文字列の中か・直前が `\` か。
        let (mut in_string, mut escaped) = (false, false);
        for character in text.chars() {
            let keep = in_string || !is_json_space(character);
            if keep {
                out.push(character);
            }
            (in_string, escaped) = match (in_string, escaped, character) {
                (true, false, '\\') => (true, true),
                (true, false, '"') | (false, _, '"') => (!in_string, false),
                (true, _, _) => (true, false),
                (false, _, _) => (false, false),
            };
        }
        out
    }

    /// 無作為の JSON は整形でき、整形しても文字列の外の空白しか変わらず、整形した物をもう一度整形しても同じ。
    #[test]
    fn layout_changes_only_the_space_outside_strings() {
        let mut rng = Xorshift(0x2026_1108_0247);
        for _ in 0..2_000_u32 {
            let mut text = String::new();
            let keyed = rng.below(2) == 0;
            let (open, close) = if keyed { ('{', '}') } else { ('[', ']') };
            random_container(&mut rng, 4, (open, close, keyed), &mut text);
            let pretty = pretty_json(&text).unwrap_or_else(|| panic!("{text:?}"));
            assert_eq!(without_space(&pretty), without_space(&text), "{text:?}");
            assert_eq!(pretty_json(&pretty).as_deref(), Some(pretty.as_str()));
        }
    }
}
