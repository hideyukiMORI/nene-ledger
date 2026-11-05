//! 窓の手続きが借りる状態。画面の状態の持ち主は `Session`（ARC-011）で、ここは入力を渡して描くだけ。

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use nene_ledger_application::intent::Intent;
use nene_ledger_application::pointer_shape::PointerShape;
use nene_ledger_application::session::Session;
use nene_ledger_application::window_view::WindowView;
use nene_ledger_application::zoom_level::ZoomLevel;
use nene_ledger_core::chrome_button::ChromeButton;
use nene_ledger_core::chrome_hit::ChromeHit;
use nene_ledger_core::display_scale::DisplayScale;
use nene_ledger_core::glyph_metrics::GlyphMetrics;
use nene_ledger_core::pixel_point::PixelPoint;
use nene_ledger_core::pixel_rect::PixelRect;
use nene_ledger_core::pixel_size::PixelSize;
use nene_ledger_core::preedit::Preedit;
use nene_ledger_core::window_chrome::WindowChrome;
use nene_ledger_core::window_placement::WindowPlacement;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::Graphics::Gdi::InvalidateRect;
use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_CLOSE};

use crate::clipboard;
use crate::pen::Pen;

/// 字を測れなかったときの寸法（半角の幅・行の高さ・基準線・倍率 100% のとき）。
const FALLBACK: (u32, u32, u32) = (8, 16, 12);

/// 窓の手続きが借りる状態。描く相手（`Session`）・窓・描く道具（失敗したら作り直す）・窓の大きさと置き方・
/// 指の乗った窓のボタン・押した窓のボタン・中身の上で押しているか・拡大と画面の倍率と字の寸法・時計の刻み。
pub(crate) struct WindowState<'session> {
    session: &'session mut dyn Session,
    window: HWND,
    pen: Option<Pen>,
    size: PixelSize,
    placement: WindowPlacement,
    hovered: Option<ChromeButton>,
    pressed: Option<ChromeButton>,
    holding: bool,
    zoom: ZoomLevel,
    scale: DisplayScale,
    metrics: (GlyphMetrics, GlyphMetrics),
    ticks: Arc<AtomicU64>,
}

impl core::fmt::Debug for WindowState<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("WindowState")
            .field("size", &self.size)
            .field("placement", &self.placement)
            .field("hovered", &self.hovered)
            .field("holding", &self.holding)
            .finish_non_exhaustive()
    }
}

impl<'session> WindowState<'session> {
    /// 描く相手 `session` を窓 `window`（大きさ `size`）に、描く道具 `pen` で描く。刻みは `ticks` で時計と分け合う。
    /// 字の寸法は画面の倍率 `scale` で道具で測る（窓の部品は拡大 100%、中身は今の拡大）。
    pub(crate) fn new(
        session: &'session mut dyn Session,
        (window, size, scale): (HWND, PixelSize, DisplayScale),
        pen: Pen,
        ticks: Arc<AtomicU64>,
    ) -> Self {
        let zoom = session.zoom();
        let chrome = pen.measure(ZoomLevel::hundred(), scale).ok();
        let content = pen.measure(zoom, scale).ok();
        let fallback = GlyphMetrics::new(
            scale.px(FALLBACK.0),
            scale.px(FALLBACK.1),
            scale.px(FALLBACK.2),
        )
        .with_scale(scale);
        Self {
            session,
            window,
            pen: Some(pen),
            size,
            placement: WindowPlacement::Normal,
            hovered: None,
            pressed: None,
            holding: false,
            zoom,
            scale,
            metrics: (chrome.unwrap_or(fallback), content.unwrap_or(fallback)),
            ticks,
        }
    }

    /// 今の窓の見え方（描く相手に渡す）。
    fn view(&self) -> WindowView {
        WindowView::new(self.size, self.metrics.0, self.placement, self.hovered)
            .with_content_metrics(self.metrics.1)
    }

