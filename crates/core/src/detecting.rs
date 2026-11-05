//! 方言の判別の計算（純関数）。試しに切った結果の一貫性で候補に点を付ける。
//!
//! 着想は CleverCSV（van den Burg ほか 2019）の「行の形の一貫性 × セルの形の一貫性」。
//! 点数は千分率の整数で、浮動小数を使わない（決定的・RS-007）。
//!
//! 🔴 各候補は 2 通りに試し切りし、点の高いほうを採る（同点なら厳格・Issue #23）。
//! - 厳格（`ToEnd`）: 閉じない引用符から標本の終わりまでが 1 つのレコードになり、標本が途中で切れていれば捨てる。
//!   標本の終わりで切れた正しい長いセルはこちらが正しく読む
//! - 閉じない引用符を文字として読む（`Literal`）: 迷子の引用符が標本の先頭近くにあっても、その後ろの行で点を
//!   付けられる。文字として読み直したレコードの割合だけ点を減らす
//!
//! 厳格だけだと、迷子の引用符 1 つで引用を使う候補が前の行だけで点を付けられ、引用を使わない候補に負ける。
//! 返す方言は、どちらで試しても厳格なまま（ADR 0018 の既定）。

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::cell_kind::CellKind;
use crate::delimiter::Delimiter;
use crate::dialect::Dialect;
use crate::dialect_candidate::DialectCandidate;
use crate::field::Field;
use crate::header_guess::HeaderGuess;
use crate::quoting::Quoting;
use crate::record::Record;
use crate::records::Records;
use crate::row_shape::RowShape;
use crate::scanning::trails_closing_quote;
use crate::text_encoding::LONGEST_BOM;
use crate::unclosed_quote::UnclosedQuote;
use crate::value::value;

/// 判別に読む上限（OPEN-17）。
pub const SAMPLE_LIMIT: usize = 65_536;

/// 判別のために原本の先頭から読む量。標本（[`SAMPLE_LIMIT`]）に、いちばん長い BOM と 1 バイトを足す。
///
/// 判別は「入力が [`SAMPLE_LIMIT`] を超えるか」で標本の後ろに続きがあるかを知り、続きがあれば標本の末尾で切れた字や
/// レコードを壊れたとみなさない。🔴 ちょうど [`SAMPLE_LIMIT`] だけ読んで渡すと、64 KiB を超えるファイルで末尾の字が
/// 切れたとき、文字コードを取り違える（GitHub Issue #10）。BOM の後ろを方言の判別に渡しても超えるように BOM の分を足す。
pub const READ_LIMIT: usize = SAMPLE_LIMIT + LONGEST_BOM + 1;

/// 前書きとみなす行の数の上限。
pub const MAX_PREAMBLE_ROWS: usize = 32;

/// 前書きを判じる表の、列の数の下限。1〜2 列の表では「中身の少ない行」と前書きを見分けられない。
const MIN_PREAMBLE_COLUMNS: usize = 3;

/// 区切りの候補。並びは同点のときの優先の順。
const DELIMITERS: [u8; 5] = *b",\t;|^";

/// 引用の流儀の候補（流儀・引用符）。並びは同点のときの優先の順。
const QUOTINGS: [(Quoting, u8); 5] = [
    (Quoting::Doubled, b'"'),
    (Quoting::Doubled, b'\''),
    (Quoting::QuotedBackslash, b'"'),
    (Quoting::Backslash, b'"'),
    (Quoting::Plain, b'"'),
];

/// 判別が見るセルの値。文字の単位に依らず同じ規則で判断できるよう、ASCII でない文字を 1 つの印に狭める（ADR 0019）。
fn judged(sample: &[u8], field: Field, dialect: Dialect) -> Vec<u8> {
    dialect.units().narrow(&value(sample, field, &dialect))
}

/// 全ての候補の方言を、優先の順に並べる。
#[must_use]
pub fn candidate_dialects() -> Vec<Dialect> {
    DELIMITERS
        .iter()
        .filter_map(|&byte| Delimiter::new(byte).ok())
        .flat_map(|delimiter| {
            QUOTINGS
                .iter()
                .filter_map(move |&(quoting, quote)| Dialect::new(delimiter, quoting, quote).ok())
        })
        .collect()
}

