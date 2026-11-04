//! 塊を順に受け取って、まばらな索引を作る（ファイルを知らない純粋な部分）。

use nene_ledger_core::dialect::Dialect;

use crate::record_starts::RecordStarts;
use crate::scan_failure::ScanFailure;
use crate::sparse_index::{MARK_INTERVAL, SparseIndex};
use crate::window_record::RECORD_READ_LIMIT;

/// 塊を順に受け取って、まばらな索引を作る。
///
/// 塊の境目では切り出しの状態（`RecordStarts`）だけを持ち越し、バイト列は持たない。
/// 閉じない引用符や巨大なセルがあっても記憶は目印の数にしか比例せず（BIG-04）、各バイトを 1 回だけ見る。
/// 🔴 塊は `position` の位置から順に渡す。`UnclosedQuote::Literal` で `reread_literally` が位置を返したら、そこから渡し直す。
#[derive(Debug, Clone)]
pub struct IndexBuilder {
    starts: RecordStarts,
    marks: Vec<u64>,
    records: u64,
    bytes: u64,
    unclosed_quotes: Vec<u64>,
}

impl IndexBuilder {
    /// `dialect` で切る索引を、バイト位置 `start`（BOM の後ろ・最初のレコードの始まり）から作り始める。
    #[must_use]
    pub fn new(dialect: Dialect, start: u64) -> Self {
        Self {
            starts: RecordStarts::new(dialect, start),
            marks: Vec::new(),
            records: 0,
            bytes: 0,
            unclosed_quotes: Vec::new(),
        }
    }

    /// ここまでに数えたレコードの数。
    #[must_use]
    pub fn records(&self) -> u64 {
        self.records
    }

    /// 次に渡す塊の位置。
    #[must_use]
    pub fn position(&self) -> u64 {
        self.starts.position()
    }

    /// 次の塊（`position` から続くバイト列）を受け取る。
    ///
    /// # Errors
    ///
    /// バイト位置が `u64` に収まらないとき。
    pub fn feed(&mut self, block: &[u8]) -> Result<(), ScanFailure> {
        for &byte in block {
            if let Some(start) = self.starts.step(byte) {
                self.count_record(start);
            }
        }
        self.bytes = self.bytes.max(self.starts.position());
        Ok(())
    }

    /// `start` から始まるレコードを数え、間隔ごとに目印を置く。
    fn count_record(&mut self, start: u64) {
        if self.records.is_multiple_of(MARK_INTERVAL) {
            self.marks.push(start);
        }
        self.records = self.records.saturating_add(1);
    }

    /// 今読んでいる引用の中のセルが `RECORD_READ_LIMIT` を超えていれば、その引用符の位置（ADR 0018 の決定 1）。
    ///
    /// # Errors
    ///
    /// 上限が `u64` に収まらないとき。
    pub fn long_quoted_cell(&self) -> Result<Option<u64>, ScanFailure> {
        let limit = u64::try_from(RECORD_READ_LIMIT.get())?;
        Ok(self.starts.long_quoted_cell(limit))
    }

    /// 入力の終わりで呼ぶ。引用符が閉じておらず方言が `Literal` なら、その引用符を文字とみなし、
    /// 読み直す位置（引用符の次）を返す。呼び出し側はそこから塊を渡し直し、また入力の終わりで呼ぶ。
    pub fn reread_literally(&mut self) -> Option<u64> {
        let quote = self.starts.unclosed_quote()?;
        let from = self.starts.resume_literally()?;
        self.unclosed_quotes.push(quote);
        Some(from)
    }

