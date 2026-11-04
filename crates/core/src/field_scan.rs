//! 1 つのセルを切った結果。

use crate::boundary::Boundary;
use crate::field::Field;
use crate::row_shape::RowShape;

/// 1 つのセルを切った結果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldScan {
    field: Field,
    stop: usize,
    boundary: Boundary,
    shape: RowShape,
    unit_len: usize,
}

impl FieldScan {
    /// 切った結果を作る。`stop` は区切り・改行の始まり（セルの終わり）の位置。
    #[must_use]
    pub fn new(field: Field, stop: usize, boundary: Boundary, shape: RowShape) -> Self {
        Self {
            field,
            stop,
            boundary,
            shape,
            unit_len: 1,
        }
    }

    /// ASCII の 1 文字のバイト数を `unit_len` にした結果（UTF-16 は 2・ADR 0019）。区切りと改行を越える量に掛ける。
    #[must_use]
    pub fn in_units(self, unit_len: usize) -> Self {
        Self { unit_len, ..self }
    }

    /// 切ったセル。
    #[must_use]
    pub fn field(self) -> Field {
        self.field
    }

    /// 区切り・改行の始まりの位置。
    #[must_use]
    pub fn stop(self) -> usize {
        self.stop
    }

    /// セルの後ろに来たもの。
    #[must_use]
    pub fn boundary(self) -> Boundary {
        self.boundary
    }

    /// このセルが方言どおりに切れたか（閉じない引用符・文字として読んだ引用符）。
    #[must_use]
    pub fn shape(self) -> RowShape {
        self.shape
    }

    /// 次に読む位置（区切りと改行を越えた所）。区切りは 1 文字、改行は `LineEnding::len` 文字で、1 文字は `unit_len` バイト。
    #[must_use]
    pub fn next(self) -> usize {
        let characters = match self.boundary {
            Boundary::Delimiter => 1_usize,
            Boundary::Terminator(ending) => ending.len(),
            Boundary::End => 0_usize,
        };
        self.stop
            .saturating_add(characters.saturating_mul(self.unit_len))
    }
}
