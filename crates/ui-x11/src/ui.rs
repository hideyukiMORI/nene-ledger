//! 窓を出して、閉じられるまで描き続ける。

use std::sync::Arc;

use nene_ledger_application::intent::Intent;
use nene_ledger_application::pointer_shape::PointerShape;
use nene_ledger_application::selection_mode::SelectionMode;
use nene_ledger_application::session::Session;
use nene_ledger_application::wakeup::Wakeup;
use nene_ledger_application::zoom_level::ZoomLevel;
use nene_ledger_core::chrome_hit::ChromeHit;
use nene_ledger_core::pixel_point::PixelPoint;
use nene_ledger_core::pixel_size::PixelSize;
use nene_ledger_core::pointer_point::PointerPoint;
use x11rb::protocol::Event;

use crate::chrome_action::ChromeAction;
use crate::cursor_shape::CursorShape;
use crate::frame_surface::{measure, paint};
use crate::key_intent::button_intent;
use crate::paste_receipt::PasteReceipt;
use crate::pointer_at::PointerAt;
use crate::screen::Screen;
use crate::step::Step;
use crate::ui_failure::UiFailure;
use crate::viewport::Viewport;
use crate::x11_window::{LEFT_BUTTON, X11Window};

/// 最初の窓の大きさ。画面ごとの倍率と、前回の大きさの記憶を入れるまでの仮。
const INITIAL_SIZE: PixelSize = PixelSize::new_const(1024, 640);

/// X11 の画面。窓を開き、作業のスレッドに起こす口を渡し、閉じられるまで `Session` を描く。
#[derive(Debug)]
pub struct Ui {
    window: X11Window,
}

impl Ui {
    /// 窓を開く。`title` は窓の題名（タスクバーなどウィンドウマネージャーが見せる名前）。
    ///
    /// # Errors
    ///
    /// X サーバにつなげない・画面の形式が違うとき。
    pub fn open(title: &str) -> Result<Self, UiFailure> {
        Ok(Self {
            window: X11Window::open(INITIAL_SIZE, title)?,
        })
    }

    /// 作業のスレッドからこの窓を起こす口。
    #[must_use]
    pub fn wakeup(&self) -> Arc<dyn Wakeup> {
        Arc::new(self.window.wakeup())
    }

    /// 閉じられるまで `session` を描く。起こされたら取り込み、キーとホイールは意図にして渡し、
    /// タイトルバーと縁の操作はウィンドウマネージャーに頼み、描き直す（ARC-011・ADR 0013）。
    ///
    /// # Errors
    ///
    /// 接続が切れた・描けないとき。
    pub fn run<S: Session>(self, session: &mut S) -> Result<(), UiFailure> {
        let mut screen = Screen::new(Viewport::new(
            INITIAL_SIZE,
            measure(ZoomLevel::hundred())?,
            self.window.placement()?,
        ));
        loop {
            let event = self.window.next_event()?;
            match self.respond(session, &event, &mut screen)? {
                Step::Wait => {}
                // 閉じる頼み（窓の管理者・× のボタン）は描く相手に尋ねる。未保存なら尋ねる画面を出して残る（ADR 0026）。
                Step::Quit if session.close() => return Ok(()),
                Step::Redraw | Step::Quit => self.redraw(session, &mut screen)?,
            }
            // 尋ねる画面で選んだ・保存が終わったので、描く相手が閉じると決めた。
            if session.finished() {
                return Ok(());
            }
        }
    }

    /// 描き直す。拡大が変わっていたら中身の字を測り直す（段階は application が決める・ADR 0020）。
    fn redraw<S: Session>(&self, session: &S, screen: &mut Screen) -> Result<(), UiFailure> {
        let zoom = session.zoom();
        if zoom != screen.zoom() {
            screen.set_zoom(zoom, measure(zoom)?);
        }
        let frame = session.frame(screen.view());
        self.window.present(&paint(&frame, zoom)?)
    }

