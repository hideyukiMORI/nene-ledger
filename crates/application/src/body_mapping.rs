//! スクロールする本体の何番目かと、ファイルの中のレコードの番号の写像。
//!
//! 見出しの行は窓の上に固定し、本体から除く（VIEW-01）。見出しの前の前書きの行は本体にそのまま残す（隠さない）。
//! `header` は見出しのレコードの番号で、見出しが無ければ `None`。

/// 本体の `body` 番目の行のレコードの番号。見出しより後ろは 1 つずれる。
#[must_use]
pub fn record_of(body: u64, header: Option<u64>) -> u64 {
    match header {
        Some(header) if body >= header => body.saturating_add(1),
        Some(_) | None => body,
    }
}

/// レコード `record` が本体の何番目に当たるか。見出しそのものは、そのすぐ後ろの行の番目にする。
#[must_use]
pub fn body_of(record: u64, header: Option<u64>) -> u64 {
    match header {
        Some(header) if record > header => record.saturating_sub(1),
        Some(_) | None => record,
    }
}

/// レコードの数 `records` のファイルの、本体の行の数。
#[must_use]
pub fn body_count(records: u64, header: Option<u64>) -> u64 {
    match header {
        Some(header) if header < records => records.saturating_sub(1),
        Some(_) | None => records,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn without_a_header_the_body_is_the_records() {
        assert_eq!(record_of(0, None), 0);
        assert_eq!(record_of(41, None), 41);
        assert_eq!(body_of(41, None), 41);
        assert_eq!(body_count(100, None), 100);
    }

    #[test]
    fn a_header_at_the_top_is_left_out_of_the_body() {
        let header = Some(0);
        assert_eq!(record_of(0, header), 1);
        assert_eq!(record_of(9, header), 10);
        assert_eq!(body_of(1, header), 0);
        assert_eq!(body_of(0, header), 0);
        assert_eq!(body_count(100, header), 99);
    }

    #[test]
    fn preamble_rows_before_the_header_stay_in_the_body() {
        // 前書き 2 行（レコード 0・1）、見出しはレコード 2。
        let header = Some(2);
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
}
