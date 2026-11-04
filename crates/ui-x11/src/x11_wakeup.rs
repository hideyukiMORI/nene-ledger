//! 作業のスレッドから X11 の窓を起こす。

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use nene_ledger_application::wakeup::Wakeup;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ClientMessageEvent, ConnectionExt, EventMask, Window};
use x11rb::rust_connection::RustConnection;

/// 作業のスレッドから窓へ `ClientMessage` を送って起こす（ADR 0014 の決定 3）。
///
/// 届く前の合図は 1 つにまとめる。窓が合図を受け取ったら `pending` を下ろし、次の合図を送れるようにする。
#[derive(Debug)]
pub(crate) struct X11Wakeup {
    connection: Arc<RustConnection>,
    window: Window,
    atom: u32,
    pending: Arc<AtomicBool>,
}

impl X11Wakeup {
    /// 窓を起こす口を作る。`atom` は合図の種類、`pending` は窓と共有する「届く前の合図がある」印。
    pub(crate) fn new(
        connection: Arc<RustConnection>,
        window: Window,
        atom: u32,
        pending: Arc<AtomicBool>,
    ) -> Self {
        Self {
            connection,
            window,
            atom,
            pending,
        }
    }
}

impl Wakeup for X11Wakeup {
    fn wake(&self) {
        if self.pending.swap(true, Ordering::AcqRel) {
            return;
        }
        let event = ClientMessageEvent::new(32, self.window, self.atom, [0_u32; 5]);
        let sent = self
            .connection
            .send_event(false, self.window, EventMask::NO_EVENT, event)
            .is_ok()
            && self.connection.flush().is_ok();
        if !sent {
            // 送れなかった（窓を閉じている途中など）。印を下ろし、次の合図で送り直せるようにする。
            self.pending.store(false, Ordering::Release);
        }
    }
}
