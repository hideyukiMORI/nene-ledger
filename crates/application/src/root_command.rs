//! 合成ルートだけが受ける意図（窓・ファイル・裏の探しを起こす・表の状態は変えない・GitHub Issue #377）。

/// 合成ルートだけが受ける意図。表（`Document`）とパレットは受けずに通す。1 つにまとめて、意図の `match` の列を短く保つ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootCommand {
    /// 新しい表を新しい窓で開く（Ctrl+N・ADR 0028 の決定 1）。
    NewTable,
    /// ファイルを選んで新しい窓で開く（Ctrl+O・GitHub Issue #344）。
    OpenFile,
    /// 列の数が見出しと違う次の（`forward` でなければ前の）行へ（F8・Shift+F8・GitHub Issue #338・#377）。
    FindRagged {
        /// 次へ探すか。
        forward: bool,
    },
}
