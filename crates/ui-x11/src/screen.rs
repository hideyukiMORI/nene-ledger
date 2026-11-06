//! 画面のスレッドが持つ今の状態。

use nene_ledger_application::clipboard_text::ClipboardText;
use nene_ledger_application::window_view::WindowView;
use nene_ledger_application::zoom_level::ZoomLevel;
use nene_ledger_core::glyph_metrics::GlyphMetrics;

use crate::cursor_shape::CursorShape;
use crate::incoming_paste::IncomingPaste;
use crate::pointer::Pointer;
use crate::viewport::Viewport;

/// 画面のスレッドが持つ今の状態。窓の大きさと置かれ方・マウス・今のカーソル・クリップボードに置いた中身・中身の字の拡大と寸法。
#[derive(Debug)]
pub(crate) struct Screen {
    viewport: Viewport,
    pointer: Pointer,
    cursor: CursorShape,
    clipboard: Option<ClipboardText>,
    zoom: ZoomLevel,
    content: GlyphMetrics,
    holding: bool,
    /// 分けて受け取っている途中の貼り付け（`INCR`）。
    incoming: Option<IncomingPaste>,
    /// 今の貼り付けの求めに使う属性の番（0 か 1・GitHub Issue #144）。
    paste_slot: usize,
    /// OS に付けてある窓の名前（変わったときだけ付け直す・GitHub Issue #167）。
    name: String,
}

impl Screen {
    /// 窓を開いたときの状態。マウスは何も押しておらず、カーソルは矢印。
    pub(crate) fn new(viewport: Viewport) -> Self {
        Self {
            viewport,
            pointer: Pointer::new(),
            cursor: CursorShape::Arrow,
            clipboard: None,
            zoom: ZoomLevel::hundred(),
            content: viewport.metrics(),
            holding: false,
            incoming: None,
            paste_slot: 0,
            name: String::new(),
        }
    }

    /// 窓の名前が `name` と違えば覚え直して `true`（OS に付け直す）。
    pub(crate) fn rename(&mut self, name: &str) -> bool {
        if self.name == name {
            return false;
        }
        name.clone_into(&mut self.name);
        true
    }

    /// 中身の字を測った拡大。
    pub(crate) fn zoom(&self) -> ZoomLevel {
        self.zoom
    }

    /// 拡大 `zoom` で測り直した中身の字の寸法 `content` に替える。
    pub(crate) fn set_zoom(&mut self, zoom: ZoomLevel, content: GlyphMetrics) {
        self.zoom = zoom;
        self.content = content;
    }

    /// 今の貼り付けの求めに使う属性の番。
    pub(crate) fn paste_slot(&self) -> usize {
        self.paste_slot
    }

    /// 新しく求めるので、属性の番を替えて返す。
    pub(crate) fn next_paste_slot(&mut self) -> usize {
        self.paste_slot = (self.paste_slot + 1) % 2;
        self.paste_slot
    }

    /// 分けて受け取っている途中の貼り付けの置き場。
    pub(crate) fn incoming_paste(&mut self) -> &mut Option<IncomingPaste> {
        &mut self.incoming
    }

    /// クリップボードに置いている中身（持ち主のときだけ）。
    pub(crate) fn clipboard(&self) -> Option<&ClipboardText> {
        self.clipboard.as_ref()
    }

    /// クリップボードの中身を替える（持ち主になった・手放した）。
    pub(crate) fn set_clipboard(&mut self, clipboard: Option<ClipboardText>) {
        self.clipboard = clipboard;
    }

    pub(crate) fn viewport(&self) -> Viewport {
        self.viewport
    }

    /// 窓の大きさか置かれ方が変わった。
    pub(crate) fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;
    }

    pub(crate) fn pointer_mut(&mut self) -> &mut Pointer {
        &mut self.pointer
    }

    /// 今のカーソル。
    pub(crate) fn cursor(&self) -> CursorShape {
        self.cursor
    }

    /// カーソルを替えた。
    pub(crate) fn set_cursor(&mut self, cursor: CursorShape) {
        self.cursor = cursor;
    }

    /// 描くときと意図を渡すときの窓の見え。
    /// 中身の上で左のボタンを押していて、まだ離していないか（ドラッグを中身へ渡す）。
    pub(crate) fn holding(&self) -> bool {
        self.holding
    }

    pub(crate) fn set_holding(&mut self, holding: bool) {
        self.holding = holding;
    }

    pub(crate) fn view(&self) -> WindowView {
        let viewport = self.viewport;
        WindowView::new(
            viewport.size(),
            viewport.metrics(),
            viewport.placement(),
            self.pointer.hovered(),
        )
        .with_content_metrics(self.content)
    }
}