    /// 窓の中の位置 `(x, y)` の当たり判定（窓の部品か中身か）。
    pub(crate) fn pointer_shape(&self) -> PointerShape {
        self.session.pointer_shape()
    }

    /// 窓の中の `(x, y)` の当たり判定。
    pub(crate) fn hit(&self, x: i32, y: i32) -> ChromeHit {
        let (Ok(x), Ok(y)) = (u32::try_from(x), u32::try_from(y)) else {
            return ChromeHit::Client;
        };
        WindowChrome::new(self.size, self.metrics.0, self.placement).hit(PixelPoint::new(x, y))
    }

    /// 窓の大きさと置き方が変わった（`WM_SIZE`）。
    pub(crate) fn resized(&mut self, size: PixelSize, placement: WindowPlacement) {
        self.size = size;
        self.placement = placement;
        if let Some(pen) = &self.pen
            && pen.resize(size).is_err()
        {
            self.pen = None;
        }
        self.invalidate();
    }

    /// 窓が別の倍率の画面へ移った（`WM_DPICHANGED`）。字を測り直して描き直す。
    /// 窓の大きさは、この後で OS の勧める矩形に置き直したときの `WM_SIZE` で変わる。
    pub(crate) fn rescaled(&mut self, scale: DisplayScale) {
        if scale != self.scale {
            self.scale = scale;
            self.remeasure();
            self.invalidate();
        }
    }

    /// 拡大か画面の倍率が、測った字の寸法と違えば測り直す。道具が無いか測れなければ前の寸法のまま（次に描くときに測る）。
    fn remeasure(&mut self) {
        let (zoom, scale) = (self.session.zoom(), self.scale);
        if zoom == self.zoom && scale == self.metrics.0.scale() {
            return;
        }
        let Some(pen) = &self.pen else {
            return;
        };
        if let (Ok(chrome), Ok(content)) = (
            pen.measure(ZoomLevel::hundred(), scale),
            pen.measure(zoom, scale),
        ) {
            self.zoom = zoom;
            self.metrics = (chrome, content);
        }
    }

    /// 意図を渡す。コピーなら文字列をクリップボードに置く。
    pub(crate) fn intent(&mut self, intent: Intent) {
        let view = self.view();
        if let Some(text) = self.session.intent(intent, view) {
            // 置けなければ置かないだけ（状態の行の知らせは描く相手が出している）。
            let _placed = clipboard::put(self.window, text.as_str());
        }
        self.invalidate();
        self.follow_finish();
    }

    /// IME の組み立て（ADR 0026 の決定 4）。確定した文字列があれば先に欄へ挿し、確定前の文字列を欄に出す（`None` で消す）。
    pub(crate) fn composition(&mut self, result: Option<&str>, preedit: Option<Preedit>) {
        let view = self.view();
        if let Some(text) = result {
            self.session.commit_text(text, view);
        }
        self.session.compose(preedit, view);
        self.invalidate();
    }

    /// IME の候補の窓を置く所（編集中のセルの、窓の中の矩形）。
    pub(crate) fn composition_area(&self) -> Option<PixelRect> {
        self.session.composition_area(self.view())
    }

    /// 閉じるよう求められた（`WM_CLOSE`）。すぐ閉じてよければ `true`。だめなら尋ねる画面を描き直す（ADR 0026）。
    pub(crate) fn close(&mut self) -> bool {
        let closing = self.session.close();
        if !closing {
            self.invalidate();
        }
        closing
    }

