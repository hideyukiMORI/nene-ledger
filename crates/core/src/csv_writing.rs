//! RFC 4180 の CSV の 1 レコードを書く規則。

use alloc::string::String;

/// RFC 4180 の 1 レコード（カンマ区切り・`\r\n` で終わる）。区切り・引用符・CR・LF を含むセルだけを `"` で囲み、
/// 中の `"` は `""` にする（Python の `csv` の既定と同じ最小限の引用）。
///
/// 空のセル 1 つだけのレコードは、空の行にする（読み手が「セルの無い行」と同じに読む約束に合わせる）。
/// 🔴 製品の保存（原本のバイト列を写す・ADR 0005）とは別の、値を書き出す規則。
#[must_use]
pub fn rfc4180_record(cells: &[String]) -> String {
    let mut out = String::new();
    for (index, cell) in cells.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        if cell.contains([',', '"', '\r', '\n']) {
            out.push('"');
            out.push_str(&cell.replace('"', "\"\""));
            out.push('"');
        } else {
            out.push_str(cell);
        }
    }
    out.push_str("\r\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    fn record(cells: &[&str]) -> String {
        let cells: Vec<String> = cells.iter().map(|&cell| String::from(cell)).collect();
        rfc4180_record(&cells)
    }

    #[test]
    fn only_cells_that_need_it_are_quoted() {
        assert_eq!(record(&["a", "b c", " d "]), "a,b c, d \r\n");
        assert_eq!(
            record(&["1,2", "say \"hi\"", "x\ny", "x\ry"]),
            "\"1,2\",\"say \"\"hi\"\"\",\"x\ny\",\"x\ry\"\r\n"
        );
        assert_eq!(record(&["", "", ""]), ",,\r\n");
        assert_eq!(record(&[""]), "\r\n");
        assert_eq!(rfc4180_record(&[]), "\r\n");
    }
}
