//! 1 バイトずつ進んで、レコードの始まりを見つける。

use nene_ledger_core::dialect::Dialect;
use nene_ledger_core::quoting::Quoting;
use nene_ledger_core::text_units::TextUnits;
use nene_ledger_core::unclosed_quote::UnclosedQuote;

use crate::char_phase::CharPhase;
use crate::scan_state::ScanState;

/// バックスラッシュ。
const BACKSLASH: u8 = b'\\';
/// ASCII でない単位の代わりに状態機械へ渡す印（どの構造の記号とも違う）。
const NOT_ASCII: u8 = 0xFF;

/// 1 バイトずつ進んで、レコードの始まりを見つける状態機械（速い経路・ADR 0006）。
///
/// 持つのは方言・今の状態・次のバイトの位置・最後に開いた引用符の位置だけ。塊の境目では状態を持ち越すので、
/// 閉じない引用符や巨大なセルがあっても記憶は増えない（BIG-04）。
/// 🔴 規則の正本は core の参照実装で、レコードの始まりと閉じなかった引用符の位置が全部それと同じであることを
/// テストで固定する（QLT-008）。
///
/// 引用符が閉じないことは入力の終わりで初めて分かる。`UnclosedQuote::Literal` のときは、呼び出し側が
/// `resume_literally` の返す位置から読み直す（ADR 0018 の決定 4）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RecordStarts {
    dialect: Dialect,
    state: ScanState,
    position: u64,
    open_quote: u64,
    phase: CharPhase,
}

impl RecordStarts {
    /// バイト位置 `position`（レコードの始まり）から進め始める。
    pub(crate) fn new(dialect: Dialect, position: u64) -> Self {
        Self {
            dialect,
            state: ScanState::RecordStart,
            position,
            open_quote: position,
            phase: CharPhase::Head,
        }
    }

    /// 次に受け取るバイトの位置。
    pub(crate) fn position(&self) -> u64 {
        self.position
    }

    /// ここで入力が終わると閉じない引用符の位置。引用の中でなければ `None`。
    pub(crate) fn unclosed_quote(&self) -> Option<u64> {
        self.state.unclosed().then_some(self.open_quote)
    }

    /// 引用の中のセルが `limit` バイトを超えていれば、その引用符の位置（ADR 0018 の決定 1・途中の知らせ）。
    pub(crate) fn long_quoted_cell(&self, limit: u64) -> Option<u64> {
        let long =
            self.state.inside_quotes() && self.position.saturating_sub(self.open_quote) > limit;
        long.then_some(self.open_quote)
    }

    /// 入力の終わりで引用符が閉じておらず、方言が `Literal` なら、その引用符を文字とみなして
    /// 引用の外のセルの続きに戻し、読み直す位置（引用符の次）を返す。
    pub(crate) fn resume_literally(&mut self) -> Option<u64> {
        let literal = match self.dialect.unclosed_quote() {
            UnclosedQuote::Literal => true,
            UnclosedQuote::ToEnd => false,
        };
        let quote = self.unclosed_quote().filter(|_| literal)?;
        self.state = ScanState::Unquoted;
        self.phase = CharPhase::Head;
        let width = u64::try_from(self.dialect.units().unit_len()).unwrap_or(1);
        self.position = quote.saturating_add(width);
        Some(self.position)
    }

    /// 1 バイト進める。このバイトで文字（UTF-16 は単位）がそろい、その文字でレコードが始まったら、その文字の頭の位置。
    ///
    /// `Shift_JIS` の 1 バイト目の次のバイトが 2 バイト目として正しければ、同じ文字の続きとして状態を変えずに飛ばす。
    /// UTF-16 は 2 バイトそろってから単位の値で状態を進める。どちらも参照実装の `TextUnits` と同じ（ADR 0019）。
    pub(crate) fn step(&mut self, byte: u8) -> Option<u64> {
        let here = self.position;
        self.position = self.position.saturating_add(1);
        let units = self.dialect.units();
        let (start, unit) = match units {
            TextUnits::Utf16Le | TextUnits::Utf16Be => {
                let CharPhase::HalfUnit(first) = self.phase else {
                    self.phase = CharPhase::HalfUnit(byte);
                    return None;
                };
                self.phase = CharPhase::Head;
                let pair = match units {
                    TextUnits::Utf16Be => u16::from_be_bytes([first, byte]),
                    TextUnits::Utf16Le | TextUnits::SingleByte | TextUnits::ShiftJis => {
                        u16::from_le_bytes([first, byte])
                    }
                };
                (here.saturating_sub(1), pair)
            }
            TextUnits::SingleByte | TextUnits::ShiftJis => {
                let trailing = self.phase == CharPhase::Trail && units.is_trail(byte);
                self.phase = CharPhase::Head;
                if trailing {
                    return None;
                }
                (here, u16::from(byte))
            }
        };
        // ASCII でない単位は、どの状態でもセルの中身として進む（構造の記号は ASCII だけ）。
        let symbol = u8::try_from(unit).unwrap_or(NOT_ASCII);
        let was_inside = self.state.inside_quotes();
        let starts = self.advance(symbol);
        if !was_inside && self.state.inside_quotes() {
            self.open_quote = start;
        }
        if units.is_lead(byte) {
            self.phase = CharPhase::Trail;
        }
        starts.then_some(start)
    }

