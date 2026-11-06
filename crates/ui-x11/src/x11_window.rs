//! X11 の窓 1 つ。描いた面を転送し、イベントを受け取る。

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use cairo::ImageSurface;
use nene_ledger_application::clipboard_text::ClipboardText;
use nene_ledger_application::intent::Intent;
use nene_ledger_core::pixel_size::PixelSize;
use nene_ledger_core::resize_edge::ResizeEdge;
use nene_ledger_core::window_placement::WindowPlacement;
use x11rb::connection::{Connection, RequestConnection};
use x11rb::protocol::Event;
use x11rb::protocol::xproto::{
    AtomEnum, ChangeWindowAttributesAux, ClientMessageEvent, ConnectionExt, CreateGCAux,
    CreateWindowAux, Cursor, EventMask, Gcontext, GetPropertyReply, ImageFormat, ImageOrder,
    KeyButMask, PropMode, Property, SELECTION_NOTIFY_EVENT, SelectionNotifyEvent,
    SelectionRequestEvent, Window, WindowClass,
};
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;

use crate::chrome_action::{MOVERESIZE_MOVE, cursor_glyph, moveresize_direction};
use crate::cursor_shape::CursorShape;
use crate::incoming_paste::IncomingPaste;
use crate::key_intent::key_intent;
use crate::key_map::KeyMap;
use crate::paste_receipt::PasteReceipt;
use crate::pointer_at::PointerAt;
use crate::ui_failure::UiFailure;
use crate::window_atoms::WindowAtoms;
use crate::x11_wakeup::X11Wakeup;

/// 描いた面の色の深さ（cairo の RGB24）。
const DEPTH: u8 = 24;
/// 描いた面の 1 画素のバイト数。
const BYTES_PER_PIXEL: usize = 4;
/// 要求の頭の分（`PutImage` の固定部）として、上限から引いておくバイト数。
const REQUEST_HEADER: usize = 64;
/// `_MOTIF_WM_HINTS` の「飾りを指定する」印（`MWM_HINTS_DECORATIONS`）。飾りは 0（無し）にする。
const MWM_HINTS_DECORATIONS: u32 = 2;
/// `_NET_WM_STATE` の切り替え（`_NET_WM_STATE_TOGGLE`）。
const NET_WM_STATE_TOGGLE: u32 = 2;
/// `WM_CHANGE_STATE` の最小化（ICCCM の `IconicState`）。
const ICONIC_STATE: u32 = 3;
/// EWMH の頼みの出どころ（通常のアプリ）。
const SOURCE_APPLICATION: u32 = 1;
/// 左のボタン。
pub(crate) const LEFT_BUTTON: u8 = 1;

/// カーソルの並び。先頭が矢印、続いて 8 方向の縁。
/// X11 の窓 1 つ。窓・描画の文脈・閉じる合図と起こす合図の原子を持つ。資源は接続を閉じるとサーバが片付ける。
///
/// 接続は作業のスレッドの `X11Wakeup` と共有する（`RustConnection` は複数のスレッドから使える）。
#[derive(Debug)]
pub(crate) struct X11Window {
    connection: Arc<RustConnection>,
    root: Window,
    window: Window,
    cursors: [Cursor; 10],
    gc: Gcontext,
    atoms: WindowAtoms,
    pending_wake: Arc<AtomicBool>,
    keys: KeyMap,
}

