//! `text/uri-list`（RFC 2483）からファイルの道筋を取り出す（XDND・CLIP-04・GitHub Issue #243）。

/// `text/uri-list` の `text` の、`file:` の URI ごとのファイルの道筋（落とした順）。`#` で始まる行・空の行・`file:` でない
/// URI は飛ばす。`file://ホスト/道筋` のホストは読み捨て、`%xx` は 1 バイトに戻す（UTF-8 でないバイトは置き換える）。
pub(crate) fn file_paths(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(file_path)
        .collect()
}

/// 1 つの `file:` の URI の道筋。`file:` でないか、道筋が `/` で始まらなければ `None`。
fn file_path(uri: &str) -> Option<String> {
    let rest = uri.strip_prefix("file:")?;
    let path = match rest.strip_prefix("//") {
        // `file://ホスト/道筋`（ホストが空なら `file:///道筋`）。
        Some(authority) => &authority[authority.find('/')?..],
        None => rest,
    };
    path.starts_with('/').then(|| percent_decoded(path))
}

/// `%xx` を 1 バイトに戻した文字列。`%` の後が 16 進の 2 字でなければ、そのまま残す。
fn percent_decoded(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0_usize;
    while let Some(&byte) = bytes.get(at) {
        let escaped = (byte == b'%')
            .then(|| bytes.get(at.saturating_add(1)..at.saturating_add(3)))
            .flatten()
            .and_then(|pair| core::str::from_utf8(pair).ok())
            .and_then(|pair| u8::from_str_radix(pair, 16).ok());
        if let Some(decoded) = escaped {
            out.push(decoded);
            at = at.saturating_add(3);
        } else {
            out.push(byte);
            at = at.saturating_add(1);
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 落とした順の道筋。空の行・注釈・`file:` でない URI は飛ばし、`%xx` とホストを扱う。
    #[test]
    fn file_uris_become_paths() {
        let list = "# from a file manager\r\nfile:///srv/tables/%E6%9D%B1%E4%BA%AC.csv\r\n\r\nhttps://example.com/a.csv\r\nfile://localhost/tmp/a%20b.tsv\r\nfile:/tmp/plain.csv\r\nfile://host-only\r\nfile:relative.csv\r\n";
        assert_eq!(
            file_paths(list),
            ["/srv/tables/東京.csv", "/tmp/a b.tsv", "/tmp/plain.csv"]
        );
    }

    /// `%` の後が 16 進の 2 字でなければ、そのまま残す。UTF-8 でないバイトは置き換える。
    #[test]
    fn broken_escapes_are_kept_and_bad_bytes_replaced() {
        assert_eq!(file_paths("file:///a%2"), ["/a%2"]);
        assert_eq!(file_paths("file:///a%zz%41"), ["/a%zzA"]);
        assert_eq!(file_paths("file:///%FF.csv"), ["/\u{FFFD}.csv"]);
        assert!(file_paths("").is_empty());
    }
}
