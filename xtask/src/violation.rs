//! 規約検査が見つけた違反 1 件。

use std::fmt;

/// 検査が見つけた違反 1 件。
///
/// 🔴 フィールドは非公開である（RS-008）。生成経路を `new` に限ることで、
/// 規則 ID の無い違反を作れないようにしている。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Violation {
    rule: &'static str,
    path: String,
    line: usize,
    message: String,
}

impl Violation {
    /// 違反を 1 件作る。`line` は 1 始まり。0 は「ファイル全体」を指す。
    pub(crate) fn new(rule: &'static str, path: &str, line: usize, message: String) -> Self {
        Self {
            rule,
            path: path.to_owned(),
            line,
            message,
        }
    }

    /// 規則 ID。テストと集計が使う。
    pub(crate) fn rule(&self) -> &'static str {
        self.rule
    }
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line == 0 {
            write!(f, "{}: {} — {}", self.rule, self.path, self.message)
        } else {
            write!(
                f,
                "{}: {}:{} — {}",
                self.rule, self.path, self.line, self.message
            )
        }
    }
}