    /// 描く相手が閉じると決めたら、`WM_CLOSE` を積む（借用を手放した後に、閉じる頼みとして届く）。
    #[expect(
        unsafe_code,
        reason = "RS-010: Win32 の PostMessageW で自分の窓に WM_CLOSE を積む（ADR 0002 の 4）"
    )]
    fn follow_finish(&self) {
        if !self.session.finished() {
            return;
        }
        // SAFETY: 自分の窓へ、値だけのメッセージを積む。
        let _posted: Result<(), _> =
            unsafe { PostMessageW(Some(self.window), WM_CLOSE, WPARAM(0), LPARAM(0)) };
    }

    /// 中身の上の左のボタンを押した（離すまで、動きはドラッグとして渡す）。
    pub(crate) fn press(&mut self, intent: Intent) {
        self.holding = true;
        self.intent(intent);
    }

    /// 左のボタンを離した（取り込みを失ったときも）。押していなければ何もしない。
    pub(crate) fn release(&mut self) {
        if self.holding {
            self.holding = false;
            self.intent(Intent::Release);
        }
    }

    /// 中身の上で指が `at` へ動いた（窓の外へ出ると負にもなる）。押していればドラッグ、そうでなければ指が乗った所を知らせる。
    pub(crate) fn moved(&mut self, (x, y): (i32, i32)) {
        self.set_hovered(None);
        if self.holding {
            let at = nene_ledger_core::pointer_point::PointerPoint::new(x, y);
            self.intent(Intent::Drag { at });
            return;
        }
        let at = u32::try_from(x)
            .ok()
            .zip(u32::try_from(y).ok())
            .map(|(x, y)| PixelPoint::new(x, y));
        self.hovered_at(at);
    }

    /// 指が中身から出た。
    pub(crate) fn left(&mut self) {
        if !self.holding {
            self.hovered_at(None);
        }
    }

    /// 指が窓のボタン `button` の上に乗った（`None` は外れた）。変われば描き直す。
    pub(crate) fn set_hovered(&mut self, button: Option<ChromeButton>) {
        if self.hovered != button {
            self.hovered = button;
            self.invalidate();
        }
    }

    /// 窓のボタン `button` を押した。
    pub(crate) fn press_button(&mut self, button: ChromeButton) {
        self.pressed = Some(button);
    }

    /// 窓のボタン `button` の上で離した。押したボタンと同じなら、そのボタン（離した後の動作をする）。
    pub(crate) fn release_button(&mut self, button: Option<ChromeButton>) -> Option<ChromeButton> {
        let pressed = self.pressed.take();
        pressed.filter(|&pressed| Some(pressed) == button)
    }

    /// 起こされた（裏の仕事の合図）。
    pub(crate) fn refresh(&mut self) {
        let view = self.view();
        self.session.refresh(view);
        self.invalidate();
        self.follow_finish();
    }

    /// 時計の刻み（`WM_TIMER`）。数えてから取り込ませる。
    pub(crate) fn tick(&mut self) {
        self.ticks.fetch_add(1, Ordering::AcqRel);
        self.refresh();
    }

    /// 描く（`WM_PAINT`）。拡大か画面の倍率が変わっていれば字を測り直す。道具が無ければ作り、描けなければ捨てて次に任せる。
    pub(crate) fn paint(&mut self) {
        if self.size.width() == 0 || self.size.height() == 0 {
            return;
        }
        if self.pen.is_none() {
            self.pen = Pen::new(self.window, self.size).ok();
        }
        self.remeasure();
        let Some(pen) = &self.pen else {
            return;
        };
        let frame = self.session.frame(self.view());
        if pen
            .paint(&frame, self.zoom, self.metrics.0.scale())
            .is_err()
        {
            self.pen = None;
        }
    }

    /// 指が乗った所を描く相手に知らせ、見た目が変われば描き直す。
    fn hovered_at(&mut self, at: Option<PixelPoint>) {
        let view = self.view();
        if self.session.hover(at, view) {
            self.invalidate();
        }
    }

    /// 窓全体の描き直しを求める。
    #[expect(
        unsafe_code,
        reason = "RS-010: Win32 の InvalidateRect を呼ぶ（ADR 0002 の 4）"
    )]
    fn invalidate(&self) {
        // SAFETY: 自分の窓に、矩形を渡さず（全体）、背景を消さずに描き直しを求めるだけ。
        let _queued = unsafe { InvalidateRect(Some(self.window), None, false) };
    }
}