impl X11Window {
    /// 窓を作って見せる。`title` は窓の題名（UTF-8）。
    pub(crate) fn open(size: PixelSize, title: &str) -> Result<Self, UiFailure> {
        let (connection, screen_number) = x11rb::connect(None)?;
        let screen = connection
            .setup()
            .roots
            .get(screen_number)
            .ok_or(UiFailure::UnsupportedScreen)?
            .clone();
        let supported = screen.root_depth == DEPTH
            && connection.setup().image_byte_order == ImageOrder::LSB_FIRST
            && connection
                .setup()
                .pixmap_formats
                .iter()
                .any(|format| format.depth == DEPTH && format.bits_per_pixel == 32);
        if !supported {
            return Err(UiFailure::UnsupportedScreen);
        }
        let window = connection.generate_id()?;
        connection.create_window(
            DEPTH,
            window,
            screen.root,
            0,
            0,
            u16::try_from(size.width())?,
            u16::try_from(size.height())?,
            0,
            WindowClass::INPUT_OUTPUT,
            screen.root_visual,
            &CreateWindowAux::new()
                .background_pixel(screen.white_pixel)
                .event_mask(
                    EventMask::EXPOSURE
                        | EventMask::STRUCTURE_NOTIFY
                        | EventMask::PROPERTY_CHANGE
                        | EventMask::KEY_PRESS
                        | EventMask::BUTTON_PRESS
                        | EventMask::BUTTON_RELEASE
                        | EventMask::POINTER_MOTION
                        | EventMask::LEAVE_WINDOW,
                ),
        )?;
        let gc = connection.generate_id()?;
        connection.create_gc(gc, window, &CreateGCAux::new())?;
        let atoms = Self::name_window(&connection, window, title)?;
        Self::remove_decorations(&connection, window, atoms)?;
        let keys = Self::key_map(&connection)?;
        let cursors = Self::cursors(&connection)?;
        connection.map_window(window)?;
        connection.flush()?;
        Ok(Self {
            connection: Arc::new(connection),
            root: screen.root,
            window,
            cursors,
            gc,
            atoms,
            pending_wake: Arc::new(AtomicBool::new(false)),
            keys,
        })
    }

    /// サーバからキーの対応表を受け取る。
    fn key_map(connection: &RustConnection) -> Result<KeyMap, UiFailure> {
        let setup = connection.setup();
        let (first, last) = (setup.min_keycode, setup.max_keycode);
        let count = last.saturating_sub(first).saturating_add(1);
        let reply = connection.get_keyboard_mapping(first, count)?.reply()?;
        Ok(KeyMap::new(first, reply.keysyms_per_keycode, reply.keysyms))
    }

    /// キーの意図。`state` は修飾キー。打たれる字は Shift の欄も引く。
    pub(crate) fn key_intent(&self, keycode: u8, state: KeyButMask) -> Option<Intent> {
        let typed = self.keys.typed(keycode, state.contains(KeyButMask::SHIFT));
        self.keys
            .keysym(keycode)
            .and_then(|keysym| key_intent(keysym, typed, state))
    }

    /// 題名を付け、閉じる合図（`WM_DELETE_WINDOW`）を受け取ると告げる。閉じる合図と起こす合図の原子を返す。
    fn name_window(
        connection: &RustConnection,
        window: Window,
        title: &str,
    ) -> Result<WindowAtoms, UiFailure> {
        let atoms = WindowAtoms::intern(connection)?;
        connection.change_property32(
            PropMode::REPLACE,
            window,
            atoms.protocols(),
            AtomEnum::ATOM,
            &[atoms.delete_window()],
        )?;
        connection.change_property8(
            PropMode::REPLACE,
            window,
            atoms.net_wm_name(),
            atoms.utf8_string(),
            title.as_bytes(),
        )?;
        Ok(atoms)
    }

    /// ウィンドウマネージャーに、タイトルバーと枠を付けないよう頼む（`_MOTIF_WM_HINTS`・ADR 0013 の決定 2）。
    fn remove_decorations(
        connection: &RustConnection,
        window: Window,
        atoms: WindowAtoms,
    ) -> Result<(), UiFailure> {
        // flags・functions・decorations・input_mode・status の 5 つ。
        connection.change_property32(
            PropMode::REPLACE,
            window,
            atoms.motif_wm_hints(),
            atoms.motif_wm_hints(),
            &[MWM_HINTS_DECORATIONS, 0, 0, 0, 0],
        )?;
        Ok(())
    }

    /// カーソルの字体から、矢印・8 方向の縁・列の幅のカーソルを作る（並びは `CursorShape::ALL`）。
    fn cursors(connection: &RustConnection) -> Result<[Cursor; 10], UiFailure> {
        let font = connection.generate_id()?;
        connection.open_font(font, b"cursor")?;
        let mut cursors = [0; 10];
        for (slot, shape) in cursors.iter_mut().zip(CursorShape::ALL) {
            let glyph = cursor_glyph(shape);
            *slot = connection.generate_id()?;
            connection.create_glyph_cursor(
                *slot,
                font,
                font,
                glyph,
                glyph.saturating_add(1),
                0,
                0,
                0,
                u16::MAX,
                u16::MAX,
                u16::MAX,
            )?;
        }
        connection.close_font(font)?;
        Ok(cursors)
    }

