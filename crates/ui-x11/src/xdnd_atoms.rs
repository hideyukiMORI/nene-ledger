//! ファイルのドラッグ＆ドロップ（XDND の版 5）の原子（CLIP-04・GitHub Issue #243）。

use x11rb::protocol::xproto::{Atom, ConnectionExt};
use x11rb::rust_connection::RustConnection;

use crate::ui_failure::UiFailure;

/// XDND の版（freedesktop.org の XDND の取り決め）。
pub(crate) const XDND_VERSION: u32 = 5;

/// XDND の原子。
#[derive(Debug, Clone, Copy)]
pub(crate) struct XdndAtoms {
    aware: Atom,
    enter: Atom,
    position: Atom,
    status: Atom,
    leave: Atom,
    drop: Atom,
    finished: Atom,
    selection: Atom,
    action_copy: Atom,
    uri_list: Atom,
    property: Atom,
}

impl XdndAtoms {
    /// サーバから原子を受け取る。
    pub(crate) fn intern(connection: &RustConnection) -> Result<Self, UiFailure> {
        let atom = |name: &[u8]| -> Result<Atom, UiFailure> {
            Ok(connection.intern_atom(false, name)?.reply()?.atom)
        };
        Ok(Self {
            aware: atom(b"XdndAware")?,
            enter: atom(b"XdndEnter")?,
            position: atom(b"XdndPosition")?,
            status: atom(b"XdndStatus")?,
            leave: atom(b"XdndLeave")?,
            drop: atom(b"XdndDrop")?,
            finished: atom(b"XdndFinished")?,
            selection: atom(b"XdndSelection")?,
            action_copy: atom(b"XdndActionCopy")?,
            uri_list: atom(b"text/uri-list")?,
            property: atom(b"_NENE_LEDGER_DROP")?,
        })
    }

    /// 受け取れる印（窓の属性・値は版）。
    pub(crate) fn aware(self) -> Atom {
        self.aware
    }

    /// 送り手が窓に入った。
    pub(crate) fn enter(self) -> Atom {
        self.enter
    }

    /// 送り手が窓の上で動いた（受け取るかを `XdndStatus` で答える）。
    pub(crate) fn position(self) -> Atom {
        self.position
    }

    /// 受け取るかの答え。
    pub(crate) fn status(self) -> Atom {
        self.status
    }

    /// 送り手が窓を出た。
    pub(crate) fn leave(self) -> Atom {
        self.leave
    }

    /// 落とされた。
    pub(crate) fn drop(self) -> Atom {
        self.drop
    }

    /// 受け取り終えた（送り手へ）。
    pub(crate) fn finished(self) -> Atom {
        self.finished
    }

    /// 落とされた中身の選択（`XdndSelection`）。
    pub(crate) fn selection(self) -> Atom {
        self.selection
    }

    /// 写す（ファイルは開くだけで、送り手の元を消さない）。
    pub(crate) fn action_copy(self) -> Atom {
        self.action_copy
    }

    /// ファイルの URI の一覧の形（`text/uri-list`）。
    pub(crate) fn uri_list(self) -> Atom {
        self.uri_list
    }

    /// 落とされた中身を書いてもらう自分の窓の属性。
    pub(crate) fn property(self) -> Atom {
        self.property
    }
}