    /// 1 つのイベントに応える。
    ///
    /// X のイベントは開いた集合なので `match` の `_` ではなく `if let` の連なりで書く（Rust 規約 第 4 節）。
    fn respond<S: Session>(
        &self,
        session: &mut S,
        event: &Event,
        screen: &mut Screen,
    ) -> Result<Step, UiFailure> {
        if self.window.is_close_request(event) {
            return Ok(Step::Quit);
        }
        if let Event::ConfigureNotify(configure) = event {
            let size = PixelSize::new(u32::from(configure.width), u32::from(configure.height));
            if size == screen.viewport().size() {
                return Ok(Step::Wait);
            }
            // 縮んだときは Expose が来ないので、ここで描き直す（ボタンの位置が右の端に付いて動く）。
            screen.set_viewport(screen.viewport().resized(size));
            return Ok(Step::Redraw);
        }
        if self.window.is_state_change(event) {
            screen.set_viewport(screen.viewport().placed(self.window.placement()?));
            return Ok(Step::Redraw);
        }
        let client = client_pointer_event(session, event, screen);
        if let Some(step) = self.pointer_event(event, screen, session.pointer_shape())? {
            return Ok(merged(client, step));
        }
        if let Some(step) = client {
            return Ok(step);
        }
        if let Some(step) = self.intent_event(session, event, screen)? {
            return Ok(step);
        }
        if let Some(step) = self.clipboard_event(event, screen)? {
            return Ok(step);
        }
        match self.window.pasted_text(event, screen.incoming_paste())? {
            Some(PasteReceipt::Done(text)) => {
                session.paste(&text, screen.view());
                return Ok(Step::Redraw);
            }
            Some(PasteReceipt::Pending) => return Ok(Step::Wait),
            None => {}
        }
        if self.window.is_wake(event) {
            session.refresh(screen.view());
            return Ok(Step::Redraw);
        }
        Ok(
            if matches!(event, Event::Expose(expose) if expose.count == 0) {
                Step::Redraw
            } else {
                Step::Wait
            },
        )
    }

    /// キーとボタンを意図にして渡す。コピーで文字列が返れば、そのキーの時刻でクリップボードの持ち主になる。
    /// 意図にならなければ `None`。
    fn intent_event<S: Session>(
        &self,
        session: &mut S,
        event: &Event,
        screen: &mut Screen,
    ) -> Result<Option<Step>, UiFailure> {
        let (intent, time) = if let Event::KeyPress(key) = event {
            (self.window.key_intent(key.detail, key.state), key.time)
        } else if let Event::ButtonPress(button) = event {
            let intent = button_intent(button.detail, button.state, button.event_x, button.event_y);
            // 表の上の 2 度目の左の押しはダブルクリック（Shift・Ctrl を押していないとき・ADR 0026 の決定 1）。
            let intent = match intent {
                Some(Intent::Point {
                    at,
                    mode: SelectionMode::Collapse,
                }) if screen.pointer_mut().client_press(PointerAt::new(
                    button.root_x,
                    button.root_y,
                    button.time,
                )) =>
                {
                    Some(Intent::DoubleClick { at })
                }
                other => other,
            };
            (intent, button.time)
        } else {
            return Ok(None);
        };
        let Some(intent) = intent else {
            return Ok(None);
        };
        if intent == Intent::Paste {
            // 自分が持ち主なら、持っている文字列をそのまま貼る。そうでなければ持ち主に求め、答えを待つ（CLIP-01）。
            if let Some(text) = screen.clipboard().map(|text| String::from(text.as_str())) {
                session.paste(&text, screen.view());
            } else {
                self.window.request_clipboard(time)?;
            }
            return Ok(Some(Step::Redraw));
        }
        if let Intent::Point { .. } | Intent::DoubleClick { .. } = intent {
            // 離すまで、指の動きはドラッグとして中身へ渡す（押している間は X が指を窓に結び付ける）。
            screen.set_holding(true);
        }
        if let Some(text) = session.intent(intent, screen.view())
            && self.window.own_clipboard(time)?
        {
            screen.set_clipboard(Some(text));
        }
        Ok(Some(Step::Redraw))
    }

    /// クリップボードの求めに応え、手放す知らせで中身を捨てる。クリップボードのイベントでなければ `None`。
    fn clipboard_event(
        &self,
        event: &Event,
        screen: &mut Screen,
    ) -> Result<Option<Step>, UiFailure> {
        if let Event::SelectionRequest(request) = event {
            self.window.answer_clipboard(request, screen.clipboard())?;
            return Ok(Some(Step::Wait));
        }
        if self.window.is_clipboard_lost(event) {
            screen.set_clipboard(None);
            return Ok(Some(Step::Wait));
        }
        Ok(None)
    }