    /// 入力が終わった。UTF-16 の単位の 1 バイト目だけが残り、それがレコードの頭なら、そのバイトの位置
    /// （参照実装は、奇数の長さの最後の 1 バイトもレコードとして数え、原本のバイトを落とさない・ADR 0019）。
    pub(crate) fn finish_input(&mut self) -> Option<u64> {
        let CharPhase::HalfUnit(_) = self.phase else {
            return None;
        };
        self.phase = CharPhase::Head;
        let at_record_start = match self.state {
            ScanState::RecordStart | ScanState::AfterCarriageReturn => true,
            ScanState::FieldStart
            | ScanState::Unquoted
            | ScanState::UnquotedEscape
            | ScanState::Quoted
            | ScanState::QuotedEscape
            | ScanState::QuoteInQuoted
            | ScanState::AfterClose => false,
        };
        at_record_start.then(|| self.position.saturating_sub(1))
    }

    /// 状態を 1 バイト進める。このバイトでレコードが始まったら `true`。
    fn advance(&mut self, byte: u8) -> bool {
        let (next, starts) = match self.state {
            ScanState::AfterCarriageReturn if byte == b'\n' => (ScanState::RecordStart, false),
            ScanState::RecordStart | ScanState::AfterCarriageReturn => {
                (self.field_start(byte), true)
            }
            ScanState::FieldStart => (self.field_start(byte), false),
            ScanState::Unquoted => (self.unquoted(byte), false),
            ScanState::UnquotedEscape => (ScanState::Unquoted, false),
            ScanState::Quoted => (self.quoted(byte), false),
            ScanState::QuotedEscape => (ScanState::Quoted, false),
            ScanState::QuoteInQuoted if byte == self.dialect.quote() => (ScanState::Quoted, false),
            ScanState::QuoteInQuoted | ScanState::AfterClose => (self.after_close(byte), false),
        };
        self.state = next;
        starts
    }

    /// 区切りか改行なら、その後ろの状態。
    fn boundary(self, byte: u8) -> Option<ScanState> {
        match byte {
            b'\n' => Some(ScanState::RecordStart),
            b'\r' => Some(ScanState::AfterCarriageReturn),
            other if other == self.dialect.delimiter().byte() => Some(ScanState::FieldStart),
            _ => None,
        }
    }

    /// セルの最初のバイト。引用を使う流儀で引用符なら引用の中へ。
    fn field_start(self, byte: u8) -> ScanState {
        let opens = match self.dialect.quoting() {
            Quoting::Doubled | Quoting::QuotedBackslash => byte == self.dialect.quote(),
            Quoting::Plain | Quoting::Backslash => false,
        };
        if opens {
            ScanState::Quoted
        } else {
            self.unquoted(byte)
        }
    }

    /// 引用の外。区切りと改行が先で、バックスラッシュの流儀だけが次の 1 バイトを逃がす。
    fn unquoted(self, byte: u8) -> ScanState {
        if let Some(next) = self.boundary(byte) {
            return next;
        }
        let escapes = match self.dialect.quoting() {
            Quoting::Backslash => byte == BACKSLASH,
            Quoting::Plain | Quoting::Doubled | Quoting::QuotedBackslash => false,
        };
        if escapes {
            ScanState::UnquotedEscape
        } else {
            ScanState::Unquoted
        }
    }