    /// 入力の終わり。索引を返す。最後のレコードは、その最初のバイトを受け取った時に数えてある。
    #[must_use]
    pub fn finish(mut self) -> SparseIndex {
        self.unclosed_quotes.extend(self.starts.unclosed_quote());
        if let Some(start) = self.starts.finish_input() {
            self.count_record(start);
        }
        SparseIndex::new(self.marks, self.records, self.bytes)
            .with_unclosed_quotes(self.unclosed_quotes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nene_ledger_core::delimiter::Delimiter;
    use nene_ledger_core::quoting::Quoting;
    use nene_ledger_core::record::Record;
    use nene_ledger_core::records::Records;
    use nene_ledger_core::row_shape::RowShape;
    use nene_ledger_core::text_units::TextUnits;
    use nene_ledger_core::unclosed_quote::UnclosedQuote;

    /// 全体を一度に参照実装で切ったときの索引（正本）。
    fn reference(input: &[u8], dialect: Dialect) -> SparseIndex {
        let mut marks = Vec::new();
        let mut records = 0_u64;
        let mut quotes = Vec::new();
        for record in Records::new(input, dialect) {
            if records.is_multiple_of(MARK_INTERVAL) {
                marks.push(u64::try_from(record.raw().start()).unwrap());
            }
            records += 1;
            quotes.extend(unclosed_quotes_of(&record, input, dialect));
        }
        SparseIndex::new(marks, records, u64::try_from(input.len()).unwrap())
            .with_unclosed_quotes(quotes)
    }

    /// 参照実装のレコードの中で、閉じなかった引用符（`ToEnd`）か文字として読んだ引用符（`Literal`）の位置。
    fn unclosed_quotes_of(record: &Record, input: &[u8], dialect: Dialect) -> Vec<u64> {
        record
            .fields()
            .iter()
            .filter(|field| match record.shape() {
                RowShape::UnclosedQuote => field.quoted() && field.raw().end() == input.len(),
                RowShape::LiteralQuote => {
                    !field.quoted()
                        && dialect
                            .units()
                            .is_ascii_at(input, field.raw().start(), dialect.quote())
                }
                RowShape::Complete => false,
            })
            .map(|field| u64::try_from(field.raw().start()).unwrap())
            .collect()
    }

    /// `size` バイトずつ渡し、入力の終わりで `Literal` の読み直しがあれば、その位置から渡し直す。
    fn in_blocks(input: &[u8], dialect: Dialect, size: usize) -> SparseIndex {
        let mut builder = IndexBuilder::new(dialect, 0);
        let mut from = 0_usize;
        loop {
            for block in input[from..].chunks(size) {
                builder.feed(block).unwrap();
            }
            let Some(next) = builder.reread_literally() else {
                return builder.finish();
            };
            from = usize::try_from(next).unwrap();
        }
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

        fn below(&mut self, bound: u64) -> usize {
            usize::try_from(self.next() % bound).unwrap()
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

    fn random_input(rng: &mut Xorshift, length: usize) -> Vec<u8> {
        let mut out = Vec::new();
        for _ in 0..length {
            out.extend_from_slice(ALPHABET[rng.below(9)]);
        }
        out
    }

    fn tsv(quoting: Quoting) -> Dialect {
        Dialect::new(Delimiter::TAB, quoting, b'"').unwrap()
    }

    fn dialects() -> Vec<Dialect> {
        vec![
            Dialect::rfc4180(),
            Dialect::new(Delimiter::COMMA, Quoting::QuotedBackslash, b'"').unwrap(),
            Dialect::new(Delimiter::COMMA, Quoting::Doubled, b'\'').unwrap(),
            tsv(Quoting::Doubled),
            tsv(Quoting::Backslash),
            tsv(Quoting::Plain),
            Dialect::rfc4180().with_unclosed_quote(UnclosedQuote::Literal),
            Dialect::new(Delimiter::COMMA, Quoting::QuotedBackslash, b'"')
                .unwrap()
                .with_unclosed_quote(UnclosedQuote::Literal),
        ]
    }

    const BLOCK_SIZES: [usize; 7] = [1, 2, 3, 5, 8, 64, 4096];

    #[test]
    fn empty_input_has_no_records_and_no_marks() {
        let index = in_blocks(b"", Dialect::rfc4180(), 4);
        assert_eq!(index, SparseIndex::new(Vec::new(), 0, 0));
    }

    #[test]
    fn crlf_split_between_blocks_is_one_line_ending() {
        let input = b"a\r\nb\r\nc";
        for size in 1..=input.len() {
            assert_eq!(in_blocks(input, Dialect::rfc4180(), size).records(), 3);
        }
    }

    #[test]
    fn line_break_inside_quotes_across_blocks_does_not_split_the_record() {
        let input = "1,\"東京都\n千代田区\",x\n2,y,z\n".as_bytes();
        for size in 1..=input.len() {
            let index = in_blocks(input, Dialect::rfc4180(), size);
            assert_eq!(index, reference(input, Dialect::rfc4180()), "{size}");
            assert_eq!(index.records(), 2);
        }
    }

    /// どの方言・どの塊の大きさでも、全体を一度に切った参照実装と同じ索引になる（QLT-008 の 1・2）。
    fn assert_matches_reference(input: &[u8]) {
        for dialect in dialects() {
            let expected = reference(input, dialect);
            for size in BLOCK_SIZES {
                assert_eq!(
                    in_blocks(input, dialect, size),
                    expected,
                    "{dialect:?} {size} {input:?}"
                );
            }
        }
    }

    #[test]
    fn short_random_inputs_match_the_reference_for_every_block_size() {
        let mut rng = Xorshift(0x9E37_79B9_7F4A_7C15);
        for _ in 0..2000_u32 {
            let length = rng.below(40);
            assert_matches_reference(&random_input(&mut rng, length));
        }
    }

    #[test]
    fn long_random_inputs_place_marks_like_the_reference() {
        let mut rng = Xorshift(0xD1B5_4A32_D192_ED03);
        for _ in 0..6_u32 {
            assert_matches_reference(&random_input(&mut rng, 30_000));
        }
    }

    #[test]
    fn many_records_get_one_mark_per_interval() {
        let input = b"a,b\n".repeat(5000);
        let index = in_blocks(&input, Dialect::rfc4180(), 1000);
        assert_eq!(index.records(), 5000);
        assert_eq!(index.marks(), &[0, 4096, 8192, 12288, 16384]);
        assert_eq!(index, reference(&input, Dialect::rfc4180()));
    }

    #[test]
    fn unclosed_quote_fed_byte_by_byte_stays_one_record() {
        let mut input = b"a\n\"".to_vec();
        input.extend(b"x,y\n".repeat(50_000));
        let index = in_blocks(&input, Dialect::rfc4180(), 1);
        assert_eq!(index.records(), 2);
        assert_eq!(index.unclosed_quotes(), &[2]);
        assert_eq!(index, reference(&input, Dialect::rfc4180()));
    }

    #[test]
    fn literal_rereads_from_the_quote_and_counts_every_line() {
        let mut input = b"a\n\"".to_vec();
        input.extend(b"x,y\n".repeat(50_000));
        let dialect = Dialect::rfc4180().with_unclosed_quote(UnclosedQuote::Literal);
        let index = in_blocks(&input, dialect, 4096);
        assert_eq!(index.records(), 50_001);
        assert_eq!(index.unclosed_quotes(), &[2]);
        assert_eq!(index.bytes(), u64::try_from(input.len()).unwrap());
        assert_eq!(index, reference(&input, dialect));
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

    /// どの塊の大きさでも、全体を一度に切った参照実装と同じ索引になる。
    fn assert_blocks_match_reference(input: &[u8], dialect: Dialect) {
        let expected = reference(input, dialect);
        for size in BLOCK_SIZES {
            assert_eq!(
                in_blocks(input, dialect, size),
                expected,
                "{dialect:?} {size} {input:?}"
            );
        }
    }

    #[test]
    fn shift_jis_inputs_match_the_reference_for_every_block_size() {
        // 1 バイト目と 2 バイト目が塊の境目で分かれても、同じ索引になる。
        let mut rng = Xorshift(0x1405_7B7E_F767_814F);
        for _ in 0..1500_u32 {
            let length = rng.below(40);
            let mut input = Vec::new();
            for _ in 0..length {
                input.extend_from_slice(SHIFT_JIS_ALPHABET[rng.below(11)]);
            }
            for dialect in shift_jis_dialects() {
                assert_blocks_match_reference(&input, dialect);
            }
        }
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

    fn assert_every_dialect_matches(input: &[u8], dialects: &[Dialect]) {
        for &dialect in dialects {
            assert_blocks_match_reference(input, dialect);
        }
    }

    #[test]
    fn utf16_inputs_match_the_reference_for_every_block_size() {
        // 単位の 2 バイトが塊の境目で分かれても（大きさ 1・3・5）、同じ索引になる。
        let mut rng = Xorshift(0x9FB2_1C65_1E98_DF25);
        for round in 0..800_u32 {
            let length = rng.below(40);
            let picks: Vec<usize> = (0..length).map(|_| rng.below(10)).collect();
            for units in [TextUnits::Utf16Le, TextUnits::Utf16Be] {
                let input = utf16_input(&picks, units, round % 5 == 0);
                assert_every_dialect_matches(&input, &utf16_dialects(units));
            }
        }
    }
}
