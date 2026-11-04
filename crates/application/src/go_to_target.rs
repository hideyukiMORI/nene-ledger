//! 飛ぶ先の行と列。

use alloc::format;
use alloc::string::String;

use crate::status_line::group_digits;

/// 飛ぶ先。ファイルの中のレコードの番号（0 始まり・見出しの行を含む）と、元の列の番号（0 始まり・無ければ今の列）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GoToTarget {
    record: u64,
    column: Option<usize>,
}

impl GoToTarget {
    /// 打った文字列 `text` を読む。`1200` は 1,200 行目、`1200,3` は 1,200 行目の 3 列目（どちらも 1 始まり・前後の空白は無視）。
    /// 読めなければ `None`。
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let (row, column) = match text.trim().split_once(',') {
            Some((row, column)) => (row, Some(column)),
            None => (text.trim(), None),
        };
        let number = |part: &str| -> Option<u64> {
            let part = part.trim();
            if part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()) {
                return None;
            }
            part.parse::<u64>().ok()?.checked_sub(1)
        };
        let record = number(row)?;
        let column = match column {
            Some(part) => Some(usize::try_from(number(part)?).ok()?),
            None => None,
        };
        Some(Self { record, column })
    }

    /// レコードの番号（0 始まり）。
    #[must_use]
    pub fn record(self) -> u64 {
        self.record
    }

    /// 元の列の番号（0 始まり）。
    #[must_use]
    pub fn column(self) -> Option<usize> {
        self.column
    }

    /// 飛ぶ先の文言（「1,200 行目 · 3 列目」）。
    #[must_use]
    pub fn describe(self) -> String {
        let row = format!("{} 行目", group_digits(self.record.saturating_add(1)));
        match self.column {
            Some(column) => format!("{row} · {} 列目", column.saturating_add(1)),
            None => row,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_and_columns_are_read_from_one_based_numbers() {
        let target = GoToTarget::parse(" 1200 ").unwrap();
        assert_eq!((target.record(), target.column()), (1_199, None));
        assert_eq!(target.describe(), "1,200 行目");
        let cell = GoToTarget::parse("1200, 3").unwrap();
        assert_eq!((cell.record(), cell.column()), (1_199, Some(2)));
        assert_eq!(cell.describe(), "1,200 行目 · 3 列目");
        for bad in ["", "0", "abc", "12,", ",3", "1,0", "-5", "1.5", "1,2,3"] {
            assert_eq!(GoToTarget::parse(bad), None, "{bad}");
        }
    }
}
