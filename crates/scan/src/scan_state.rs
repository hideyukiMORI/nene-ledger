//! 切り出しの途中の状態。

/// 1 バイトずつ進む切り出しの、途中の状態（`RecordStarts` が持つ）。
///
/// 参照実装（core の `scanning`）の先読み（`\r\n` の `\n`・`""` の 2 つ目）は、「次のバイトで決める」状態にしてある。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScanState {
    /// 次のバイトでレコードが始まる（入力の先頭・`\n` の後ろ）。
    RecordStart,
    /// 次のバイトでセルが始まる（区切りの後ろ）。
    FieldStart,
    /// 引用の外のセルの中。
    Unquoted,
    /// 引用の外で、バックスラッシュの次のバイトを待っている。
    UnquotedEscape,
    /// 引用の中。
    Quoted,
    /// 引用の中で、バックスラッシュの次のバイトを待っている。
    QuotedEscape,
    /// 引用の中で引用符を見た。次が引用符なら `""`、そうでなければ閉じた。
    QuoteInQuoted,
    /// 引用符が閉じた後ろ（区切りか改行までは同じセル）。
    AfterClose,
    /// `\r` でレコードが終わった。次が `\n` なら `\r\n`。
    AfterCarriageReturn,
}

impl ScanState {
    /// 引用符で始まったセルの中か（`""` の 2 つ目を待つ間も含む）。引用符が開いたことを見分けるのに使う。
    pub(crate) fn inside_quotes(self) -> bool {
        match self {
            Self::Quoted | Self::QuotedEscape | Self::QuoteInQuoted => true,
            Self::RecordStart
            | Self::FieldStart
            | Self::Unquoted
            | Self::UnquotedEscape
            | Self::AfterClose
            | Self::AfterCarriageReturn => false,
        }
    }

    /// ここで入力が終わると、引用符が閉じていないか。`""` の 2 つ目を待つ間は、閉じたとみなす（参照実装と同じ）。
    pub(crate) fn unclosed(self) -> bool {
        match self {
            Self::Quoted | Self::QuotedEscape => true,
            Self::RecordStart
            | Self::FieldStart
            | Self::Unquoted
            | Self::UnquotedEscape
            | Self::QuoteInQuoted
            | Self::AfterClose
            | Self::AfterCarriageReturn => false,
        }
    }
}
