//! 入力をレコードの列に切る（参照実装）。

use alloc::vec::Vec;

use crate::boundary::Boundary;
use crate::byte_range::ByteRange;
use crate::dialect::Dialect;
use crate::field_scan::FieldScan;
use crate::line_ending::LineEnding;
use crate::record::Record;
use crate::row_shape::RowShape;
use crate::scanning::scan_field;

/// 入力をレコードの列に切る反復子（参照実装・ADR 0006）。
///
/// 空の入力はレコード 0 件。最後の改行の後ろに空のレコードは作らない。空行は空のセル 1 つのレコード。
#[derive(Debug, Clone)]
pub struct Records<'input> {
    input: &'input [u8],
    dialect: Dialect,
    position: usize,
}

impl<'input> Records<'input> {
    /// `input` を `dialect` で切る。
    #[must_use]
    pub fn new(input: &'input [u8], dialect: Dialect) -> Self {
        Self {
            input,
            dialect,
            position: 0,
        }
    }

    /// 最後のセルを切った結果から、レコードを閉じる。`shape` はセルの形のうち方言どおりでなかったもの。
    fn close(
        start: usize,
        last: FieldScan,
        fields: Vec<crate::field::Field>,
        shape: RowShape,
    ) -> Record {
        let terminator = match last.boundary() {
            Boundary::Terminator(ending) => ending,
            Boundary::Delimiter | Boundary::End => LineEnding::EndOfInput,
        };
        Record::new(
            ByteRange::between(start, last.stop()),
            terminator,
            fields,
            shape,
        )
    }
}