    /// マウスのカーソルを `shape` にする。
    pub(crate) fn set_cursor(&self, shape: CursorShape) -> Result<(), UiFailure> {
        let index = CursorShape::ALL
            .iter()
            .position(|candidate| *candidate == shape)
            .unwrap_or(0);
        let cursor = self.cursors.get(index).copied().unwrap_or(x11rb::NONE);
        self.connection.change_window_attributes(
            self.window,
            &ChangeWindowAttributesAux::new().cursor(cursor),
        )?;
        self.connection.flush()?;
        Ok(())
    }

    /// ウィンドウマネージャーに、画面の位置 `at` から窓を動かす（`edge` が無ければ）か、`edge` の向きにリサイズするよう頼む。
    /// 左のボタンを押したまま頼み、ウィンドウマネージャーがマウスを掴めるよう、先に自分の掴みを放す（EWMH）。
    pub(crate) fn begin_move_resize(
        &self,
        edge: Option<ResizeEdge>,
        at: PointerAt,
    ) -> Result<(), UiFailure> {
        let direction = edge.map_or(MOVERESIZE_MOVE, moveresize_direction);
        self.connection.ungrab_pointer(at.time())?;
        self.send_to_window_manager(
            self.atoms.net_wm_moveresize(),
            [
                i32::from(at.root_x()).cast_unsigned(),
                i32::from(at.root_y()).cast_unsigned(),
                direction,
                u32::from(LEFT_BUTTON),
                SOURCE_APPLICATION,
            ],
        )
    }

    /// ウィンドウマネージャーに、最大化と元に戻すを切り替えるよう頼む（`_NET_WM_STATE`）。
    pub(crate) fn toggle_maximize(&self) -> Result<(), UiFailure> {
        self.send_to_window_manager(
            self.atoms.net_wm_state(),
            [
                NET_WM_STATE_TOGGLE,
                self.atoms.maximized_vert(),
                self.atoms.maximized_horz(),
                SOURCE_APPLICATION,
                0,
            ],
        )
    }

    /// ウィンドウマネージャーに、最小化するよう頼む（ICCCM の `WM_CHANGE_STATE`）。
    pub(crate) fn minimize(&self) -> Result<(), UiFailure> {
        self.send_to_window_manager(self.atoms.wm_change_state(), [ICONIC_STATE, 0, 0, 0, 0])
    }

    /// ルートの窓へ、この窓についての頼みを送る（EWMH の約束の宛先と印）。
    fn send_to_window_manager(&self, type_: u32, payload: [u32; 5]) -> Result<(), UiFailure> {
        let message = ClientMessageEvent::new(32, self.window, type_, payload);
        self.connection.send_event(
            false,
            self.root,
            EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
            message,
        )?;
        self.connection.flush()?;
        Ok(())
    }

    /// 今の置かれ方。`_NET_WM_STATE` に縦と横の最大化が両方あれば最大化。
    pub(crate) fn placement(&self) -> Result<WindowPlacement, UiFailure> {
        let reply = self
            .connection
            .get_property(
                false,
                self.window,
                self.atoms.net_wm_state(),
                AtomEnum::ATOM,
                0,
                u32::from(u8::MAX),
            )?
            .reply()?;
        let states: Vec<u32> = reply.value32().into_iter().flatten().collect();
        let maximized = states.contains(&self.atoms.maximized_vert())
            && states.contains(&self.atoms.maximized_horz());
        Ok(if maximized {
            WindowPlacement::Maximized
        } else {
            WindowPlacement::Normal
        })
    }

    /// 置かれ方が変わったかもしれない知らせ（`_NET_WM_STATE` の書き換え）か。
    pub(crate) fn is_state_change(&self, event: &Event) -> bool {
        matches!(event, Event::PropertyNotify(notify)
            if notify.window == self.window && notify.atom == self.atoms.net_wm_state())
    }

    /// 時刻 `time` で `CLIPBOARD` の持ち主になる。なれたら `true`。
    pub(crate) fn own_clipboard(&self, time: u32) -> Result<bool, UiFailure> {
        let clipboard = self.atoms.clipboard();
        self.connection
            .set_selection_owner(self.window, clipboard, time)?;
        let owner = self
            .connection
            .get_selection_owner(clipboard)?
            .reply()?
            .owner;
        Ok(owner == self.window)
    }

