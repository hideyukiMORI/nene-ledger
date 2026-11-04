//! 開いているコマンドパレットと、その色。

use nene_ledger_application::command_palette::CommandPalette;
use nene_ledger_core::palette::Palette;

/// 開いているコマンドパレットの状態と、開いたときの配色（配色を試しても、パレットはこの色で描く）。
#[derive(Debug)]
pub(crate) struct OpenPalette {
    state: CommandPalette,
    colors: Palette,
}

impl OpenPalette {
    pub(crate) fn new(state: CommandPalette, colors: Palette) -> Self {
        Self { state, colors }
    }

    pub(crate) fn state(&self) -> &CommandPalette {
        &self.state
    }

    pub(crate) fn state_mut(&mut self) -> &mut CommandPalette {
        &mut self.state
    }

    pub(crate) fn colors(&self) -> Palette {
        self.colors
    }
}
