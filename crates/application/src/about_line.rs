//! 「このアプリについて」の行（ADR 0032 の決定 6・`DEVELOPMENT_WORKFLOW` 第 10 節）。

use alloc::format;
use alloc::string::String;

/// 製品の版。`Cargo.toml` の `[workspace.package] version` から導く（版の入力はそこだけ・`DEVELOPMENT_WORKFLOW` 第 10 節）。
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 「このアプリについて」の画面に、読むだけで出す行。閉じた集合で、この順に出す。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AboutLine {
    /// 名前と版。
    Version,
    /// 著作権。
    Copyright,
    /// ライセンスの要旨（ADR 0009）。
    License,
    /// 保証が無いこと。
    Warranty,
    /// ライセンスの全文と、外部クレートの著作権表示のありか。
    Notices,
    /// ソースの置き場。
    Repository,
}

impl AboutLine {
    /// 出す順の全部。
    pub const ALL: [Self; 6] = [
        Self::Version,
        Self::Copyright,
        Self::License,
        Self::Warranty,
        Self::Notices,
        Self::Repository,
    ];

    /// 行の文。
    #[must_use]
    pub fn text(self) -> String {
        match self {
            Self::Version => format!("NeNe Ledger {VERSION}"),
            Self::Copyright => String::from("Copyright (c) 2026 Hideyuki Mori"),
            Self::License => {
                String::from("使うのは自由・改変はクレジット必須・配布は無償・販売は禁止")
            }
            Self::Warranty => {
                String::from("無保証（NeNe Ledger License 1.0・LicenseRef-NeNe-Ledger-1.0）")
            }
            Self::Notices => {
                String::from("全文は LICENSE、部品の著作権表示は THIRD-PARTY-NOTICES.txt")
            }
            Self::Repository => String::from("https://github.com/hideyukiMORI/nene-ledger"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 版は `Cargo.toml` の版そのもの（第 2 の版の入力を作らない）。
    #[test]
    fn the_version_line_carries_the_package_version() {
        assert!(!VERSION.is_empty());
        assert_eq!(
            AboutLine::Version.text(),
            format!("NeNe Ledger {}", env!("CARGO_PKG_VERSION"))
        );
    }
}
