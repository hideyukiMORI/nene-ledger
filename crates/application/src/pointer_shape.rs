//! 中身の上のマウスのカーソルの形。

/// 中身（表）の上で UI が出すマウスのカーソルの形（Issue #64）。窓の縁のカーソルは UI が飾りの当たり判定から決める。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PointerShape {
    /// 矢印。
    Arrow,
    /// 列の幅を変える（左右の矢印）。
    ColumnResize,
    /// 詳細ペインの高さを変える（上下の矢印・GitHub Issue #452）。
    RowResize,
}
