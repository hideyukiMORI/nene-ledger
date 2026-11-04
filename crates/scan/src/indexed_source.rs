//! 範囲を読む材料（原本・索引・読み方）。

use nene_ledger_core::text_format::TextFormat;

use crate::source::Source;
use crate::sparse_index::SparseIndex;

/// 範囲を読む材料。原本と、読み始めの目印を引く索引（作り終える前のものでもよい）と、読み方。
#[derive(Debug)]
pub struct IndexedSource {
    source: Source,
    index: SparseIndex,
    format: TextFormat,
}

impl IndexedSource {
    /// `source` を `format` で、`index` の目印から読む。
    #[must_use]
    pub fn new(source: Source, index: SparseIndex, format: TextFormat) -> Self {
        Self {
            source,
            index,
            format,
        }
    }

    /// 原本。
    #[must_use]
    pub fn source(&self) -> &Source {
        &self.source
    }

    /// 索引。
    #[must_use]
    pub fn index(&self) -> &SparseIndex {
        &self.index
    }

    /// 読み方。
    #[must_use]
    pub fn format(&self) -> TextFormat {
        self.format
    }
}
