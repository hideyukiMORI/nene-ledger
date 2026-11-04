//! 1 枚の画面。

use alloc::vec::Vec;

use crate::draw_command::DrawCommand;
use crate::pixel_size::PixelSize;

/// 1 枚の画面。窓の大きさと、描く順の命令。UI は画面外の面に順に描いてから窓へ転送する（ADR 0002 の決定 3）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    size: PixelSize,
    commands: Vec<DrawCommand>,
}

impl Frame {
    /// 画面を作る。
    #[must_use]
    pub fn new(size: PixelSize, commands: Vec<DrawCommand>) -> Self {
        Self { size, commands }
    }

    /// 窓の大きさ。
    #[must_use]
    pub fn size(&self) -> PixelSize {
        self.size
    }

    /// 描く順の命令。
    #[must_use]
    pub fn commands(&self) -> &[DrawCommand] {
        &self.commands
    }

    /// 描く順の命令を取り出す（別のフレームに重ねるとき）。
    #[must_use]
    pub fn into_commands(self) -> Vec<DrawCommand> {
        self.commands
    }
}
