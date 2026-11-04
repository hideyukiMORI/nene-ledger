//! 引用の流儀。

/// セルの引用とエスケープの流儀。閉じた集合（RS-002）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Quoting {
    /// RFC 4180。引用符で囲めば区切りと改行を含められ、中の引用符は 2 つ重ねる（`""`）。
    Doubled,
    /// 引用符で囲み、中ではバックスラッシュで逃がす（`\"`）。
    QuotedBackslash,
    /// 引用符を使わず、バックスラッシュで逃がす（`\t` `\n` `\\`。PostgreSQL の COPY TEXT の TSV）。
    Backslash,
    /// 引用もエスケープもしない（区切りと改行はセルに入れられない。IANA の TSV）。
    Plain,
}
