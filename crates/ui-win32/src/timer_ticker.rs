//! 窓のタイマー（`SetTimer`）で刻む時計。

use core::cell::Cell;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use nene_ledger_application::ticker::Ticker;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{KillTimer, SetTimer};

/// 時計のタイマーの番号。
pub(crate) const TICK_TIMER: usize = 1;
/// 刻みの間隔（ミリ秒・X11 版と同じ 50ms）。
const TICK_MILLISECONDS: u32 = 50;

/// 窓のタイマーで刻む時計（`Ticker` の実装・Issue #55）。刻みは窓の手続きが `WM_TIMER` で数え、ここで受け取る。
/// 画面のスレッドだけが使う（タイマーは窓と同じスレッドに届く）。
#[derive(Debug)]
pub(crate) struct TimerTicker {
    window: HWND,
    ticks: Arc<AtomicU64>,
    running: Cell<bool>,
}

impl TimerTicker {
    /// 窓 `window` のタイマーで刻み、刻んだ数を `ticks` で分け合う。
    pub(crate) fn new(window: HWND, ticks: Arc<AtomicU64>) -> Self {
        Self {
            window,
            ticks,
            running: Cell::new(false),
        }
    }
}

impl Ticker for TimerTicker {
    #[expect(
        unsafe_code,
        reason = "RS-010: Win32 の SetTimer を呼ぶ（ADR 0002 の 4）"
    )]
    fn start(&self) {
        // 刻んでいれば何もしない（置き直すと数え直しになり、指を動かし続ける間は刻まなくなる）。
        if self.running.replace(true) {
            return;
        }
        // SAFETY: 自分の窓に番号でタイマーを結ぶ。手続きは無く WM_TIMER で届く。
        let _timer = unsafe { SetTimer(Some(self.window), TICK_TIMER, TICK_MILLISECONDS, None) };
    }

    #[expect(
        unsafe_code,
        reason = "RS-010: Win32 の KillTimer を呼ぶ（ADR 0002 の 4）"
    )]
    fn stop(&self) {
        if !self.running.replace(false) {
            return;
        }
        // SAFETY: 自分の窓のタイマーを番号で止めるだけ。止まっていれば失敗するだけ。
        let _stopped: Result<(), _> = unsafe { KillTimer(Some(self.window), TICK_TIMER) };
        self.ticks.store(0, Ordering::Release);
    }

    fn take(&self) -> u64 {
        self.ticks.swap(0, Ordering::AcqRel)
    }
}