    /// 引用の中。
    fn quoted(self, byte: u8) -> ScanState {
        let quote = byte == self.dialect.quote();
        match self.dialect.quoting() {
            Quoting::Doubled if quote => ScanState::QuoteInQuoted,
            Quoting::QuotedBackslash if quote => ScanState::AfterClose,
            Quoting::QuotedBackslash if byte == BACKSLASH => ScanState::QuotedEscape,
            Quoting::Plain | Quoting::Backslash | Quoting::Doubled | Quoting::QuotedBackslash => {
                ScanState::Quoted
            }
        }
    }

    /// 閉じた引用符の後ろ。区切りか改行までは同じセルで、引用符もバックスラッシュも文字。
    fn after_close(self, byte: u8) -> ScanState {
        self.boundary(byte).unwrap_or(ScanState::AfterClose)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nene_ledger_core::delimiter::Delimiter;
    use nene_ledger_core::field::Field;
    use nene_ledger_core::records::Records;
    use nene_ledger_core::row_shape::RowShape;
    use nene_ledger_core::text_units::TextUnits;

    /// 状態機械で入力を最後まで進め、`Literal` なら読み直しも行う。レコードの始まりと、閉じなかった引用符の位置。
    fn walk(input: &[u8], dialect: Dialect) -> (Vec<usize>, Vec<usize>) {
        let mut machine = RecordStarts::new(dialect, 0);
        let mut starts = Vec::new();
        let mut quotes = Vec::new();
        let mut from = 0_usize;
        loop {
            starts.extend(
                input[from..]
                    .iter()
                    .filter_map(|&byte| machine.step(byte))
                    .map(|start| usize::try_from(start).unwrap()),
            );
            let Some(quote) = machine.unclosed_quote() else {
                starts.extend(
                    machine
                        .finish_input()
                        .map(|start| usize::try_from(start).unwrap()),
                );
                break;
            };
            quotes.push(usize::try_from(quote).unwrap());
            let Some(next) = machine.resume_literally() else {
                break;
            };
            from = usize::try_from(next).unwrap();
        }
        (starts, quotes)
    }

    fn starts(input: &[u8], dialect: Dialect) -> Vec<usize> {
        walk(input, dialect).0
    }

    /// 参照実装の、レコードの始まりと閉じなかった引用符の位置。
    fn reference(input: &[u8], dialect: Dialect) -> (Vec<usize>, Vec<usize>) {
        let mut starts = Vec::new();
        let mut quotes = Vec::new();
        for record in Records::new(input, dialect) {
            starts.push(record.raw().start());
            let unclosed = record
                .fields()
                .iter()
                .filter(|field| is_unclosed_quote(record.shape(), field, input, dialect))
                .map(|field| field.raw().start());
            quotes.extend(unclosed);
        }
        (starts, quotes)
    }

    /// 閉じなかった引用符（`ToEnd`）か、文字として読んだ引用符（`Literal`）で始まるセルか。
    fn is_unclosed_quote(shape: RowShape, field: &Field, input: &[u8], dialect: Dialect) -> bool {
        match shape {
            RowShape::UnclosedQuote => field.quoted() && field.raw().end() == input.len(),
            RowShape::LiteralQuote => {
                !field.quoted()
                    && dialect
                        .units()
                        .is_ascii_at(input, field.raw().start(), dialect.quote())
            }
            RowShape::Complete => false,
        }
    }

    fn tsv(quoting: Quoting) -> Dialect {
        Dialect::new(Delimiter::TAB, quoting, b'"').unwrap()
    }

    fn literal(dialect: Dialect) -> Dialect {
        dialect.with_unclosed_quote(UnclosedQuote::Literal)
    }

    fn dialects() -> Vec<Dialect> {
        let backslash_quoted =
            Dialect::new(Delimiter::COMMA, Quoting::QuotedBackslash, b'"').unwrap();
        vec![
            Dialect::rfc4180(),
            backslash_quoted,
            Dialect::new(Delimiter::COMMA, Quoting::Doubled, b'\'').unwrap(),
            tsv(Quoting::Doubled),
            tsv(Quoting::Backslash),
            tsv(Quoting::Plain),
            Dialect::new(Delimiter::new(b'\\').unwrap(), Quoting::Backslash, b'"').unwrap(),
            literal(Dialect::rfc4180()),
            literal(backslash_quoted),
            literal(tsv(Quoting::Doubled)),
        ]
    }

    #[test]
    fn line_endings_and_quoted_line_breaks() {
        let rfc = Dialect::rfc4180();
        assert_eq!(starts(b"a\r\nb\nc\rd", rfc), vec![0, 3, 5, 7]);
        assert_eq!(starts(b"a\n\n", rfc), vec![0, 2]);
        assert_eq!(starts(b"\"x\ny\",z\nw", rfc), vec![0, 8]);
        assert_eq!(starts(b"\"a\"\"\n\"\nb", rfc), vec![0, 7]);
        assert_eq!(starts(b"\"a\"b\nc", rfc), vec![0, 5]);
    }

    #[test]
    fn backslash_escapes_swallow_the_next_byte() {
        assert_eq!(starts(b"a\\\nb\nc", tsv(Quoting::Backslash)), vec![0, 5]);
        assert_eq!(starts(b"a\\\r\nb", tsv(Quoting::Backslash)), vec![0, 4]);
        let quoted = Dialect::new(Delimiter::COMMA, Quoting::QuotedBackslash, b'"').unwrap();
        assert_eq!(starts(b"\"a\\\"\nb\"\nc", quoted), vec![0, 8]);
    }

    #[test]
    fn unclosed_quote_is_remembered_and_literal_reads_on() {
        let input = b"a,b\nc,\"d\ne,f\n";
        assert_eq!(walk(input, Dialect::rfc4180()), (vec![0, 4], vec![6]));
        assert_eq!(
            walk(input, literal(Dialect::rfc4180())),
            (vec![0, 4, 9], vec![6])
        );
    }

    #[test]
    fn a_long_quoted_cell_is_reported_but_a_short_one_is_not() {
        let mut machine = RecordStarts::new(Dialect::rfc4180(), 0);
        for &byte in b"a,\"xyz" {
            machine.step(byte);
        }
        assert_eq!(machine.long_quoted_cell(2), Some(2));
        assert_eq!(machine.long_quoted_cell(4), None);
        machine.step(b'"');
        machine.step(b',');
        assert_eq!(machine.long_quoted_cell(2), None);
    }

    /// 種を固定した xorshift（テストは乱数の名前に頼らない・ARC-007）。
    struct Xorshift(u64);

    impl Xorshift {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13_u32;
            self.0 ^= self.0 >> 7_u32;
            self.0 ^= self.0 << 17_u32;
            self.0
        }
    }