    /// クリップボードを手放す知らせ（別のアプリが持ち主になった）か。
    pub(crate) fn is_clipboard_lost(&self, event: &Event) -> bool {
        matches!(event, Event::SelectionClear(clear)
            if clear.owner == self.window && clear.selection == self.atoms.clipboard())
    }

    /// `CLIPBOARD` の中身を `UTF8_STRING` で、自分の窓の属性に書いてもらうよう求める（貼り付け・CLIP-01）。
    /// 答えは `SelectionNotify` で届く（[`Self::pasted_text`]）。`time` は Ctrl+V のキーの時刻。
    pub(crate) fn request_clipboard(&self, time: u32) -> Result<(), UiFailure> {
        self.connection.convert_selection(
            self.window,
            self.atoms.clipboard(),
            self.atoms.utf8_string(),
            self.atoms.paste(),
            time,
        )?;
        self.connection.flush()?;
        Ok(())
    }

    /// 求めた貼り付けの答えか、分けて渡される中身の続きなら、その受け取り（ICCCM の `INCR` も受ける）。
    /// どちらでもなければ `None`。読んだ属性は消す（持ち主への「次を送ってよい」の合図にもなる）。断られたら空の文字列。
    /// 分けて受け取っている途中の中身は `incoming` に溜める。
    pub(crate) fn pasted_text(
        &self,
        event: &Event,
        incoming: &mut Option<IncomingPaste>,
    ) -> Result<Option<PasteReceipt>, UiFailure> {
        if let Event::PropertyNotify(notify) = event
            && notify.window == self.window
            && notify.atom == self.atoms.paste()
            && notify.state == Property::NEW_VALUE
            && let Some(mut receiving) = incoming.take()
        {
            let part = self.take_paste_property()?;
            if part.value.is_empty() {
                return Ok(Some(PasteReceipt::Done(receiving.finish())));
            }
            receiving.push(&part.value);
            *incoming = Some(receiving);
            return Ok(Some(PasteReceipt::Pending));
        }
        let Event::SelectionNotify(notify) = event else {
            return Ok(None);
        };
        if notify.requestor != self.window || notify.selection != self.atoms.clipboard() {
            return Ok(None);
        }
        if notify.property == x11rb::NONE {
            return Ok(Some(PasteReceipt::Done(String::new())));
        }
        let reply = self.take_paste_property()?;
        if reply.type_ == self.atoms.incr() {
            // 属性を消したので、持ち主が分けて書き始める（PropertyNotify の NewValue で届く）。
            *incoming = Some(IncomingPaste::new());
            return Ok(Some(PasteReceipt::Pending));
        }
        Ok(Some(PasteReceipt::Done(
            String::from_utf8_lossy(&reply.value).into_owned(),
        )))
    }

    /// 貼り付けの属性を読んで消す。
    fn take_paste_property(&self) -> Result<GetPropertyReply, UiFailure> {
        let length = u32::try_from(self.connection.maximum_request_bytes() / 4).unwrap_or(u32::MAX);
        Ok(self
            .connection
            .get_property(
                true,
                self.window,
                self.atoms.paste(),
                AtomEnum::ANY,
                0,
                length,
            )?
            .reply()?)
    }

    /// 別のアプリからの `CLIPBOARD` の求めに応える。`TARGETS` には渡せる形の一覧を、`UTF8_STRING` と
    /// `text/plain;charset=utf-8` には `text` を、求めた窓の属性に書く。持っていない・1 つの要求に収まらない・
    /// 知らない形なら断る（属性を `NONE` にして知らせる）。
    pub(crate) fn answer_clipboard(
        &self,
        request: &SelectionRequestEvent,
        text: Option<&ClipboardText>,
    ) -> Result<(), UiFailure> {
        // 古いクライアントは属性を NONE で求める。そのときは形の名前を属性に使う（ICCCM）。
        let property = if request.property == x11rb::NONE {
            request.target
        } else {
            request.property
        };
        let answered = self.write_clipboard(request, property, text)?;
        let notify = SelectionNotifyEvent {
            response_type: SELECTION_NOTIFY_EVENT,
            sequence: 0,
            time: request.time,
            requestor: request.requestor,
            selection: request.selection,
            target: request.target,
            property: if answered { property } else { x11rb::NONE },
        };
        self.connection
            .send_event(false, request.requestor, EventMask::NO_EVENT, notify)?;
        self.connection.flush()?;
        Ok(())
    }