/// 標本を切る。標本が途中で切れていれば（`complete` が偽なら）、最後のレコードは捨てる。
#[must_use]
pub fn sample_records(sample: &[u8], dialect: Dialect, complete: bool) -> Vec<Record> {
    let mut records: Vec<Record> = Records::new(sample, dialect).collect();
    if !complete && records.len() > 1 {
        records.pop();
    }
    records
}

/// 千分率の割り算（分母が 0 なら 0）。
fn permille(numerator: usize, denominator: usize) -> u32 {
    let value = numerator
        .saturating_mul(1000)
        .checked_div(denominator)
        .unwrap_or(0);
    u32::try_from(value).unwrap_or(u32::MAX)
}

/// 行の形の一貫性（千分率）。列の数ごとの行の数 `n_k` について `Σ n_k (k−1)/k` を、
/// 列の数の種類 K と行の数で割る。1 列だけの行は 0 点（区切りが効いていない）。
#[must_use]
pub fn pattern_score(records: &[Record]) -> u32 {
    let mut groups: BTreeMap<usize, usize> = BTreeMap::new();
    for record in records {
        let entry = groups.entry(record.fields().len()).or_insert(0);
        *entry = entry.saturating_add(1);
    }
    let weighted: usize = groups
        .iter()
        .map(|(&columns, &rows)| {
            rows.saturating_mul(columns.saturating_sub(1))
                .saturating_mul(1000)
                .checked_div(columns)
                .unwrap_or(0)
        })
        .sum();
    let scale = groups.len().saturating_mul(records.len());
    u32::try_from(weighted.checked_div(scale).unwrap_or(0)).unwrap_or(u32::MAX)
}

/// セルの形のもっともらしさ（千分率）。`Other` でないセルの割合。
#[must_use]
pub fn type_score(sample: &[u8], records: &[Record], dialect: &Dialect) -> u32 {
    let (plausible, total) = records
        .iter()
        .flat_map(|record| record.fields().iter())
        .fold((0_usize, 0_usize), |(plausible, total), &field| {
            let kind = CellKind::of(&judged(sample, field, *dialect));
            (
                plausible.saturating_add(usize::from(kind.is_plausible())),
                total.saturating_add(1),
            )
        });
    permille(plausible, total)
}

/// いちばん多い列の数。同数なら少ないほう。
#[must_use]
pub fn modal_columns(records: &[Record]) -> usize {
    let mut groups: BTreeMap<usize, usize> = BTreeMap::new();
    for record in records {
        let entry = groups.entry(record.fields().len()).or_insert(0);
        *entry = entry.saturating_add(1);
    }
    groups
        .iter()
        .fold(
            (0_usize, 0_usize),
            |(best, best_rows), (&columns, &rows)| {
                if rows > best_rows {
                    (columns, rows)
                } else {
                    (best, best_rows)
                }
            },
        )
        .0
}

/// 閉じた引用符の後ろに文字が続いたセルを含むか（`"a"b`・Issue #60）。
fn trails(sample: &[u8], record: &Record, dialect: Dialect) -> bool {
    record
        .fields()
        .iter()
        .any(|&field| trails_closing_quote(sample, field, &dialect))
}

/// 1 つの読み方の点。閉じない引用符の行が残れば 0 点。文字として読み直したレコードと、閉じた引用符の後ろに
/// 文字が続いたセルを含むレコードの割合だけ点を減らす。
///
/// 🔴 後者は、引用の中に区切りの無い `\"` のファイルを `""` の流派で読むと、行の形もセルの形も崩れずに
/// 同点になるため（Issue #60）。どちらの流派の読み違いにも同じだけ効く。
fn points(sample: &[u8], dialect: Dialect, complete: bool) -> u32 {
    let records = sample_records(sample, dialect, complete);
    if records
        .iter()
        .any(|record| record.shape() == RowShape::UnclosedQuote)
    {
        return 0;
    }
    let doubtful = records
        .iter()
        .filter(|record| {
            record.shape() == RowShape::LiteralQuote || trails(sample, record, dialect)
        })
        .count();
    let consistency = pattern_score(&records)
        .saturating_mul(type_score(sample, &records, &dialect))
        .checked_div(1000)
        .unwrap_or(0);
    let kept = permille(records.len().saturating_sub(doubtful), records.len());
    consistency
        .saturating_mul(kept)
        .checked_div(1000)
        .unwrap_or(0)
}

