//! 規則 ID（`RS-001` / `ARC-002` / `QLT-003` / `CNF-004` / `GIT-005`）の字句。

/// 規則 ID の接頭辞。文書と検査器が同じ集合を見る。
const PREFIXES: [&str; 5] = ["RS", "ARC", "QLT", "CNF", "GIT"];

/// 規則 ID の形をしているか調べる。
pub(crate) fn is_rule_id(token: &str) -> bool {
    let Some((prefix, number)) = token.split_once('-') else {
        return false;
    };
    PREFIXES.contains(&prefix) && number.len() == 3 && number.chars().all(|c| c.is_ascii_digit())
}

/// 1 行に現れる規則 ID を、出現順に全て取り出す。
///
/// 区切りは英数字以外。`ARC-0021` のように桁が多いものや `XARC-001` のように
/// 前に文字が付くものは ID として数えない。
pub(crate) fn rule_ids_in(line: &str) -> Vec<String> {
    line.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
        .filter(|token| is_rule_id(token))
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_each_prefix() {
        for id in ["RS-001", "ARC-012", "QLT-013", "CNF-009", "GIT-004"] {
            assert!(is_rule_id(id), "{id}");
        }
    }

    #[test]
    fn rejects_wrong_shapes() {
        for id in ["RS-1", "RS-0001", "CPP-001", "RS001", "rs-001", "ARC-00a"] {
            assert!(!is_rule_id(id), "{id}");
        }
    }

    #[test]
    fn extracts_ids_from_prose() {
        let found = rule_ids_in("依存方向（ARC-002）と `RS-005`、XARC-001 は違う。");
        assert_eq!(found, vec!["ARC-002".to_owned(), "RS-005".to_owned()]);
    }
}