    /// 左のボタン・マウスの動き・窓の外へ出たこと。飾りの操作に当たらなければ `None`（ホイールなどは次へ回す）。
    /// `shape` は中身が求めるカーソルの形（中身へ渡した後のもの）。
    fn pointer_event(
        &self,
        event: &Event,
        screen: &mut Screen,
        shape: PointerShape,
    ) -> Result<Option<Step>, UiFailure> {
        let viewport = screen.viewport();
        let (action, at) = if let Event::ButtonPress(press) = event {
            if press.detail != LEFT_BUTTON {
                return Ok(None);
            }
            let at = PointerAt::new(press.root_x, press.root_y, press.time);
            let hit = viewport.hit(press.event_x, press.event_y);
            if hit == ChromeHit::Client {
                // 中身のクリックはセルを選ぶ意図にする（次の段で）。
                return Ok(None);
            }
            (screen.pointer_mut().press(hit, at), at)
        } else if let Event::ButtonRelease(release) = event {
            if release.detail != LEFT_BUTTON {
                return Ok(None);
            }
            let at = PointerAt::new(release.root_x, release.root_y, release.time);
            let hit = viewport.hit(release.event_x, release.event_y);
            (screen.pointer_mut().release(hit), at)
        } else if let Event::MotionNotify(motion) = event {
            let at = PointerAt::new(motion.root_x, motion.root_y, motion.time);
            let hit = viewport.hit(motion.event_x, motion.event_y);
            self.show_cursor(CursorShape::of(hit, shape), screen)?;
            (screen.pointer_mut().motion(hit, at), at)
        } else if let Event::LeaveNotify(leave) = event {
            let at = PointerAt::new(leave.root_x, leave.root_y, leave.time);
            (screen.pointer_mut().leave(), at)
        } else {
            return Ok(None);
        };
        self.perform(action, at).map(Some)
    }

    /// カーソルを `shape` にする。変わったときだけ X に送る。
    fn show_cursor(&self, shape: CursorShape, screen: &mut Screen) -> Result<(), UiFailure> {
        if shape != screen.cursor() {
            self.window.set_cursor(shape)?;
            screen.set_cursor(shape);
        }
        Ok(())
    }

    /// 飾りの操作が求めることを、ウィンドウマネージャーへの頼みに写す（ADR 0013 の決定 2）。
    fn perform(&self, action: ChromeAction, at: PointerAt) -> Result<Step, UiFailure> {
        match action {
            ChromeAction::Nothing => return Ok(Step::Wait),
            ChromeAction::Redraw => return Ok(Step::Redraw),
            ChromeAction::Close => return Ok(Step::Quit),
            ChromeAction::Move => self.window.begin_move_resize(None, at)?,
            ChromeAction::Resize(edge) => {
                self.window.begin_move_resize(Some(edge), at)?;
            }
            ChromeAction::ToggleMaximize => self.window.toggle_maximize()?,
            ChromeAction::Minimize => self.window.minimize()?,
        }
        Ok(Step::Wait)
    }
}

/// 中身へ渡す指の動き。中身の上で押したまま動かせばドラッグ、離せば終わり。押していなければ指が乗った所を知らせる。
/// 中身へ渡すイベントでなければ `None`（飾りの操作は次の段で）。
fn client_pointer_event<S: Session>(
    session: &mut S,
    event: &Event,
    screen: &mut Screen,
) -> Option<Step> {
    if let Event::MotionNotify(motion) = event {
        if screen.holding() {
            let at = PointerPoint::new(i32::from(motion.event_x), i32::from(motion.event_y));
            session.intent(Intent::Drag { at }, screen.view());
            return Some(Step::Redraw);
        }
        let at = (screen.viewport().hit(motion.event_x, motion.event_y) == ChromeHit::Client)
            .then(|| {
                Some(PixelPoint::new(
                    u32::try_from(motion.event_x).ok()?,
                    u32::try_from(motion.event_y).ok()?,
                ))
            })
            .flatten();
        return session.hover(at, screen.view()).then_some(Step::Redraw);
    }
    if let Event::ButtonRelease(release) = event
        && release.detail == LEFT_BUTTON
        && screen.holding()
    {
        screen.set_holding(false);
        session.intent(Intent::Release, screen.view());
        return Some(Step::Redraw);
    }
    if let Event::LeaveNotify(_) = event
        && !screen.holding()
    {
        return session.hover(None, screen.view()).then_some(Step::Redraw);
    }
    None
}

/// 中身へ渡した結果 `client` と、飾りの操作の結果 `chrome` を合わせる（どちらかが描き直すなら描き直す）。
fn merged(client: Option<Step>, chrome: Step) -> Step {
    match (client, chrome) {
        (_, Step::Quit) => Step::Quit,
        (Some(Step::Redraw), Step::Wait | Step::Redraw) | (_, Step::Redraw) => Step::Redraw,
        (Some(Step::Wait | Step::Quit) | None, Step::Wait) => Step::Wait,
    }
}