/// 候補 `dialect` の試し切りに使う読み方。厳格と、閉じない引用符を文字として読むのとで、点の高いほう（同点なら厳格）。
#[must_use]
pub fn trial_dialect(sample: &[u8], dialect: Dialect, complete: bool) -> Dialect {
    let lenient = dialect.with_unclosed_quote(UnclosedQuote::Literal);
    if points(sample, lenient, complete) > points(sample, dialect, complete) {
        lenient
    } else {
        dialect
    }
}

/// 1 つの候補に点を付ける。試し切りは `trial_dialect` の読み方で、返す方言は `dialect`（厳格）のまま。
#[must_use]
pub fn score(sample: &[u8], dialect: Dialect, complete: bool) -> DialectCandidate {
    let trial = trial_dialect(sample, dialect, complete);
    let records = sample_records(sample, trial, complete);
    DialectCandidate::new(
        dialect,
        points(sample, trial, complete),
        modal_columns(&records),
    )
}

/// 列ごとに、先頭の行のセルの形と、それより下のセルの形の多数派を比べる。
fn header_votes(sample: &[u8], records: &[Record], dialect: Dialect) -> (usize, usize) {
    let Some((head, body)) = records.split_first() else {
        return (0, 0);
    };
    head.fields().iter().enumerate().fold(
        (0_usize, 0_usize),
        |(present, absent), (column, &field)| {
            let heading = CellKind::of(&judged(sample, field, dialect));
            match column_majority(sample, body, dialect, column) {
                Some(kind) if heading == kind => (present, absent.saturating_add(1)),
                Some(_) if heading == CellKind::Word => (present.saturating_add(1), absent),
                Some(_) | None => (present, absent),
            }
        },
    )
}

/// 列の下のセルのうち、数か日付に見えるものが 8 割以上なら、その形を返す。
fn column_majority(
    sample: &[u8],
    body: &[Record],
    dialect: Dialect,
    column: usize,
) -> Option<CellKind> {
    let kinds: Vec<CellKind> = body
        .iter()
        .filter_map(|record| record.fields().get(column))
        .map(|&field| CellKind::of(&judged(sample, field, dialect)))
        .collect();
    [CellKind::Number, CellKind::DateLike]
        .into_iter()
        .find(|&wanted| {
            let hits = kinds.iter().filter(|&&kind| kind == wanted).count();
            !kinds.is_empty() && permille(hits, kinds.len()) >= 800
        })
}

/// 切り方の指紋（各レコードの各セルの値）。同じ指紋の候補は、そのファイルでは同じ切り方になる。
#[must_use]
pub fn segmentation(sample: &[u8], dialect: Dialect, complete: bool) -> Vec<Vec<Vec<u8>>> {
    sample_records(sample, trial_dialect(sample, dialect, complete), complete)
        .iter()
        .map(|record| {
            record
                .fields()
                .iter()
                .map(|&field| judged(sample, field, dialect))
                .collect()
        })
        .collect()
}