impl Iterator for Records<'_> {
    type Item = Record;

    fn next(&mut self) -> Option<Record> {
        if self.position >= self.input.len() {
            return None;
        }
        let start = self.position;
        let mut fields = Vec::new();
        let mut cursor = start;
        let mut shape = RowShape::Complete;
        loop {
            let scan = scan_field(self.input, cursor, &self.dialect);
            fields.push(scan.field());
            if scan.shape() != RowShape::Complete {
                shape = scan.shape();
            }
            if scan.boundary() != Boundary::Delimiter {
                self.position = scan.next();
                return Some(Self::close(start, scan, fields, shape));
            }
            cursor = scan.next();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::delimiter::Delimiter;
    use crate::line_ending::LineEnding;
    use crate::quoting::Quoting;
    use crate::text_units::{TextUnits, characters};
    use crate::unclosed_quote::UnclosedQuote;
    use crate::value::value;
    use alloc::vec;
    use alloc::vec::Vec;

    /// 各レコードのセルの値を集める。
    fn table(input: &[u8], dialect: Dialect) -> Vec<Vec<Vec<u8>>> {
        Records::new(input, dialect)
            .map(|record| {
                record
                    .fields()
                    .iter()
                    .map(|&field| value(input, field, &dialect))
                    .collect()
            })
            .collect()
    }

    /// レコードの区間と改行をつなげ直す（往復の土台）。
    fn rejoin(input: &[u8], dialect: Dialect) -> Vec<u8> {
        let mut out = Vec::new();
        for record in Records::new(input, dialect) {
            out.extend_from_slice(record.raw().slice(input).unwrap());
            out.extend_from_slice(record.terminator().bytes());
        }
        out
    }

    fn cells(rows: &[&[&str]]) -> Vec<Vec<Vec<u8>>> {
        rows.iter()
            .map(|row| row.iter().map(|cell| cell.as_bytes().to_vec()).collect())
            .collect()
    }

    fn tsv(quoting: Quoting) -> Dialect {
        Dialect::new(Delimiter::TAB, quoting, b'"').unwrap()
    }

    #[test]
    fn empty_input_has_no_records() {
        assert_eq!(Records::new(b"", Dialect::rfc4180()).count(), 0);
    }

    #[test]
    fn rfc4180_basic_and_quoted_fields() {
        let input = b"aaa,\"b,bb\",ccc\r\nzzz,\"y\"\"yy\",xxx\r\n";
        assert_eq!(
            table(input, Dialect::rfc4180()),
            cells(&[&["aaa", "b,bb", "ccc"], &["zzz", "y\"yy", "xxx"]])
        );
    }

    #[test]
    fn line_break_inside_quotes_stays_in_the_cell() {
        let input = "1,\"東京都\n千代田区\",x\n2,y,z".as_bytes();
        let records: Vec<Record> = Records::new(input, Dialect::rfc4180()).collect();
        assert_eq!(records.len(), 2);
        assert_eq!(
            table(input, Dialect::rfc4180()),
            cells(&[&["1", "東京都\n千代田区", "x"], &["2", "y", "z"]])
        );
        assert_eq!(records[1].terminator(), LineEnding::EndOfInput);
    }

    #[test]
    fn keeps_each_line_ending_including_mixed_and_bare_cr() {
        let input = b"a\r\nb\nc\rd";
        let endings: Vec<LineEnding> = Records::new(input, Dialect::rfc4180())
            .map(|record| record.terminator())
            .collect();
        assert_eq!(
            endings,
            vec![
                LineEnding::CrLf,
                LineEnding::Lf,
                LineEnding::Cr,
                LineEnding::EndOfInput
            ]
        );
    }

    #[test]
    fn empty_line_is_one_empty_cell_and_trailing_delimiter_is_an_empty_last_cell() {
        let input = b"a,b\n\nc,\n";
        assert_eq!(
            table(input, Dialect::rfc4180()),
            cells(&[&["a", "b"], &[""], &["c", ""]])
        );
    }

    #[test]
    fn ragged_rows_keep_their_own_widths() {
        let widths: Vec<usize> = Records::new(b"a,b,c\nd\ne,f\n", Dialect::rfc4180())
            .map(|record| record.fields().len())
            .collect();
        assert_eq!(widths, vec![3, 1, 2]);
    }

    #[test]
    fn leading_zeros_dates_and_exponents_stay_text() {
        let input = b"00123,1-2,1.20E+03,1234567890123456789, x \n";
        assert_eq!(
            table(input, Dialect::rfc4180()),
            cells(&[&["00123", "1-2", "1.20E+03", "1234567890123456789", " x "]])
        );
    }

    #[test]
    fn unclosed_quote_runs_to_the_end_and_is_marked() {
        let input = b"a,b\nc,\"d\ne,f\n";
        let records: Vec<Record> = Records::new(input, Dialect::rfc4180()).collect();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].shape(), RowShape::Complete);
        assert_eq!(records[1].shape(), RowShape::UnclosedQuote);
        assert_eq!(records[1].raw().end(), input.len());
        assert_eq!(rejoin(input, Dialect::rfc4180()), input.to_vec());
    }

    #[test]
    fn literal_reads_an_unclosed_quote_as_text_and_keeps_cutting() {
        let input = b"a,b\nc,\"d\ne,f\n";
        let dialect = literal(Dialect::rfc4180());
        assert_eq!(
            table(input, dialect),
            cells(&[&["a", "b"], &["c", "\"d"], &["e", "f"]])
        );
        let shapes: Vec<RowShape> = Records::new(input, dialect).map(|r| r.shape()).collect();
        assert_eq!(
            shapes,
            vec![
                RowShape::Complete,
                RowShape::LiteralQuote,
                RowShape::Complete
            ]
        );
        assert_eq!(rejoin(input, dialect), input.to_vec());
    }

    #[test]
    fn literal_only_touches_the_quote_that_never_closes() {
        let input = b"\"x\ny\",\"z\n1,2\n";
        let dialect = literal(Dialect::rfc4180());
        assert_eq!(
            table(input, dialect),
            cells(&[&["x\ny", "\"z"], &["1", "2"]])
        );
        let quoted: Vec<bool> = Records::new(input, dialect)
            .next()
            .unwrap()
            .fields()
            .iter()
            .map(|field| field.quoted())
            .collect();
        assert_eq!(quoted, vec![true, false]);
    }

    #[test]
    fn text_after_a_closing_quote_stays_in_the_same_cell() {
        let input = b"\"a\"b,c\n";
        assert_eq!(table(input, Dialect::rfc4180()), cells(&[&["ab", "c"]]));
        assert!(
            Records::new(input, Dialect::rfc4180())
                .next()
                .unwrap()
                .fields()[0]
                .quoted()
        );
    }

    #[test]
    fn quote_in_the_middle_of_an_unquoted_cell_is_literal() {
        assert_eq!(
            table(b"a\"b,c\n", Dialect::rfc4180()),
            cells(&[&["a\"b", "c"]])
        );
    }

    #[test]
    fn tsv_with_csv_style_quotes() {
        let input = b"a\t\"b\tc\"\t\"d\"\"e\"\n";
        assert_eq!(
            table(input, tsv(Quoting::Doubled)),
            cells(&[&["a", "b\tc", "d\"e"]])
        );
    }

    #[test]
    fn tsv_with_backslash_escapes() {
        let input = b"a\\tb\tc\\\\d\te\\nf\\\tg\n";
        assert_eq!(
            table(input, tsv(Quoting::Backslash)),
            cells(&[&["a\tb", "c\\d", "e\nf\tg"]])
        );
    }

    #[test]
    fn tsv_without_escapes_takes_quotes_literally() {
        let input = b"\"a\t\"b\"\n";
        assert_eq!(
            table(input, tsv(Quoting::Plain)),
            cells(&[&["\"a", "\"b\""]])
        );
    }

    #[test]
    fn backslash_inside_quotes() {
        let dialect = Dialect::new(Delimiter::COMMA, Quoting::QuotedBackslash, b'"').unwrap();
        let input = b"\"a\\\"b,c\",\"x\"y,\"unclosed\\\"\n";
        let rows = table(input, dialect);
        assert_eq!(rows[0][0], b"a\"b,c".to_vec());
        assert_eq!(rows[0][1], b"xy".to_vec());
        assert_eq!(rows[0][2], b"unclosed\"\n".to_vec());
        let shapes: Vec<RowShape> = Records::new(input, dialect).map(|r| r.shape()).collect();
        assert_eq!(shapes, vec![RowShape::UnclosedQuote]);
    }

    #[test]
    fn single_quote_dialect() {
        let dialect = Dialect::new(Delimiter::COMMA, Quoting::Doubled, b'\'').unwrap();
        assert_eq!(table(b"'a,b','c''d'\n", dialect), cells(&[&["a,b", "c'd"]]));
    }

    #[test]
    fn trailing_backslash_at_the_end_of_input() {
        assert_eq!(table(b"a\\", tsv(Quoting::Backslash)), cells(&[&["a\\"]]));
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

    fn random_input(rng: &mut Xorshift) -> Vec<u8> {
        let length = usize::try_from(rng.next() % 40).unwrap();
        let mut out = Vec::new();
        for _ in 0..length {
            let pick = usize::try_from(rng.next() % 9).unwrap();
            out.extend_from_slice(ALPHABET[pick]);
        }
        out
    }

    fn literal(dialect: Dialect) -> Dialect {
        dialect.with_unclosed_quote(UnclosedQuote::Literal)
    }

    fn dialects() -> Vec<Dialect> {
        vec![
            Dialect::rfc4180(),
            Dialect::new(Delimiter::COMMA, Quoting::QuotedBackslash, b'"').unwrap(),
            Dialect::new(Delimiter::COMMA, Quoting::Doubled, b'\'').unwrap(),
            tsv(Quoting::Doubled),
            tsv(Quoting::Backslash),
            tsv(Quoting::Plain),
            literal(Dialect::rfc4180()),
            literal(Dialect::new(Delimiter::COMMA, Quoting::QuotedBackslash, b'"').unwrap()),
        ]
    }

    #[test]
    fn random_inputs_rejoin_to_the_same_bytes() {
        let mut rng = Xorshift(0x9E37_79B9_7F4A_7C15);
        for _ in 0..5000_u32 {
            let input = random_input(&mut rng);
            for dialect in dialects() {
                assert_eq!(rejoin(&input, dialect), input, "{dialect:?} {input:?}");
            }
        }
    }

    fn assert_fields_in_order(record: &Record) {
        let mut previous = record.raw().start();
        for field in record.fields() {
            assert!(field.raw().start() >= previous);
            assert!(field.raw().end() <= record.raw().end());
            previous = field.raw().end();
        }
        assert!(!record.fields().is_empty());
    }

    #[test]
    fn random_inputs_have_fields_inside_their_record_in_order() {
        let mut rng = Xorshift(0xD1B5_4A32_D192_ED03);
        for _ in 0..5000_u32 {
            let input = random_input(&mut rng);
            for dialect in dialects() {
                Records::new(&input, dialect).for_each(|record| assert_fields_in_order(&record));
            }
        }
    }

    fn shift_jis(dialect: Dialect) -> Dialect {
        dialect.with_units(TextUnits::ShiftJis)
    }

    fn pipe() -> Dialect {
        Dialect::new(Delimiter::new(b'|').unwrap(), Quoting::Doubled, b'"').unwrap()
    }

    fn bytes(rows: &[&[&[u8]]]) -> Vec<Vec<Vec<u8>>> {
        rows.iter()
            .map(|row| row.iter().map(|cell| cell.to_vec()).collect())
            .collect()
    }

    #[test]
    fn a_shift_jis_trail_byte_equal_to_the_delimiter_stays_in_its_cell() {
        // 「ポ」は 83 7C。2 バイト目が `|` と同じ。
        let input = b"\x83\x7C|a\n";
        assert_eq!(
            table(input, shift_jis(pipe())),
            bytes(&[&[b"\x83\x7C", b"a"]])
        );
        // 1 バイトずつでは漢字の途中で切れる（ADR 0019 の文脈）。
        assert_eq!(table(input, pipe()).first().map(Vec::len), Some(3));
    }

    #[test]
    fn a_shift_jis_trail_backslash_does_not_escape() {
        // 「表」は 95 5C。2 バイト目がバックスラッシュでも、エスケープでも閉じる引用符を逃がす印でもない。
        let escaped = shift_jis(tsv(Quoting::Backslash));
        assert_eq!(
            table(b"\x95\x5Cx\tb\n", escaped),
            bytes(&[&[b"\x95\x5Cx", b"b"]])
        );
        // バックスラッシュが逃がすのは次の 1 文字（2 バイト）。
        assert_eq!(
            table(b"\\\x95\x5C\tb\n", escaped),
            bytes(&[&[b"\x95\x5C", b"b"]])
        );
        let quoted =
            shift_jis(Dialect::new(Delimiter::COMMA, Quoting::QuotedBackslash, b'"').unwrap());
        assert_eq!(
            table(b"\"\x95\x5C\",b\n", quoted),
            bytes(&[&[b"\x95\x5C", b"b"]])
        );
    }

    #[test]
    fn a_broken_lead_byte_does_not_swallow_a_line_break() {
        let dialect = shift_jis(Dialect::rfc4180());
        assert_eq!(
            table(b"\x95\na\n\x95", dialect),
            bytes(&[&[b"\x95"], &[b"a"], &[b"\x95"]])
        );
    }

    const SHIFT_JIS_ALPHABET: [&[u8]; 10] = [
        b"a",
        b"|",
        b"\"",
        b"\n",
        b"\\",
        b"\x95\x5C",
        b"\x83\x7C",
        b"\x81\x40",
        b"\x95",
        b"\r\n",
    ];

    fn random_shift_jis(rng: &mut Xorshift) -> Vec<u8> {
        let length = usize::try_from(rng.next() % 40).unwrap();
        let mut out = Vec::new();
        for _ in 0..length {
            let pick = usize::try_from(rng.next() % 10).unwrap();
            out.extend_from_slice(SHIFT_JIS_ALPHABET[pick]);
        }
        out
    }

    /// `input` の文字の頭の位置（と終わり）。
    fn char_starts(input: &[u8], units: TextUnits) -> Vec<usize> {
        let mut starts = vec![0];
        let mut position = 0_usize;
        for character in characters(input, units) {
            position += character.len();
            starts.push(position);
        }
        starts
    }

    /// セルが順に並び、どのセルの始まりと終わりも文字の頭（`starts`）にある。
    fn assert_cut_at_character_starts(record: &Record, starts: &[usize]) {
        assert_fields_in_order(record);
        for field in record.fields() {
            assert!(starts.contains(&field.raw().start()));
            assert!(starts.contains(&field.raw().end()));
        }
    }

    #[test]
    fn random_shift_jis_cuts_only_at_character_starts_and_rejoins() {
        let mut rng = Xorshift(0x2545_F491_4F6C_DD1D);
        let dialects = [
            shift_jis(pipe()),
            shift_jis(tsv(Quoting::Backslash)),
            shift_jis(
                Dialect::new(
                    Delimiter::new(b'|').unwrap(),
                    Quoting::QuotedBackslash,
                    b'"',
                )
                .unwrap(),
            ),
            shift_jis(literal(pipe())),
        ];
        for _ in 0..5000_u32 {
            let input = random_shift_jis(&mut rng);
            let starts = char_starts(&input, TextUnits::ShiftJis);
            for dialect in dialects {
                assert_eq!(rejoin(&input, dialect), input, "{dialect:?} {input:?}");
                Records::new(&input, dialect)
                    .for_each(|record| assert_cut_at_character_starts(&record, &starts));
            }
        }
    }

    /// 同じ表を書いた文字列の字。構造の記号・ASCII・ひらがな・サロゲートの組（U+20BB7）。
    const UTF16_ALPHABET: [&str; 10] = ["a", ",", "\"", "\r", "\n", "\t", "\\", "'", "あ", "𠮷"];

    fn random_text(rng: &mut Xorshift) -> alloc::string::String {
        let length = usize::try_from(rng.next() % 40).unwrap();
        (0..length)
            .map(|_| UTF16_ALPHABET[usize::try_from(rng.next() % 10).unwrap()])
            .collect()
    }

    fn utf16(text: &str, units: TextUnits) -> Vec<u8> {
        text.encode_utf16()
            .flat_map(|unit| match units {
                TextUnits::Utf16Be => unit.to_be_bytes(),
                TextUnits::Utf16Le | TextUnits::SingleByte | TextUnits::ShiftJis => {
                    unit.to_le_bytes()
                }
            })
            .collect()
    }

    fn from_utf16(bytes: &[u8], units: TextUnits) -> alloc::string::String {
        let words: Vec<u16> = bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|&pair| match units {
                TextUnits::Utf16Be => u16::from_be_bytes(pair),
                TextUnits::Utf16Le | TextUnits::SingleByte | TextUnits::ShiftJis => {
                    u16::from_le_bytes(pair)
                }
            })
            .collect();
        alloc::string::String::from_utf16(&words).unwrap()
    }

    /// 値のバイト列を、文字の単位に合わせて文字列にする。
    fn text_of(bytes: Vec<u8>, units: TextUnits) -> alloc::string::String {
        match units {
            TextUnits::Utf16Le | TextUnits::Utf16Be => from_utf16(&bytes, units),
            TextUnits::SingleByte | TextUnits::ShiftJis => {
                alloc::string::String::from_utf8(bytes).unwrap()
            }
        }
    }

    /// 表の値を文字列にし、レコードの形と改行も添える（文字コードに依らない比較のため）。
    fn decoded(
        input: &[u8],
        dialect: Dialect,
    ) -> Vec<(Vec<alloc::string::String>, RowShape, LineEnding)> {
        let units = dialect.units();
        Records::new(input, dialect)
            .map(|record| {
                let cells = record
                    .fields()
                    .iter()
                    .map(|&field| text_of(value(input, field, &dialect), units))
                    .collect();
                (cells, record.shape(), record.terminator())
            })
            .collect()
    }

    /// `text` を UTF-8 と UTF-16 の両方で切ると、同じ値・形・改行になり、UTF-16 はつなげ直すと元に戻る。
    fn assert_utf16_matches_utf8(text: &str, dialect: Dialect) {
        let expected = decoded(text.as_bytes(), dialect);
        for units in [TextUnits::Utf16Le, TextUnits::Utf16Be] {
            let wide = dialect.with_units(units);
            let input = utf16(text, units);
            assert_eq!(decoded(&input, wide), expected, "{wide:?} {text:?}");
            assert_eq!(rejoin_units(&input, wide), input, "{wide:?} {text:?}");
        }
    }

    #[test]
    fn utf16_cuts_the_same_table_as_utf8() {
        let mut rng = Xorshift(0x94D0_49BB_1331_11EB);
        for _ in 0..3000_u32 {
            let text = random_text(&mut rng);
            for dialect in dialects() {
                assert_utf16_matches_utf8(&text, dialect);
            }
        }
    }

    /// レコードの区間と、改行を単位で書いたものをつなげ直す。
    fn rejoin_units(input: &[u8], dialect: Dialect) -> Vec<u8> {
        let units = dialect.units();
        let mut out = Vec::new();
        for record in Records::new(input, dialect) {
            out.extend_from_slice(record.raw().slice(input).unwrap());
            for &byte in record.terminator().bytes() {
                units.push_ascii(&mut out, byte);
            }
        }
        out
    }

    #[test]
    fn an_odd_trailing_byte_ends_the_input() {
        let dialect = Dialect::rfc4180().with_units(TextUnits::Utf16Le);
        let mut input = utf16("a,b\nc", TextUnits::Utf16Le);
        input.push(0x41);
        let records: Vec<Record> = Records::new(&input, dialect).collect();
        assert_eq!(records.len(), 2);
        assert_eq!(records[1].raw().end(), input.len());
        assert_eq!(
            decoded(
                &utf16("x\"\"y", TextUnits::Utf16Be),
                Dialect::rfc4180().with_units(TextUnits::Utf16Be)
            )[0]
            .0,
            vec!["x\"\"y"]
        );
    }
}