    const ALPHABET: [&[u8]; 9] = [
        b"a",
        b",",
        b"\"",
        b"\r",
        b"\n",
        b"\t",
        b"\\",
        b"'",
        "あ".as_bytes(),
    ];

    #[test]
    fn every_record_start_and_unclosed_quote_matches_the_reference_on_random_inputs() {
        let mut rng = Xorshift(0xA076_1D64_78BD_642F);
        for _ in 0..20_000_u32 {
            let length = usize::try_from(rng.next() % 60).unwrap();
            let mut input = Vec::new();
            for _ in 0..length {
                input.extend_from_slice(ALPHABET[usize::try_from(rng.next() % 9).unwrap()]);
            }
            for dialect in dialects() {
                assert_eq!(
                    walk(&input, dialect),
                    reference(&input, dialect),
                    "{dialect:?} {input:?}"
                );
            }
        }
    }

    /// `Shift_JIS` の試しの文字。「表」（95 5C）・「ポ」（83 7C）・全角空白（81 40）は 2 バイト目が ASCII と重なり、
    /// `95` 単独は壊れた 1 バイト目（次が 2 バイト目として正しくなければ 1 バイトの文字）。
    const SHIFT_JIS_ALPHABET: [&[u8]; 11] = [
        b"a",
        b"|",
        b",",
        b"\"",
        b"\n",
        b"\r",
        b"\\",
        b"\x95\x5C",
        b"\x83\x7C",
        b"\x81\x40",
        b"\x95",
    ];

    fn shift_jis_dialects() -> Vec<Dialect> {
        let pipe = Dialect::new(Delimiter::new(b'|').unwrap(), Quoting::Doubled, b'"').unwrap();
        let pipe_backslash = Dialect::new(
            Delimiter::new(b'|').unwrap(),
            Quoting::QuotedBackslash,
            b'"',
        )
        .unwrap();
        [
            pipe,
            pipe_backslash,
            Dialect::new(Delimiter::new(b'|').unwrap(), Quoting::Backslash, b'"').unwrap(),
            Dialect::rfc4180(),
            pipe.with_unclosed_quote(UnclosedQuote::Literal),
            pipe_backslash.with_unclosed_quote(UnclosedQuote::Literal),
        ]
        .into_iter()
        .map(|dialect| dialect.with_units(TextUnits::ShiftJis))
        .collect()
    }

