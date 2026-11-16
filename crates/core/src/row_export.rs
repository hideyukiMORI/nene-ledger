//! 表の 1 行を、選んだ方言で書く規則（方言を変えた書き出し・ADR 0036・SAVE-03）。
//!
//! 保存（原本と同じ方言で、触っていないセルは原本のバイト列を写す・ADR 0024）とは別の道で、行の全部のセルを値から
//! 書き直す。セルの書き方の正本は [`crate::cell_writing::written_cell`]。

use alloc::string::String;

use crate::cell_writing::{CellWriteFailure, written_cell};
use crate::dialect::Dialect;
use crate::line_ending::LineEnding;
use crate::quote_policy::QuotePolicy;

/// 行のセルの値 `cells` を、方言 `dialect`・引用符の付け方 `policy` で 1 行の文字列にする（行の終わりは付けない）。
/// セルの中の改行は `line_break` に揃える。セルが 1 つも無い行は空の文字列。
///
/// # Errors
///
/// 引用もエスケープもしない方言で、値が区切りか改行を含むとき [`CellWriteFailure::NeedsQuoting`]。
pub fn exported_row<S: AsRef<str>>(
    cells: &[S],
    (dialect, policy): (&Dialect, QuotePolicy),
    line_break: LineEnding,
) -> Result<String, CellWriteFailure> {
    let delimiter = char::from(dialect.delimiter().byte());
    let quoted = policy == QuotePolicy::All;
    let mut row = String::new();
    for (index, cell) in cells.iter().enumerate() {
        if index > 0 {
            row.push(delimiter);
        }
        row.push_str(&written_cell(cell.as_ref(), quoted, dialect, line_break)?);
    }
    Ok(row)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::delimiter::Delimiter;
    use crate::quoting::Quoting;
    use crate::records::Records;
    use crate::value::value;
    use alloc::vec::Vec;

    fn dialect(quoting: Quoting, delimiter: u8) -> Dialect {
        Dialect::new(Delimiter::new(delimiter).unwrap(), quoting, b'"').unwrap()
    }

    #[test]
    fn cells_are_joined_and_quoted_by_the_policy() {
        let comma = dialect(Quoting::Doubled, b',');
        let cells = ["0012", "a,b", "", "say \"hi\""];
        assert_eq!(
            exported_row(&cells, (&comma, QuotePolicy::AsNeeded), LineEnding::CrLf).unwrap(),
            "0012,\"a,b\",,\"say \"\"hi\"\"\""
        );
        assert_eq!(
            exported_row(&cells, (&comma, QuotePolicy::All), LineEnding::CrLf).unwrap(),
            "\"0012\",\"a,b\",\"\",\"say \"\"hi\"\"\""
        );
        let tab = dialect(Quoting::Doubled, b'\t');
        assert_eq!(
            exported_row(&cells, (&tab, QuotePolicy::AsNeeded), LineEnding::Lf).unwrap(),
            "0012\ta,b\t\t\"say \"\"hi\"\"\""
        );
        let none: [&str; 0] = [];
        assert_eq!(
            exported_row(&none, (&comma, QuotePolicy::All), LineEnding::Lf).unwrap(),
            ""
        );
        let plain = dialect(Quoting::Plain, b',');
        assert_eq!(
            exported_row(&["a,b"], (&plain, QuotePolicy::All), LineEnding::Lf),
            Err(CellWriteFailure::NeedsQuoting)
        );
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

    /// 無作為のセルの値。区切り・引用符・バックスラッシュ・空白・日本語と、行の終わり方に揃えた改行を混ぜる。
    fn random_cell(rng: &mut Xorshift, line_break: &str) -> String {
        const PARTS: [&str; 12] = [
            "a", "0012", ",", "\t", "\"", "\\", " ", "東京", "'", ";", "1.20E+03", "|",
        ];
        let mut cell = String::new();
        for _ in 0..rng.below(5) {
            match rng.below(14) {
                13 => cell.push_str(line_break),
                index => cell.push_str(PARTS[index.min(11)]),
            }
        }
        cell
    }

    /// 書いた行を、同じ方言で参照実装に読ませたセルの値。
    fn read_back(row: &str, ending: &str, dialect: Dialect) -> Vec<String> {
        let mut input = String::from(row);
        input.push_str(ending);
        let bytes = input.as_bytes();
        let record = Records::new(bytes, dialect).next().unwrap();
        record
            .fields()
            .iter()
            .map(|&field| String::from_utf8(value(bytes, field, &dialect)).unwrap())
            .collect()
    }

    /// 書いた行を同じ方言で読み戻すと、元のセルの値に戻る（乱数の値 × 方言 × 引用符の付け方 × 改行・QLT-008）。
    #[test]
    fn exported_rows_read_back_as_the_same_cells() {
        let dialects = [
            dialect(Quoting::Doubled, b','),
            dialect(Quoting::Doubled, b'\t'),
            dialect(Quoting::Doubled, b';'),
            dialect(Quoting::QuotedBackslash, b','),
        ];
        let endings = [(LineEnding::Lf, "\n"), (LineEnding::CrLf, "\r\n")];
        let mut rng = Xorshift(0x2545_F491_4F6C_DD1D);
        for round in 0..4_000_usize {
            let dialect = dialects[round % dialects.len()];
            let (line_break, ending) = endings[rng.below(2)];
            let policy = [QuotePolicy::AsNeeded, QuotePolicy::All][rng.below(2)];
            let cells: Vec<String> = (0..=rng.below(5))
                .map(|_| random_cell(&mut rng, ending))
                .collect();
            let row = exported_row(&cells, (&dialect, policy), line_break).unwrap();
            assert_eq!(
                read_back(&row, ending, dialect),
                cells,
                "{row:?} {policy:?}"
            );
        }
    }
}
