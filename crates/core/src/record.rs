//! 1 つのレコード。

use alloc::vec::Vec;

use crate::byte_range::ByteRange;
use crate::field::Field;
use crate::line_ending::LineEnding;
use crate::row_shape::RowShape;

/// 1 つのレコード（行）。原本の区間・終わり方・セルの位置・壊れているか。
///
/// `raw` と `terminator` を順につなげると、原本のバイト列にそのまま戻る（KEEP-02 の土台）。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Record {
    raw: ByteRange,
    terminator: LineEnding,
    fields: Vec<Field>,
    shape: RowShape,
}

impl Record {
    /// レコードを作る。`raw` は改行を含まない区間。
    #[must_use]
    pub fn new(
        raw: ByteRange,
        terminator: LineEnding,
        fields: Vec<Field>,
        shape: RowShape,
    ) -> Self {
        Self {
            raw,
            terminator,
            fields,
            shape,
        }
    }

    /// 改行を含まない、原本の区間。
    #[must_use]
    pub fn raw(&self) -> ByteRange {
        self.raw
    }

    /// 終わり方（原本の改行）。
    #[must_use]
    pub fn terminator(&self) -> LineEnding {
        self.terminator
    }

    /// セルの位置。列の数は行ごとに違ってよい（OPEN-14）。
    #[must_use]
    pub fn fields(&self) -> &[Field] {
        &self.fields
    }

    /// 方言どおりに切れたか。
    #[must_use]
    pub fn shape(&self) -> RowShape {
        self.shape
    }
}