    #[test]
    fn shift_jis_record_starts_match_the_reference_on_random_inputs() {
        let mut rng = Xorshift(0x5851_F42D_4C95_7F2D);
        for _ in 0..20_000_u32 {
            let length = usize::try_from(rng.next() % 60).unwrap();
            let mut input = Vec::new();
            for _ in 0..length {
                input.extend_from_slice(
                    SHIFT_JIS_ALPHABET[usize::try_from(rng.next() % 11).unwrap()],
                );
            }
            for dialect in shift_jis_dialects() {
                assert_eq!(
                    walk(&input, dialect),
                    reference(&input, dialect),
                    "{dialect:?} {input:?}"
                );
            }
        }
    }

    #[test]
    fn a_trail_byte_equal_to_a_structure_byte_does_not_cut() {
        let pipe = Dialect::new(Delimiter::new(b'|').unwrap(), Quoting::Backslash, b'"')
            .unwrap()
            .with_units(TextUnits::ShiftJis);
        // 「表」（95 5C）の 2 バイト目のバックスラッシュは、次の改行を逃がさない。
        assert_eq!(starts(b"\x95\x5C\nb", pipe), vec![0, 3]);
        // 1 バイト目の次が改行なら、改行はそのまま。
        assert_eq!(starts(b"\x95\nb", pipe), vec![0, 2]);
    }

    /// UTF-16 の試しの字。構造の記号・ASCII・ひらがな・サロゲートの組。
    const UTF16_ALPHABET: [&str; 10] = ["a", ",", "\"", "\r", "\n", "\t", "\\", "'", "あ", "𠮷"];

    /// 乱数の字を UTF-16 の `units` で書き、`odd` なら奇数の長さにする最後の 1 バイトを足す。
    fn utf16_input(picks: &[usize], units: TextUnits, odd: bool) -> Vec<u8> {
        let text: String = picks
            .iter()
            .map(|&pick| UTF16_ALPHABET[pick % 10])
            .collect();
        let mut bytes: Vec<u8> = text
            .encode_utf16()
            .flat_map(|unit| match units {
                TextUnits::Utf16Be => unit.to_be_bytes(),
                TextUnits::Utf16Le | TextUnits::SingleByte | TextUnits::ShiftJis => {
                    unit.to_le_bytes()
                }
            })
            .collect();
        if odd {
            bytes.push(b'a');
        }
        bytes
    }

    fn utf16_dialects(units: TextUnits) -> Vec<Dialect> {
        let backslash_quoted =
            Dialect::new(Delimiter::COMMA, Quoting::QuotedBackslash, b'"').unwrap();
        [
            Dialect::rfc4180(),
            backslash_quoted,
            Dialect::new(Delimiter::TAB, Quoting::Backslash, b'"').unwrap(),
            Dialect::new(Delimiter::TAB, Quoting::Doubled, b'"').unwrap(),
            Dialect::rfc4180().with_unclosed_quote(UnclosedQuote::Literal),
            backslash_quoted.with_unclosed_quote(UnclosedQuote::Literal),
        ]
        .into_iter()
        .map(|dialect| dialect.with_units(units))
        .collect()
    }

    /// どの方言でも、状態機械が参照実装と同じレコードの始まりと閉じない引用符を出す。
    fn assert_walks_like_the_reference(input: &[u8], dialects: &[Dialect]) {
        for &dialect in dialects {
            assert_eq!(
                walk(input, dialect),
                reference(input, dialect),
                "{dialect:?} {input:?}"
            );
        }
    }

    #[test]
    fn utf16_record_starts_match_the_reference_on_random_inputs() {
        let mut rng = Xorshift(0xBF58_476D_1CE4_E5B9);
        for round in 0..6000_u32 {
            let length = usize::try_from(rng.next() % 40).unwrap();
            let picks: Vec<usize> = (0..length)
                .map(|_| usize::try_from(rng.next() % 10).unwrap())
                .collect();
            for units in [TextUnits::Utf16Le, TextUnits::Utf16Be] {
                let input = utf16_input(&picks, units, round % 7 == 0);
                assert_walks_like_the_reference(&input, &utf16_dialects(units));
            }
        }
    }
}
