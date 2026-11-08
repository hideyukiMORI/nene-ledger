//! スクロールする本体の何番目かと、ファイルの中のレコードの番号の写像。
//!
//! 見出しの行（続いた 1〜3 行・ADR 0034）は窓の上に固定し、本体から除く（VIEW-01）。見出しの前の前書きの行は本体に
//! そのまま残す（隠さない）。`header` は見出しの行の範囲で、見出しが無ければ `None`。

use nene_ledger_core::header_rows::HeaderRows;

/// 本体の `body` 番目の行のレコードの番号。見出しより後ろは見出しの行の数だけずれる。
#[must_use]
pub fn record_of(body: u64, header: Option<HeaderRows>) -> u64 {
    match header {
        Some(header) if body >= header.first() => body.saturating_add(header.count()),
        Some(_) | None => body,
    }
}

/// レコード `record` が本体の何番目に当たるか。見出しの行は、そのすぐ後ろの行の番目にする。
#[must_use]
pub fn body_of(record: u64, header: Option<HeaderRows>) -> u64 {
    match header {
        Some(header) if record >= header.end() => record.saturating_sub(header.count()),
        Some(header) if record >= header.first() => header.first(),
        Some(_) | None => record,
    }
}

/// レコードの数 `records` のファイルの、本体の行の数（ファイルにある見出しの行だけを除く）。
#[must_use]
pub fn body_count(records: u64, header: Option<HeaderRows>) -> u64 {
    match header {
        Some(header) => {
            let present = records.min(header.end()).saturating_sub(header.first());
            records.saturating_sub(present)
        }
        None => records,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    fn rows(first: u64, count: u64) -> Option<HeaderRows> {
        HeaderRows::new(first, count)
    }

    #[test]
    fn without_a_header_the_body_is_the_records() {
        assert_eq!(record_of(0, None), 0);
        assert_eq!(record_of(41, None), 41);
        assert_eq!(body_of(41, None), 41);
        assert_eq!(body_count(100, None), 100);
    }

    #[test]
    fn a_header_at_the_top_is_left_out_of_the_body() {
        let header = rows(0, 1);
        assert_eq!(record_of(0, header), 1);
        assert_eq!(record_of(9, header), 10);
        assert_eq!(body_of(1, header), 0);
        assert_eq!(body_of(0, header), 0);
        assert_eq!(body_count(100, header), 99);
    }

    #[test]
    fn preamble_rows_before_the_header_stay_in_the_body() {
        // 前書き 2 行（レコード 0・1）、見出しはレコード 2。
        let header = rows(2, 1);
        assert_eq!(record_of(0, header), 0);
        assert_eq!(record_of(1, header), 1);
        assert_eq!(record_of(2, header), 3);
        assert_eq!(body_of(3, header), 2);
        assert_eq!(body_of(2, header), 2);
        assert_eq!(body_of(1, header), 1);
        assert_eq!(body_count(10, header), 9);
        // 見出しのレコードが無い（短いファイル）なら、数は減らない。
        assert_eq!(body_count(2, header), 2);
    }

    #[test]
    fn a_header_of_several_rows_is_left_out_as_a_whole() {
        // 前書き 1 行（レコード 0）、見出しはレコード 1〜3。
        let header = rows(1, 3);
        let records: Vec<u64> = (0..3).map(|body| record_of(body, header)).collect();
        assert_eq!(records, [0, 4, 5]);
        let bodies: Vec<u64> = (0..6).map(|record| body_of(record, header)).collect();
        assert_eq!(bodies, [0, 1, 1, 1, 1, 2]);
        // 見出しの途中で終わるファイルは、ある見出しの行だけを除く。
        let counts: Vec<u64> = [10, 3, 1].map(|total| body_count(total, header)).to_vec();
        assert_eq!(counts, [7, 1, 1]);
    }

    #[test]
    fn the_mapping_goes_back_and_forth() {
        for header in [None, rows(0, 1), rows(1, 3), rows(4, 2)] {
            for body in 0..20 {
                assert_eq!(body_of(record_of(body, header), header), body);
            }
        }
    }
}
