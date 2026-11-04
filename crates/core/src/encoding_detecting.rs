//! 文字コードの判別の純関数（`EncodingGuess` が使う）。バイト列の構造を見るだけで、表は引かない。

use crate::text_encoding::TextEncoding;

/// BOM で決まる文字コード。
pub(crate) fn bom_encoding(input: &[u8]) -> Option<TextEncoding> {
    [
        TextEncoding::Utf8WithBom,
        TextEncoding::Utf16LeWithBom,
        TextEncoding::Utf16BeWithBom,
    ]
    .into_iter()
    .find(|encoding| input.starts_with(encoding.bom()))
}

/// BOM の無い UTF-16 らしさ。NUL が奇数の位置（LE の ASCII）か偶数の位置（BE の ASCII）に偏るか。
pub(crate) fn utf16_without_bom(sample: &[u8]) -> Option<TextEncoding> {
    let (pairs, _) = sample.as_chunks::<2>();
    let units = u32::try_from(pairs.len()).unwrap_or(u32::MAX);
    let (mut even, mut odd) = (0_u32, 0_u32);
    for &[first, second] in pairs {
        even = even.saturating_add(u32::from(first == 0));
        odd = odd.saturating_add(u32::from(second == 0));
    }
    // 4 分の 1 以上の単位で片側だけが NUL なら UTF-16 とみなす（ASCII の多い CSV では半分を超える）。
    let leaning =
        |many: u32, few: u32| many.saturating_mul(4) >= units && few.saturating_mul(10) < many;
    if units < 2 {
        None
    } else if leaning(odd, even) {
        Some(TextEncoding::Utf16Le)
    } else if leaning(even, odd) {
        Some(TextEncoding::Utf16Be)
    } else {
        None
    }
}

/// UTF-8 として正しいか。標本が途中で切れているなら、最後の欠けた文字は誤りとしない。
pub(crate) fn utf8_valid(sample: &[u8], complete: bool) -> bool {
    match core::str::from_utf8(sample) {
        Ok(_) => true,
        Err(error) => !complete && error.error_len().is_none(),
    }
}

/// 次のバイトが `accept` を満たすか。入力が尽きたら、標本が途中で切れたときだけ認める。
fn next_is<I>(bytes: &mut I, accept: fn(u8) -> bool, complete: bool) -> bool
where
    I: Iterator<Item = u8>,
{
    bytes.next().map_or(!complete, accept)
}

/// CP932 の 2 バイト目。
fn shift_jis_trail(byte: u8) -> bool {
    matches!(byte, 0x40..=0x7E | 0x80..=0xFC)
}

/// EUC-JP の 2 バイト目（と JIS X 0208 / 0212 の各バイト）。
fn euc_jp_trail(byte: u8) -> bool {
    matches!(byte, 0xA1..=0xFE)
}

/// EUC-JP の半角カナの 2 バイト目。
fn euc_jp_kana(byte: u8) -> bool {
    matches!(byte, 0xA1..=0xDF)
}

/// CP932（WHATWG の `Shift_JIS`）としてバイトの並びが正しいか（表に字があるかまでは見ない）。
pub(crate) fn shift_jis_valid(sample: &[u8], complete: bool) -> bool {
    let mut bytes = sample.iter().copied();
    while let Some(byte) = bytes.next() {
        let fine = match byte {
            0x00..=0x80 | 0xA1..=0xDF => true,
            0x81..=0x9F | 0xE0..=0xFC => next_is(&mut bytes, shift_jis_trail, complete),
            0xA0 | 0xFD..=0xFF => false,
        };
        if !fine {
            return false;
        }
    }
    true
}

/// EUC-JP としてバイトの並びが正しいか（表に字があるかまでは見ない）。
pub(crate) fn euc_jp_valid(sample: &[u8], complete: bool) -> bool {
    let mut bytes = sample.iter().copied();
    while let Some(byte) = bytes.next() {
        let fine = match byte {
            0x00..=0x7F => true,
            0x8E => next_is(&mut bytes, euc_jp_kana, complete),
            0x8F => {
                next_is(&mut bytes, euc_jp_trail, complete)
                    && next_is(&mut bytes, euc_jp_trail, complete)
            }
            0xA1..=0xFE => next_is(&mut bytes, euc_jp_trail, complete),
            0x80..=0x8D | 0x90..=0xA0 | 0xFF => false,
        };
        if !fine {
            return false;
        }
    }
    true
}

/// EUC-JP として読むと、ひらがな（`A4 A1`〜`A4 F3`）があるか。
///
/// CP932 のひらがな（先頭 `0x82`）は EUC-JP として誤りなので、両方として読める標本でひらがなが見えるのは EUC-JP の文章。
pub(crate) fn euc_jp_has_hiragana(sample: &[u8]) -> bool {
    let mut bytes = sample.iter().copied();
    while let Some(byte) = bytes.next() {
        match byte {
            0xA4 => {
                if bytes
                    .next()
                    .is_some_and(|trail| matches!(trail, 0xA1..=0xF3))
                {
                    return true;
                }
            }
            0x8E | 0xA1..=0xA3 | 0xA5..=0xFE => {
                bytes.next();
            }
            0x8F => {
                bytes.next();
                bytes.next();
            }
            0x00..=0x8D | 0x90..=0xA0 | 0xFF => {}
        }
    }
    false
}
