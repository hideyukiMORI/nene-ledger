//! 文字コード: BOM の後ろから切ること・表示のための復号・`encoding_rs` で作った標本での判別の突き合わせ。

// 🔴 中身を `#[cfg(test)]` の mod に入れる（reading.rs と同じ理由）。
#[cfg(test)]
mod encoding {
    use std::ops::ControlFlow;
    use std::path::PathBuf;

    use encoding_rs::{EUC_JP, SHIFT_JIS, UTF_16LE};
    use nene_ledger_core::dialect::Dialect;
    use nene_ledger_core::encoding_guess::EncodingGuess;
    use nene_ledger_core::text_encoding::TextEncoding;
    use nene_ledger_core::text_format::TextFormat;
    use nene_ledger_core::value::value;
    use nene_ledger_scan::block_size::BlockSize;
    use nene_ledger_scan::decoding::decode;
    use nene_ledger_scan::index_outcome::IndexOutcome;
    use nene_ledger_scan::indexing::build_index;
    use nene_ledger_scan::record_range::RecordRange;
    use nene_ledger_scan::source::Source;
    use nene_ledger_scan::sparse_index::SparseIndex;
    use nene_ledger_scan::windowing::read_records;

    fn fixture(name: &str, contents: &[u8]) -> PathBuf {
        let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("encoding-{name}.csv"));
        std::fs::write(&path, contents).unwrap();
        path
    }

    fn format(encoding: TextEncoding) -> TextFormat {
        TextFormat::new(encoding, Dialect::rfc4180())
    }

    fn index(source: &Source, format: TextFormat) -> SparseIndex {
        let outcome = build_index(source, format, BlockSize::new(4).unwrap(), |_| {
            ControlFlow::Continue(())
        })
        .unwrap();
        match outcome {
            IndexOutcome::Finished(index) => index,
            IndexOutcome::Cancelled => panic!("止まった"),
        }
    }

    /// 範囲の全セルを、引用符を外して `encoding` で復号した文字列。
    fn cells(source: &Source, format: TextFormat, count: usize) -> Vec<Vec<String>> {
        let index = index(source, format);
        let window = read_records(source, &index, format, RecordRange::new(0, count)).unwrap();
        window
            .records()
            .iter()
            .map(|record| {
                record
                    .record()
                    .fields()
                    .iter()
                    .map(|&field| {
                        let raw = value(record.bytes(), field, &format.dialect());
                        decode(format.encoding(), &raw).text().to_owned()
                    })
                    .collect()
            })
            .collect()
    }

    #[test]
    fn a_bom_is_skipped_so_the_first_quoted_cell_stays_quoted() {
        let input = "\u{FEFF}\"id\",\"名前\"\n1,\"東京\"\n".as_bytes();
        let source = Source::open(&fixture("bom", input)).unwrap();
        let utf8_bom = format(TextEncoding::Utf8WithBom);
        let index = index(&source, utf8_bom);
        assert_eq!((index.records(), index.marks()), (2, &[3_u64][..]));
        assert_eq!(
            cells(&source, utf8_bom, 5),
            vec![vec!["id", "名前"], vec!["1", "東京"]]
        );
        // BOM が無いと読むと、BOM が先頭のセルに残り、そのセルは引用として切られない。
        let plain = cells(&source, format(TextEncoding::Utf8), 1);
        assert_eq!(plain[0][0], "\u{FEFF}\"id\"");
    }

    #[test]
    fn a_bom_that_is_not_there_is_not_skipped() {
        let source = Source::open(&fixture("no-bom", b"a,b\n")).unwrap();
        assert_eq!(source.content_start(TextEncoding::Utf8WithBom).unwrap(), 0);
        assert_eq!(
            cells(&source, format(TextEncoding::Utf8WithBom), 1),
            vec![vec!["a", "b"]]
        );
    }

    #[test]
    fn shift_jis_and_euc_jp_cells_are_decoded_for_display() {
        let text = "コード,名前\n1,\"日本語,表\"\n";
        for (table, encoding) in [
            (SHIFT_JIS, TextEncoding::ShiftJis),
            (EUC_JP, TextEncoding::EucJp),
        ] {
            let (bytes, _, unmappable) = table.encode(text);
            assert!(!unmappable);
            let source = Source::open(&fixture(table.name(), &bytes)).unwrap();
            assert_eq!(
                cells(&source, format(encoding), 5),
                vec![vec!["コード", "名前"], vec!["1", "日本語,表"]],
                "{}",
                table.name()
            );
        }
    }

    #[test]
    fn undecodable_bytes_are_reported() {
        let broken = decode(TextEncoding::ShiftJis, b"a\x81\x20b");
        assert!(broken.malformed());
        assert!(broken.text().contains('\u{FFFD}'));
        assert!(!decode(TextEncoding::Utf8, "東京".as_bytes()).malformed());
        let (utf16, _, _) = UTF_16LE.encode("東京");
        // encoding_rs は UTF-16 へ符号化しない（WHATWG の定め）ので、UTF-16 の標本は手で作る。
        assert_eq!(utf16.as_ref(), "東京".as_bytes());
        assert_eq!(
            decode(TextEncoding::Utf16Le, b"\x71\x67\xAC\x4E").text(),
            "東京"
        );
    }

    #[test]
    fn hand_written_samples_in_core_match_encoding_rs() {
        assert_eq!(
            SHIFT_JIS.encode("日本語").0.as_ref(),
            b"\x93\xFA\x96\x7B\x8C\xEA"
        );
        assert_eq!(
            EUC_JP.encode("日本語").0.as_ref(),
            b"\xC6\xFC\xCB\xDC\xB8\xEC"
        );
        assert_eq!(SHIFT_JIS.encode("ｱｲｳｴ").0.as_ref(), b"\xB1\xB2\xB3\xB4");
        assert_eq!(
            EUC_JP.encode("あいう").0.as_ref(),
            b"\xA4\xA2\xA4\xA4\xA4\xA6"
        );
        assert_eq!(SHIFT_JIS.encode("表").0.as_ref(), b"\x95\x5C");
    }

    #[test]
    fn detection_agrees_with_samples_made_by_encoding_rs() {
        let text = "社員番号,氏名,所属\n1001,山田太郎,経理部\n1002,鈴木花子,営業部\n";
        let sjis = SHIFT_JIS.encode(text).0;
        assert_eq!(EncodingGuess::of(&sjis).best(), TextEncoding::ShiftJis);
        assert!(EncodingGuess::of(&sjis).confident());
        let euc = EUC_JP.encode(text).0;
        assert_eq!(EncodingGuess::of(&euc).best(), TextEncoding::EucJp);
        assert!(EncodingGuess::of(&euc).confident());
        assert_eq!(
            EncodingGuess::of(text.as_bytes()).best(),
            TextEncoding::Utf8
        );
    }

    /// `text` を BOM 付きの UTF-16LE で書いたバイト列。`odd` なら奇数の長さにする 1 バイトを足す。
    fn utf16le_with_bom(text: &str, odd: bool) -> Vec<u8> {
        let mut bytes = vec![0xFF_u8, 0xFE];
        bytes.extend(text.encode_utf16().flat_map(u16::to_le_bytes));
        if odd {
            bytes.push(b'x');
        }
        bytes
    }

    #[test]
    fn utf16le_tabs_with_a_bom_are_cut_and_decoded() {
        // Excel の「Unicode テキスト」と同じ形（BOM 付きの UTF-16LE・タブ区切り・CRLF）。
        let text = "コード\t品名\r\n1\t\"表計算\tソフト\"\r\n2\t𠮷野家\r\n";
        let source = Source::open(&fixture("utf16le", &utf16le_with_bom(text, false))).unwrap();
        let tab = Dialect::new(
            nene_ledger_core::delimiter::Delimiter::TAB,
            nene_ledger_core::quoting::Quoting::Doubled,
            b'"',
        )
        .unwrap();
        let utf16 = TextFormat::new(TextEncoding::Utf16LeWithBom, tab);
        let index = index(&source, utf16);
        // BOM の 2 バイトの後ろから切り、目印は単位の頭（2）に置く。
        assert_eq!((index.records(), index.marks()), (3, &[2_u64][..]));
        assert_eq!(
            cells(&source, utf16, 5),
            vec![
                vec!["コード", "品名"],
                vec!["1", "表計算\tソフト"],
                vec!["2", "𠮷野家"]
            ]
        );
        // 2 行目からの範囲も、目印から読み直して同じ値。
        let window = read_records(&source, &index, utf16, RecordRange::new(1, 1)).unwrap();
        assert_eq!(window.records().len(), 1);
    }

    #[test]
    fn an_odd_trailing_byte_is_kept_as_its_own_record() {
        let source = Source::open(&fixture("utf16odd", &utf16le_with_bom("a,b\n", true))).unwrap();
        let utf16 = format(TextEncoding::Utf16LeWithBom);
        let index = index(&source, utf16);
        assert_eq!(index.records(), 2);
        let window = read_records(&source, &index, utf16, RecordRange::new(0, 5)).unwrap();
        assert_eq!(window.records().len(), 2);
    }

    /// 1 字ぶんのバイト列を復号し、読めれば 1 字の値と元のバイト数。
    fn one_character(encoding: TextEncoding, bytes: &[u8]) -> Option<(String, u64)> {
        let decoded = decode(encoding, bytes);
        let text = decoded.text();
        (!decoded.malformed() && text.chars().count() == 1)
            .then(|| (String::from(text), u64::try_from(bytes.len()).unwrap()))
    }

    /// `encoding` で復号できる全部の 1 バイトと 2 バイトの 1 字で、core の `encoded_length` と元のバイト数を突き合わせ、
    /// 確かめた字の数を返す。EUC-JP の 3 バイト（`8F`・補助漢字）は 2 と数えると決めたので除く。
    fn check_lengths(encoding: TextEncoding) -> u32 {
        let singles = (0_u8..=0xFF).map(|byte| vec![byte]);
        let pairs = (0x80_u8..=0xFF)
            .filter(|&lead| !(encoding == TextEncoding::EucJp && lead == 0x8F))
            .flat_map(|lead| (0x40_u8..=0xFF).map(move |trail| vec![lead, trail]));
        let mut checked = 0_u32;
        for (text, length) in singles
            .chain(pairs)
            .filter_map(|bytes| one_character(encoding, &bytes))
        {
            assert_eq!(
                encoding.encoded_length(&text),
                length,
                "{encoding:?} {text}"
            );
            checked += 1;
        }
        checked
    }

    /// 詳細ペインのバイト数（core の `encoded_length`・表を引かない）が、`encoding_rs` で復号できる字の元のバイト数と
    /// 一致する（Issue #59）。
    #[test]
    fn encoded_lengths_match_every_decodable_character() {
        assert!(check_lengths(TextEncoding::ShiftJis) > 7_000);
        assert!(check_lengths(TextEncoding::EucJp) > 6_000);
        assert_eq!(check_lengths(TextEncoding::Windows1252), 256);
    }
}