/// 見出しの前の前書きの行の数（OPEN-11・Issue #25）。
///
/// 先頭から次の 3 つを満たす行を前書きとし、満たさない行で止める。
/// - 中身のある（空白だけでない）セルが、使われている列の幅（`used_width`）の半分に満たない
/// - 表題らしい: 中身のあるセルが 1 つ以下か、左端から始まり `PREAMBLE_FILLED_CELLS`（3）個以下（表題と右端の単位・
///   ページ・作成日）。🔴 段に分けた見出し（`,総数,病院,,,診療所`・`,男,,女`）は左端が空で中身が散るので、疎でも前書きにしない
/// - セルの数がちょうど列の数（空のセルで埋めた行・`PREAMBLE,,,,`）か、1 つ以下（区切りの無い表題・空行）か、
///   列の数より少なく、どのセルも改行と区切りの字を含まない（e-Stat の `平成27年,人口動態調査,,`）
///
/// 🔴 3 つめの条件は、汚れた見出しを前書きにしないため（迷子の引用符が後ろの引用符と組み違えると、見出しは
/// 少ない数のセルになり、飲み込んだ改行や区切りの字を含む）。見出しを捨てる誤りは、前書きを残す誤りより重い（Issue #25 の Pollock の測定）。
/// 幅が `MIN_PREAMBLE_COLUMNS` 未満なら 0。前書きは `MAX_PREAMBLE_ROWS` 行までで、表の行を少なくとも 1 行残す。
#[must_use]
pub fn preamble_rows(sample: &[u8], records: &[Record], dialect: &Dialect) -> usize {
    let columns = modal_columns(records);
    let width = used_width(sample, records, *dialect).min(columns);
    if width < MIN_PREAMBLE_COLUMNS {
        return 0;
    }
    let sparse = |record: &Record| {
        let filled: Vec<usize> = filled_positions(sample, record, dialect).collect();
        let cells = record.fields().len();
        let titled = match filled.as_slice() {
            [] | [_] => true,
            [first, ..] => *first == 0 && filled.len() <= PREAMBLE_FILLED_CELLS,
        };
        let shaped = cells == columns
            || cells <= 1
            || (cells < columns && !merges_cells(sample, record, *dialect));
        filled.len().saturating_mul(2) < width && titled && shaped
    };
    let limit = MAX_PREAMBLE_ROWS.min(records.len().saturating_sub(1));
    records
        .iter()
        .take(limit)
        .take_while(|record| sparse(record))
        .count()
}

/// 左端から始まる前書きの行の、中身のあるセルの数の上限（表題・単位・作成日）。
const PREAMBLE_FILLED_CELLS: usize = 3;

/// `record` の中身のある（空白だけでない・全角の空白も空白とみなす）セルの位置。
fn filled_positions<'a>(
    sample: &'a [u8],
    record: &'a Record,
    dialect: &'a Dialect,
) -> impl Iterator<Item = usize> + 'a {
    record
        .fields()
        .iter()
        .enumerate()
        .filter(|&(_, &field)| !dialect.units().is_blank(&value(sample, field, dialect)))
        .map(|(position, _)| position)
}

/// 使われている列の幅。どこかの行で中身のある、いちばん右のセルの位置 ＋ 1。
/// 🔴 右に空の列が続く表（10 列のうち 3 列だけ使う）で、見出しを「疎な行」とみなさないため。
fn used_width(sample: &[u8], records: &[Record], dialect: Dialect) -> usize {
    records
        .iter()
        .filter_map(|record| filled_positions(sample, record, &dialect).last())
        .max()
        .map_or(0, |position| position.saturating_add(1))
}

/// `record` のどれかのセルの値が、改行か区切りの字を含むか（引用符の組み違いで、行や隣のセルを飲み込んだ跡）。
/// 🔴 Pollock の `row_extra_quote0_col0`: 見出しの先頭の迷子の引用符が 7 つのセルを 1 つに飲み込み、1 行のまま
/// 3 つのセルになる。区切りの字を含むセルで見分けないと、短い表題の行とみなして見出しを捨てる。
fn merges_cells(sample: &[u8], record: &Record, dialect: Dialect) -> bool {
    let delimiter = dialect.delimiter().byte();
    record.fields().iter().any(|&field| {
        judged(sample, field, dialect)
            .iter()
            .any(|&byte| byte == b'\n' || byte == b'\r' || byte == delimiter)
    })
}

/// 見出しの有無を推し量る。数や日付の列の先頭が文字列なら見出しあり、先頭も同じ形なら見出しなし。
#[must_use]
pub fn header_guess(sample: &[u8], records: &[Record], dialect: &Dialect) -> HeaderGuess {
    match header_votes(sample, records, *dialect) {
        (present, absent) if present > absent => HeaderGuess::Present,
        (present, absent) if absent > present => HeaderGuess::Absent,
        (_, _) => HeaderGuess::Unknown,
    }
}
