//! ゲート発火の証明 1 件の入力。

/// 違反 1 件の入力。書き込むファイルと、期待する診断の断片。
#[derive(Debug, Clone)]
pub(crate) struct ProofCase {
    id: &'static str,
    rule: &'static str,
    files: Vec<(String, String)>,
    diagnostic: &'static str,
}

impl ProofCase {
    /// 証明の番号・守る規則・書き込むファイル（相対パスと本文）・期待する診断。
    pub(crate) fn new(
        id: &'static str,
        rule: &'static str,
        files: Vec<(String, String)>,
        diagnostic: &'static str,
    ) -> Self {
        Self {
            id,
            rule,
            files,
            diagnostic,
        }
    }

    /// 証明の番号（`P-01` 等）。
    pub(crate) fn id(&self) -> &'static str {
        self.id
    }

    /// 守る規則。
    pub(crate) fn rule(&self) -> &'static str {
        self.rule
    }

    /// 書き込むファイル。
    pub(crate) fn files(&self) -> &[(String, String)] {
        &self.files
    }

    /// 期待する診断の断片。
    pub(crate) fn diagnostic(&self) -> &'static str {
        self.diagnostic
    }
}
