//! 窓が使う原子。

use x11rb::protocol::xproto::{Atom, ConnectionExt};
use x11rb::rust_connection::RustConnection;

use crate::ui_failure::UiFailure;

/// 窓が使う原子（題名・閉じる合図・起こす合図・ウィンドウマネージャーへの頼み）。
#[derive(Debug, Clone, Copy)]
pub(crate) struct WindowAtoms {
    protocols: Atom,
    delete_window: Atom,
    net_wm_name: Atom,
    utf8_string: Atom,
    wake: Atom,
    motif_wm_hints: Atom,
    net_wm_moveresize: Atom,
    net_wm_state: Atom,
    net_wm_state_maximized_vert: Atom,
    net_wm_state_maximized_horz: Atom,
    wm_change_state: Atom,
    clipboard: Atom,
    targets: Atom,
    text_plain_utf8: Atom,
    paste: [Atom; 2],
    incr: Atom,
}

impl WindowAtoms {
    /// サーバから原子を受け取る。
    pub(crate) fn intern(connection: &RustConnection) -> Result<Self, UiFailure> {
        let atom = |name: &[u8]| -> Result<Atom, UiFailure> {
            Ok(connection.intern_atom(false, name)?.reply()?.atom)
        };
        Ok(Self {
            protocols: atom(b"WM_PROTOCOLS")?,
            delete_window: atom(b"WM_DELETE_WINDOW")?,
            net_wm_name: atom(b"_NET_WM_NAME")?,
            utf8_string: atom(b"UTF8_STRING")?,
            wake: atom(b"_NENE_LEDGER_WAKE")?,
            motif_wm_hints: atom(b"_MOTIF_WM_HINTS")?,
            net_wm_moveresize: atom(b"_NET_WM_MOVERESIZE")?,
            net_wm_state: atom(b"_NET_WM_STATE")?,
            net_wm_state_maximized_vert: atom(b"_NET_WM_STATE_MAXIMIZED_VERT")?,
            net_wm_state_maximized_horz: atom(b"_NET_WM_STATE_MAXIMIZED_HORZ")?,
            wm_change_state: atom(b"WM_CHANGE_STATE")?,
            clipboard: atom(b"CLIPBOARD")?,
            targets: atom(b"TARGETS")?,
            text_plain_utf8: atom(b"text/plain;charset=utf-8")?,
            paste: [
                atom(b"_NENE_LEDGER_PASTE_0")?,
                atom(b"_NENE_LEDGER_PASTE_1")?,
            ],
            incr: atom(b"INCR")?,
        })
    }

    /// `WM_PROTOCOLS`。
    pub(crate) fn protocols(self) -> Atom {
        self.protocols
    }

    /// 閉じる合図（`WM_DELETE_WINDOW`）。
    pub(crate) fn delete_window(self) -> Atom {
        self.delete_window
    }

    /// 題名（`_NET_WM_NAME`）。
    pub(crate) fn net_wm_name(self) -> Atom {
        self.net_wm_name
    }

    /// `UTF8_STRING`。
    pub(crate) fn utf8_string(self) -> Atom {
        self.utf8_string
    }

    /// 起こす合図（`_NENE_LEDGER_WAKE`）。
    pub(crate) fn wake(self) -> Atom {
        self.wake
    }

    /// 飾りの指定（`_MOTIF_WM_HINTS`）。
    pub(crate) fn motif_wm_hints(self) -> Atom {
        self.motif_wm_hints
    }

    /// 移動とリサイズの頼み（`_NET_WM_MOVERESIZE`）。
    pub(crate) fn net_wm_moveresize(self) -> Atom {
        self.net_wm_moveresize
    }

    /// 置かれ方（`_NET_WM_STATE`）。
    pub(crate) fn net_wm_state(self) -> Atom {
        self.net_wm_state
    }

    /// 縦の最大化（`_NET_WM_STATE_MAXIMIZED_VERT`）。
    pub(crate) fn maximized_vert(self) -> Atom {
        self.net_wm_state_maximized_vert
    }

    /// 横の最大化（`_NET_WM_STATE_MAXIMIZED_HORZ`）。
    pub(crate) fn maximized_horz(self) -> Atom {
        self.net_wm_state_maximized_horz
    }

    /// 最小化の頼み（`WM_CHANGE_STATE`）。
    pub(crate) fn wm_change_state(self) -> Atom {
        self.wm_change_state
    }

    /// クリップボード（`CLIPBOARD`）。
    pub(crate) fn clipboard(self) -> Atom {
        self.clipboard
    }

    /// 貼り付けで受け取る中身を書いてもらう、自分の窓の属性（`_NENE_LEDGER_PASTE_0` と `_1`）。求めるたびに交互に使い、
    /// 途中で止まった前の `INCR` の持ち主が書く分を、次の受け取りに混ぜない（GitHub Issue #144）。
    pub(crate) fn paste(self, slot: usize) -> Atom {
        self.paste.get(slot % 2).copied().unwrap_or(x11rb::NONE)
    }

    /// 大きな中身を分けて渡す印（`INCR`・ICCCM）。分けた受け取りはまだしない。
    pub(crate) fn incr(self) -> Atom {
        self.incr
    }

    /// 渡せる形の一覧（`TARGETS`）。
    pub(crate) fn targets(self) -> Atom {
        self.targets
    }

    /// `text/plain;charset=utf-8`。
    pub(crate) fn text_plain_utf8(self) -> Atom {
        self.text_plain_utf8
    }
}