    /// 求めに合う中身を、求めた窓の属性 `property` に書く。書いたら `true`。
    fn write_clipboard(
        &self,
        request: &SelectionRequestEvent,
        property: u32,
        text: Option<&ClipboardText>,
    ) -> Result<bool, UiFailure> {
        let Some(text) = text.filter(|_| request.selection == self.atoms.clipboard()) else {
            return Ok(false);
        };
        let utf8 = [self.atoms.utf8_string(), self.atoms.text_plain_utf8()];
        if request.target == self.atoms.targets() {
            let offered = [self.atoms.targets(), utf8[0], utf8[1]];
            self.connection.change_property32(
                PropMode::REPLACE,
                request.requestor,
                property,
                AtomEnum::ATOM,
                &offered,
            )?;
            return Ok(true);
        }
        let fits = text.as_str().len().saturating_add(REQUEST_HEADER)
            <= self.connection.maximum_request_bytes();
        if !utf8.contains(&request.target) || !fits {
            return Ok(false);
        }
        self.connection.change_property8(
            PropMode::REPLACE,
            request.requestor,
            property,
            request.target,
            text.as_str().as_bytes(),
        )?;
        Ok(true)
    }

    /// 作業のスレッドからこの窓を起こす口。
    pub(crate) fn wakeup(&self) -> X11Wakeup {
        X11Wakeup::new(
            Arc::clone(&self.connection),
            self.window,
            self.atoms.wake(),
            Arc::clone(&self.pending_wake),
        )
    }

    /// 起こす合図か。そうなら「届く前の合図がある」印を下ろす（次の合図を送れるようにする）。
    pub(crate) fn is_wake(&self, event: &Event) -> bool {
        let Event::ClientMessage(message) = event else {
            return false;
        };
        let wake = message.window == self.window && message.type_ == self.atoms.wake();
        if wake {
            self.pending_wake.store(false, Ordering::Release);
        }
        wake
    }

    /// 次のイベントを待つ。
    pub(crate) fn next_event(&self) -> Result<Event, UiFailure> {
        Ok(self.connection.wait_for_event()?)
    }

    /// 閉じる合図か。
    pub(crate) fn is_close_request(&self, event: &Event) -> bool {
        let Event::ClientMessage(message) = event else {
            return false;
        };
        message.window == self.window
            && message.data.as_data32().first() == Some(&self.atoms.delete_window())
    }

    /// 描き終えた面を窓の左上へ転送する。要求の大きさの上限に合わせて、行の帯に分けて送る。
    pub(crate) fn present(&self, surface: &ImageSurface) -> Result<(), UiFailure> {
        let width = u16::try_from(surface.width())?;
        let stride = usize::try_from(surface.stride())?;
        if stride != usize::from(width).saturating_mul(BYTES_PER_PIXEL) {
            return Err(UiFailure::UnsupportedScreen);
        }
        let budget = self
            .connection
            .maximum_request_bytes()
            .saturating_sub(REQUEST_HEADER);
        let rows_per_strip = (budget / stride.max(1)).max(1);
        let mut outcome = Ok(());
        surface.with_data(|bytes| {
            outcome = self.put_strips(bytes, width, stride.saturating_mul(rows_per_strip));
        })?;
        outcome?;
        self.connection.flush()?;
        Ok(())
    }

    /// 面のバイト列を、`strip_bytes` ずつの帯にして送る。
    fn put_strips(&self, bytes: &[u8], width: u16, strip_bytes: usize) -> Result<(), UiFailure> {
        let row_bytes = usize::from(width).saturating_mul(BYTES_PER_PIXEL).max(1);
        for (index, strip) in bytes.chunks(strip_bytes.max(1)).enumerate() {
            let top = index.saturating_mul(strip_bytes) / row_bytes;
            self.connection.put_image(
                ImageFormat::Z_PIXMAP,
                self.window,
                self.gc,
                width,
                u16::try_from(strip.len() / row_bytes)?,
                0,
                i16::try_from(top)?,
                0,
                DEPTH,
                strip,
            )?;
        }
        Ok(())
    }
}
