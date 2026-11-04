//! レコードの終わり方。

/// 1 つのレコードの終わり方。原本の改行をそのまま覚えて、保存で同じ改行を書き戻す（KEEP-02）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LineEnding {
    /// `\r\n`。
    CrLf,
    /// `\n`。
    Lf,
    /// `\r` だけ（古い Mac）。
    Cr,
    /// 改行が無いまま入力が終わった（最終行の改行なし）。
    EndOfInput,
}

impl LineEnding {
    /// 改行のバイト列。
    #[must_use]
    pub fn bytes(self) -> &'static [u8] {
        match self {
            Self::CrLf => b"\r\n",
            Self::Lf => b"\n",
            Self::Cr => b"\r",
            Self::EndOfInput => b"",
        }
    }

    /// 改行のバイト数。
    #[must_use]
    pub fn len(self) -> usize {
        self.bytes().len()
    }

    /// 改行が無い（入力の終わり）か。
    #[must_use]
    pub fn is_empty(self) -> bool {
        self == Self::EndOfInput
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lengths_match_bytes() {
        for ending in [
            LineEnding::CrLf,
            LineEnding::Lf,
            LineEnding::Cr,
            LineEnding::EndOfInput,
        ] {
            assert_eq!(ending.len(), ending.bytes().len());
        }
        assert!(LineEnding::EndOfInput.is_empty());
        assert!(!LineEnding::Cr.is_empty());
    }
}
