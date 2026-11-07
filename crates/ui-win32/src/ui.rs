//! Windows の窓（Issue #55）。X11 版の `Ui` と同じ口で、窓を作り、メッセージを状態へ渡し、閉じられるまで回す。
//!
//! 窓は `WS_OVERLAPPEDWINDOW` のまま `WM_NCCALCSIZE` で枠とタイトルバーを消し、`WM_NCHITTEST` で core の当たり判定を
//! OS の領域に写す（ADR 0013 の 1。スナップ・`Win+矢印`・ダブルクリックで最大化・`Alt+Space` は OS が行う）。
//! 窓の手続きは `GWLP_USERDATA` に結んだ `RefCell<WindowState>` を借りる。借りている最中に同じスレッドへ同期的に届いた
//! メッセージは、借りられないので既定の手続きへ流れる（NeNeDocket と同じ）。

use core::cell::RefCell;
use core::ffi::c_void;
use core::ptr;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use nene_ledger_application::pointer_shape::PointerShape;
use nene_ledger_application::session::Session;
use nene_ledger_application::ticker::Ticker;
use nene_ledger_application::wakeup::Wakeup;
use nene_ledger_core::chrome_button::ChromeButton;
use nene_ledger_core::display_scale::DisplayScale;
use nene_ledger_core::pixel_size::PixelSize;
use nene_ledger_core::preedit::Preedit;
use nene_ledger_core::window_placement::WindowPlacement;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{
    DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND, DwmExtendFrameIntoClientArea,
    DwmSetWindowAttribute,
};
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, PAINTSTRUCT, ScreenToClient};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::MARGINS;
use windows::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, GetDpiForWindow, GetSystemMetricsForDpi,
    SetProcessDpiAwarenessContext,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, ReleaseCapture, SetCapture, TME_LEAVE, TME_NONCLIENT, TRACKMOUSEEVENT,
    TrackMouseEvent, VK_CONTROL, VK_MENU, VK_RETURN, VK_SHIFT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CS_DBLCLKS, CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DispatchMessageW, GWLP_USERDATA,
    GetClientRect, GetMessageW, GetWindowLongPtrW, IDC_ARROW, IDC_SIZEWE, IsZoomed, LoadCursorW,
    MSG, NCCALCSIZE_PARAMS, PostMessageW, PostQuitMessage, RegisterClassExW, SM_CXFRAME,
    SM_CXPADDEDBORDER, SM_CYFRAME, SW_SHOW, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE,
    SWP_NOSIZE, SWP_NOZORDER, SetCursor, SetWindowLongPtrW, SetWindowPos, ShowWindow,
    TranslateMessage, WINDOW_EX_STYLE, WM_CAPTURECHANGED, WM_CHAR, WM_CLOSE, WM_DESTROY,
    WM_DPICHANGED, WM_ERASEBKGND, WM_IME_COMPOSITION, WM_IME_ENDCOMPOSITION, WM_IME_SETCONTEXT,
    WM_IME_STARTCOMPOSITION, WM_KEYDOWN, WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_LBUTTONUP,
    WM_MOUSEHWHEEL, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_NCCALCSIZE, WM_NCHITTEST, WM_NCLBUTTONDBLCLK,
    WM_NCLBUTTONDOWN, WM_NCLBUTTONUP, WM_NCMOUSEMOVE, WM_PAINT, WM_SETCURSOR, WM_SIZE, WM_SYSCHAR,
    WM_SYSCOMMAND, WM_SYSKEYDOWN, WM_TIMER, WNDCLASSEXW, WS_OVERLAPPEDWINDOW,
};
use windows::core::{HSTRING, PCWSTR, w};

use crate::hit_code::{HTCLIENT, button_of, hit_code};
use crate::ime;
use crate::key_intent::{
    Modifiers, double_press_intent, key_intent, press_intent, sideways_wheel_intent, typed_intent,
    wheel_intent,
};
use crate::pen::Pen;
use crate::timer_ticker::{TICK_TIMER, TimerTicker};
use crate::win32_failure::Win32Failure;
use crate::win32_wakeup::{WM_WAKE, Win32Wakeup};
use crate::window_state::WindowState;

/// 窓の類の名。
const CLASS: PCWSTR = w!("NeNeLedger");
/// 最初の窓の大きさ（画素・倍率 100% のとき・X11 版と同じ）。
const INITIAL_WIDTH: u32 = 1024;
const INITIAL_HEIGHT: u32 = 640;
/// `WM_MOUSELEAVE` と `WM_NCMOUSELEAVE`（winuser.h の 0x02A3・0x02A2）。
/// `WM_SYSKEYDOWN` で受ける Alt のキー：Enter（セルの中の改行）・C・W・R・Z・K（検索の欄の条件・ADR 0030 の決定 3・
/// ADR 0031・GitHub Issue #199）・A（全部の置換・決定 6）。🔴 `key_intent` が Alt で意図を返すキーは、ここと `ALT_CHARS`
/// にも足す（足さないと Windows では OS に渡って効かず、警告の音が鳴る・テスト `every_alt_key_is_received`）。
const ALT_KEYS: [u16; 7] = [VK_RETURN.0, 0x43, 0x57, 0x52, 0x5A, 0x4B, 0x41];
/// 受けた Alt のキーに続く `WM_SYSCHAR`（OS に渡すと警告の音が鳴る）。
const ALT_CHARS: &[u8; 13] = b"\rcCwWrRzZkKaA";

const WM_MOUSELEAVE: u32 = 0x02A3;
const WM_NCMOUSELEAVE: u32 = 0x02A2;
/// `WM_SYSCOMMAND` の命令（winuser.h の `SC_MINIMIZE`・`SC_MAXIMIZE`・`SC_CLOSE`・`SC_RESTORE`）。
const SC_MINIMIZE: usize = 0xF020;
const SC_MAXIMIZE: usize = 0xF030;
const SC_CLOSE: usize = 0xF060;
const SC_RESTORE: usize = 0xF120;
/// `WM_SIZE` の wparam の「最大化した」（`SIZE_MAXIMIZED`）。
const SIZE_MAXIMIZED: usize = 2;
/// `WM_LBUTTONDOWN` の wparam の Shift と Ctrl（`MK_SHIFT`・`MK_CONTROL`）。
const MK_SHIFT: usize = 0x0004;
const MK_CONTROL: usize = 0x0008;

/// 窓の手続きが借りる状態（手続きの中では借用の長さを忘れる。`run` の間だけ辿られる）。
type StateCell = RefCell<WindowState<'static>>;

/// Windows の窓。作った窓と、時計の刻みの数（窓の手続きと時計が分け合う）。
#[derive(Debug)]
pub struct Ui {
    window: HWND,
    ticks: Arc<AtomicU64>,
}

impl Ui {
    /// 題名 `title` の窓を作る（まだ見せない）。プロセスを per-monitor v2 の倍率の対応にし、窓の出る画面の倍率で
    /// 大きさを決め、影と角丸を DWM に頼む。
    ///
    /// # Errors
    ///
    /// 窓を作れない。
    pub fn open(title: &str) -> Result<Self, Win32Failure> {
        declare_dpi_awareness();
        register_class()?;
        let window = create(title)?;
        fit_initial_size(window);
        decorate(window);
        Ok(Self {
            window,
            ticks: Arc::new(AtomicU64::new(0)),
        })
    }

    /// 作業のスレッドからこの窓を起こす口。
    #[must_use]
    pub fn wakeup(&self) -> Arc<dyn Wakeup> {
        Arc::new(Win32Wakeup::new(self.window))
    }

    /// 押し続けとドラッグの時計（窓のタイマー・50ms）。
    #[must_use]
    pub fn ticker(&self) -> Box<dyn Ticker> {
        Box::new(TimerTicker::new(self.window, Arc::clone(&self.ticks)))
    }

    /// 閉じられるまで `session` を描く。
    ///
    /// # Errors
    ///
    /// 描く道具を作れない。
    #[expect(
        unsafe_code,
        reason = "RS-010: Win32 の窓に状態を結び、メッセージのループを回す（ADR 0002 の 4）"
    )]
    pub fn run<S: Session>(self, session: &mut S) -> Result<(), Win32Failure> {
        let size = client_size(self.window);
        let pen = Pen::new(self.window, size)?;
        let scale = scale_of(self.window);
        let state = WindowState::new(
            session,
            (self.window, size, scale),
            pen,
            Arc::clone(&self.ticks),
        );
        let cell = RefCell::new(state);
        // 窓の手続きは借用の長さを知らない。結ぶのはこの関数の間だけで、終わる前に外す。
        let address = (&raw const cell)
            .cast::<StateCell>()
            .expose_provenance()
            .cast_signed();
        // SAFETY: cell はこの関数の終わりまで生き、ループの後で結び目を 0 に戻す。
        unsafe {
            let _previous = SetWindowLongPtrW(self.window, GWLP_USERDATA, address);
        }
        // 枠を消す計算（WM_NCCALCSIZE）をやり直させ、見せる。
        // SAFETY: 自分の窓の置き方を変えずに、枠の計算だけをやり直させる。
        unsafe {
            let _reframed: Result<(), _> = SetWindowPos(
                self.window,
                None,
                0,
                0,
                0,
                0,
                SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER,
            );
            let _was_visible = ShowWindow(self.window, SW_SHOW);
        }
        let mut message = MSG::default();
        // SAFETY: このスレッドのメッセージを、WM_QUIT（WM_DESTROY で積む）まで取り出して配る。
        while unsafe { GetMessageW(&raw mut message, None, 0, 0) }.as_bool() {
            // SAFETY: 取り出したばかりのメッセージを訳して配る。
            unsafe {
                let _translated = TranslateMessage(&raw const message);
                DispatchMessageW(&raw const message);
            }
        }
        // SAFETY: 窓はもう壊れているが、念のため結び目を外す（cell はこの後で落ちる）。
        unsafe {
            let _previous = SetWindowLongPtrW(self.window, GWLP_USERDATA, 0);
        }
        Ok(())
    }
}

/// 窓を作る前に、プロセスを per-monitor v2 の倍率の対応にする（Windows に拡大させず、物理画素で描く・WIN-09 の土台）。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の SetProcessDpiAwarenessContext を呼ぶ（ADR 0002 の 4）"
)]
fn declare_dpi_awareness() {
    // SAFETY: 値だけを渡す。既に決まっていれば失敗するが、そのときは既定の対応のまま続ける。
    let _declared: Result<(), _> =
        unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
}

/// 窓の類を登録する。2 度目の登録の失敗は無視する（同じ類）。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の RegisterClassExW / LoadCursorW を呼ぶ（ADR 0002 の 4）"
)]
fn register_class() -> Result<(), Win32Failure> {
    // SAFETY: 自分のモジュールのハンドルと、系統の共有の矢印を読むだけ。
    let (instance, arrow) = unsafe { (GetModuleHandleW(None)?, LoadCursorW(None, IDC_ARROW)?) };
    let class = WNDCLASSEXW {
        cbSize: u32::try_from(size_of::<WNDCLASSEXW>())?,
        lpfnWndProc: Some(procedure),
        hInstance: instance.into(),
        lpszClassName: CLASS,
        hCursor: arrow,
        // 中身の 2 度目の押しを WM_LBUTTONDBLCLK で受ける（ダブルクリックで編集・ADR 0026 の決定 1）。
        style: CS_DBLCLKS,
        ..WNDCLASSEXW::default()
    };
    // SAFETY: class はこの関数の局所で、登録の間だけ読まれる。
    let _registered: u16 = unsafe { RegisterClassExW(&raw const class) };
    Ok(())
}

/// 通常の窓の型で、題名 `title` の窓を作る（見せるのは `run`）。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の CreateWindowExW を呼ぶ（ADR 0002 の 4）"
)]
fn create(title: &str) -> Result<HWND, Win32Failure> {
    // SAFETY: 自分のモジュールのハンドルを読むだけ。
    let instance = unsafe { GetModuleHandleW(None) }?;
    let title = HSTRING::from(title);
    // SAFETY: 類は登録済み、題名はこの関数の間生きている。状態はまだ結ばない（run で結ぶ）。
    let created = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            CLASS,
            &title,
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            Some(instance.into()),
            None,
        )
    }?;
    Ok(created)
}

/// 窓の出る画面の倍率で、最初の窓の大きさ（`INITIAL_WIDTH` × `INITIAL_HEIGHT`）にする（作る前は画面が決まらない）。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の SetWindowPos を呼ぶ（ADR 0002 の 4）"
)]
fn fit_initial_size(window: HWND) {
    let scale = scale_of(window);
    let (Ok(width), Ok(height)) = (
        i32::try_from(scale.px(INITIAL_WIDTH)),
        i32::try_from(scale.px(INITIAL_HEIGHT)),
    ) else {
        return;
    };
    // SAFETY: 自分の窓の大きさだけを変える（まだ状態を結んでいない・見せていない）。
    let _sized: Result<(), _> = unsafe {
        SetWindowPos(
            window,
            None,
            0,
            0,
            width,
            height,
            SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
        )
    };
}

/// 窓のある画面の倍率（`GetDpiForWindow`）。読めなければ 100%。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の GetDpiForWindow を呼ぶ（ADR 0002 の 4）"
)]
fn scale_of(window: HWND) -> DisplayScale {
    // SAFETY: 自分の窓の dpi を読むだけ。失敗すれば 0 が返る。
    let dpi = unsafe { GetDpiForWindow(window) };
    if dpi == 0 {
        DisplayScale::hundred()
    } else {
        DisplayScale::of_dpi(dpi)
    }
}

/// 影（枠を 1 画素だけ中身へ広げる）と角丸を DWM に頼む（ADR 0013 の 1・WIN-06）。失敗しても続ける（古い Windows）。
#[expect(
    unsafe_code,
    reason = "RS-010: DWM の DwmExtendFrameIntoClientArea / DwmSetWindowAttribute を呼ぶ（ADR 0002 の 4）"
)]
fn decorate(window: HWND) {
    let margins = MARGINS {
        cxLeftWidth: 0,
        cxRightWidth: 0,
        cyTopHeight: 1,
        cyBottomHeight: 0,
    };
    let corner = DWMWCP_ROUND;
    let Ok(size) = u32::try_from(size_of_val(&corner)) else {
        return;
    };
    // SAFETY: margins と corner はこの関数の局所で、呼び出しの間だけ読まれる。
    unsafe {
        let _shadowed: Result<(), _> = DwmExtendFrameIntoClientArea(window, &raw const margins);
        let _rounded: Result<(), _> = DwmSetWindowAttribute(
            window,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            (&raw const corner).cast::<c_void>(),
            size,
        );
    }
}

/// 窓の手続き。枠を消す計算は状態が無くても行う。状態を借りられれば処理し、借りられない（入れ子）か扱わない種類なら
/// 既定の手続きへ渡す。
#[expect(
    unsafe_code,
    reason = "RS-010: 窓の手続きは Win32 から呼ばれ、窓に結んだポインタを辿る（ADR 0002 の 4）"
)]
unsafe extern "system" fn procedure(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_NCCALCSIZE && wparam.0 != 0 {
        return client_covers_window(window, lparam);
    }
    // 確定前の文字列はセルの中に自分で描くので、IME の組み立ての窓は出させない（ADR 0026 の決定 4）。
    if message == WM_IME_SETCONTEXT {
        // SAFETY: 届いたメッセージを、lparam の印を 1 つ落として既定の手続きへ渡す。
        return unsafe {
            DefWindowProcW(
                window,
                message,
                wparam,
                ime::without_composition_window(lparam),
            )
        };
    }
    // SAFETY: GWLP_USERDATA は run が結んだ状態か 0。状態は run の終わりまで生き、run はループの後で 0 に戻す。
    // 共有の参照だけを作り、書き換えは RefCell の借用を通す。
    let cell = unsafe {
        ptr::with_exposed_provenance::<StateCell>(
            GetWindowLongPtrW(window, GWLP_USERDATA).cast_unsigned(),
        )
        .as_ref()
    };
    let handled = cell.and_then(|cell| {
        let mut state = cell.try_borrow_mut().ok()?;
        dispatch(&mut state, window, (message, wparam, lparam))
    });
    // 倍率が変わったら、借用を手放してから OS の勧める矩形に置き直す（同期で届く WM_SIZE を状態が受け取れるように）。
    if message == WM_DPICHANGED {
        follow_suggested(window, lparam);
        return LRESULT(0);
    }
    // 状態を借りられない間（名前を付けて保存の画面などを出している間）の閉じる頼みは受けない。既定の手続きに渡すと、
    // 未保存を尋ねずに窓を壊す。
    if message == WM_CLOSE && cell.is_some_and(|cell| cell.try_borrow_mut().is_err()) {
        return LRESULT(0);
    }
    match handled {
        Some(result) => result,
        // SAFETY: 既定の手続きにそのまま渡す。借用は上で手放してある。
        None => unsafe { DefWindowProcW(window, message, wparam, lparam) },
    }
}

/// `WM_DPICHANGED`: OS の勧める矩形（新しい倍率で同じ見かけの大きさ・lparam）に窓を置き直す（WIN-09）。
#[expect(
    unsafe_code,
    reason = "RS-010: WM_DPICHANGED の RECT を読み、Win32 の SetWindowPos を呼ぶ（ADR 0002 の 4）"
)]
fn follow_suggested(window: HWND, lparam: LPARAM) {
    // SAFETY: WM_DPICHANGED の lparam は勧める矩形の RECT を指し、呼び出しの間だけ読んでよい。自分の窓を置き直すだけ。
    unsafe {
        let Some(suggested) =
            ptr::with_exposed_provenance::<RECT>(lparam.0.cast_unsigned()).as_ref()
        else {
            return;
        };
        let _placed: Result<(), _> = SetWindowPos(
            window,
            None,
            suggested.left,
            suggested.top,
            suggested.right.saturating_sub(suggested.left),
            suggested.bottom.saturating_sub(suggested.top),
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
}

/// `WM_NCCALCSIZE`: 中身を窓全体に広げて枠とタイトルバーを消す。最大化した窓は OS が枠の分だけ画面の外へ出すので、
/// その分（窓のある画面の倍率での枠の幅）を内へ寄せる（画面の外へはみ出さない）。
#[expect(
    unsafe_code,
    reason = "RS-010: WM_NCCALCSIZE の NCCALCSIZE_PARAMS を書き換える（ADR 0002 の 4）"
)]
fn client_covers_window(window: HWND, lparam: LPARAM) -> LRESULT {
    // SAFETY: wparam が真の WM_NCCALCSIZE の lparam は NCCALCSIZE_PARAMS を指し、呼び出しの間だけ書いてよい。
    unsafe {
        if IsZoomed(window).as_bool()
            && let Some(params) =
                ptr::with_exposed_provenance_mut::<NCCALCSIZE_PARAMS>(lparam.0.cast_unsigned())
                    .as_mut()
        {
            let dpi = GetDpiForWindow(window);
            let x = GetSystemMetricsForDpi(SM_CXFRAME, dpi)
                + GetSystemMetricsForDpi(SM_CXPADDEDBORDER, dpi);
            let y = GetSystemMetricsForDpi(SM_CYFRAME, dpi)
                + GetSystemMetricsForDpi(SM_CXPADDEDBORDER, dpi);
            let rect = &mut params.rgrc[0];
            rect.left += x;
            rect.right -= x;
            rect.top += y;
            rect.bottom -= y;
        }
    }
    LRESULT(0)
}

/// キーのメッセージ。`None` は既定の手続きへ渡す印。
fn keyboard(state: &mut WindowState<'_>, message: u32, wparam: WPARAM) -> Option<LRESULT> {
    match message {
        WM_KEYDOWN => {
            let key = u16::try_from(wparam.0).ok()?;
            let intent = key_intent(key, modifiers())?;
            state.intent(intent);
            Some(LRESULT(0))
        }
        // Alt+Enter（セルの中の改行）と、検索と置換の欄の Alt の字（`ALT_KEYS`）だけを受ける。ほかの Alt のキー（Alt+F4 など）は
        // OS に任せる。
        WM_SYSKEYDOWN if ALT_KEYS.iter().any(|&key| wparam.0 == usize::from(key)) => {
            let key = u16::try_from(wparam.0).ok()?;
            let intent = key_intent(key, modifiers())?;
            state.intent(intent);
            Some(LRESULT(0))
        }
        // 続く `WM_SYSCHAR` を OS に渡すと、メニューが無いので警告の音が鳴る。
        WM_SYSCHAR if ALT_CHARS.iter().any(|&unit| wparam.0 == usize::from(unit)) => {
            Some(LRESULT(0))
        }
        WM_CHAR => {
            let unit = u32::try_from(wparam.0).ok()?;
            let intent = char::from_u32(unit)
                .and_then(|character| typed_intent(character, key_down(VK_CONTROL.0)))?;
            state.intent(intent);
            Some(LRESULT(0))
        }
        _ => None,
    }
}

/// メッセージを状態へ。`None` は既定の手続きへ渡す印。
fn dispatch(
    state: &mut WindowState<'_>,
    window: HWND,
    (message, wparam, lparam): (u32, WPARAM, LPARAM),
) -> Option<LRESULT> {
    match message {
        WM_SIZE => {
            let (width, height) = packed_words(lparam);
            let placement = if wparam.0 == SIZE_MAXIMIZED {
                WindowPlacement::Maximized
            } else {
                WindowPlacement::Normal
            };
            state.resized(
                PixelSize::new(u32::from(width), u32::from(height)),
                placement,
            );
            Some(LRESULT(0))
        }
        WM_PAINT => {
            paint(state, window);
            Some(LRESULT(0))
        }
        // 全部を自分で描くので、背景は消さない（ちらつきを防ぐ）。
        WM_ERASEBKGND => Some(LRESULT(1)),
        WM_NCHITTEST => {
            let (x, y) = client_point(window, lparam)?;
            let code = hit_code(state.hit(x, y));
            Some(LRESULT(isize::try_from(code).ok()?))
        }
        WM_SETCURSOR => cursor_message(state, lparam),
        WM_NCMOUSEMOVE | WM_NCMOUSELEAVE | WM_NCLBUTTONDOWN | WM_NCLBUTTONUP
        | WM_NCLBUTTONDBLCLK => chrome_message(state, window, (message, wparam)),
        WM_LBUTTONDOWN | WM_LBUTTONDBLCLK | WM_LBUTTONUP | WM_MOUSEMOVE | WM_MOUSELEAVE
        | WM_CAPTURECHANGED | WM_MOUSEWHEEL | WM_MOUSEHWHEEL => {
            pointer_message(state, window, (message, wparam, lparam))
        }
        WM_KEYDOWN | WM_SYSKEYDOWN | WM_SYSCHAR | WM_CHAR => keyboard(state, message, wparam),
        WM_DPICHANGED => rescale(state, wparam),
        WM_IME_STARTCOMPOSITION | WM_IME_COMPOSITION | WM_IME_ENDCOMPOSITION => {
            ime_message(state, window, (message, lparam))
        }
        WM_TIMER if wparam.0 == TICK_TIMER => {
            state.tick();
            Some(LRESULT(0))
        }
        WM_WAKE => {
            state.refresh();
            Some(LRESULT(0))
        }
        // × のボタン（`SC_CLOSE`）・Alt+F4 も、既定の手続きを通ってここに来る。閉じてよければ既定の手続き（`DestroyWindow`）へ。
        WM_CLOSE => (!state.close()).then_some(LRESULT(0)),
        WM_DESTROY => {
            post_quit();
            Some(LRESULT(0))
        }
        _ => None,
    }
}

/// IME の組み立て（ADR 0026 の決定 4）。既定の手続きへは渡さない（渡すと IME の窓が組み立てを描き、確定が `WM_CHAR` でも届く）。
/// 始まり・組み立て・終わりを描く相手に渡し、候補の窓を編集中のセルの位置に置く。
fn ime_message(
    state: &mut WindowState<'_>,
    window: HWND,
    (message, lparam): (u32, LPARAM),
) -> Option<LRESULT> {
    match message {
        WM_IME_STARTCOMPOSITION => state.composition(None, Some(Preedit::new("", 0))),
        WM_IME_COMPOSITION => {
            let (result, preedit) = ime::read_composition(window, lparam);
            state.composition(result.as_deref(), Some(preedit));
        }
        WM_IME_ENDCOMPOSITION => state.composition(None, None),
        _ => return None,
    }
    if let Some(cell) = state.composition_area() {
        ime::place_windows(window, cell);
    }
    Some(LRESULT(0))
}

/// `WM_DPICHANGED`: wparam の下位語が新しい dpi（上位語と同じ）。字を測り直す。置き直しは `procedure` が借用を
/// 手放してから行う。
fn rescale(state: &mut WindowState<'_>, wparam: WPARAM) -> Option<LRESULT> {
    let dpi = u32::try_from(wparam.0 & 0xFFFF).ok()?;
    state.rescaled(DisplayScale::of_dpi(dpi));
    Some(LRESULT(0))
}

/// 窓の部品の上の指（`WM_NC*`）。ボタンは自分で押す・離すを受け、離したら最小化・最大化 / 元に戻す・閉じる。
/// タイトルバーと縁は既定の手続きに任せる（移動・リサイズ・ダブルクリックで最大化）。
fn chrome_message(
    state: &mut WindowState<'_>,
    window: HWND,
    (message, wparam): (u32, WPARAM),
) -> Option<LRESULT> {
    let button = u32::try_from(wparam.0).ok().and_then(button_of);
    match message {
        WM_NCMOUSEMOVE => {
            track_leave(window, true);
            state.set_hovered(button);
            None
        }
        WM_NCMOUSELEAVE => {
            state.set_hovered(None);
            None
        }
        WM_NCLBUTTONDOWN | WM_NCLBUTTONDBLCLK => {
            let pressed = button?;
            state.press_button(pressed);
            Some(LRESULT(0))
        }
        WM_NCLBUTTONUP => {
            let released = state.release_button(button)?;
            perform(window, released);
            Some(LRESULT(0))
        }
        _ => None,
    }
}

/// 窓のボタン `button` の動作。その場では行わず、システムの命令（`WM_SYSCOMMAND`）として積む。
/// 🔴 状態を借りたまま `ShowWindow` を呼ぶと、同期で届く `WM_SIZE` を状態が受け取れず、窓の大きさを古いまま覚えてしまう
/// （施主の実機で、ボタンで最大化すると重くなり反応しなくなった）。積めば借用を手放した後に、ダブルクリックと同じ経路で処理される。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の IsZoomed / PostMessageW を呼ぶ（ADR 0002 の 4）"
)]
fn perform(window: HWND, button: ChromeButton) {
    // SAFETY: 自分の窓の状態を読み、システムの命令を積むだけ。
    unsafe {
        let command = match button {
            ChromeButton::Minimize => SC_MINIMIZE,
            ChromeButton::Maximize if IsZoomed(window).as_bool() => SC_RESTORE,
            ChromeButton::Maximize => SC_MAXIMIZE,
            ChromeButton::Close => SC_CLOSE,
        };
        let _posted: Result<(), _> =
            PostMessageW(Some(window), WM_SYSCOMMAND, WPARAM(command), LPARAM(0));
    }
}

/// 中身の上の指。押すと取り込み、離すと外す。押したまま動かせばドラッグ。
fn pointer_message(
    state: &mut WindowState<'_>,
    window: HWND,
    (message, wparam, lparam): (u32, WPARAM, LPARAM),
) -> Option<LRESULT> {
    match message {
        WM_LBUTTONDOWN | WM_LBUTTONDBLCLK => {
            capture(window);
            let (x, y) = signed_words(lparam);
            let held = Modifiers::new(wparam.0 & MK_SHIFT != 0, wparam.0 & MK_CONTROL != 0, false);
            let intent = if message == WM_LBUTTONDBLCLK {
                double_press_intent(x, y, held)
            } else {
                press_intent(x, y, held)
            };
            if let Some(intent) = intent {
                state.press(intent);
            }
        }
        WM_LBUTTONUP => {
            release_capture();
            state.release();
        }
        WM_CAPTURECHANGED => state.release(),
        WM_MOUSEMOVE => {
            track_leave(window, false);
            state.moved(signed_words(lparam));
        }
        WM_MOUSELEAVE => state.left(),
        WM_MOUSEWHEEL => {
            if let Some(intent) = wheel_intent(wheel_delta(wparam), modifiers()) {
                state.intent(intent);
            }
        }
        WM_MOUSEHWHEEL => {
            if let Some(intent) = sideways_wheel_intent(wheel_delta(wparam)) {
                state.intent(intent);
            }
        }
        _ => return None,
    }
    Some(LRESULT(0))
}

/// `WM_SETCURSOR`: 中身の上で列の境目に載っていれば左右の矢印にする（Issue #64）。それ以外は既定の手続き
/// （類の矢印・縁の向き）に任せる。lparam の下位語が当たり判定の値。
fn cursor_message(state: &WindowState<'_>, lparam: LPARAM) -> Option<LRESULT> {
    let over_content = u32::try_from(lparam.0 & 0xFFFF).ok() == Some(HTCLIENT);
    if !over_content || state.pointer_shape() != PointerShape::ColumnResize {
        return None;
    }
    show_column_cursor();
    Some(LRESULT(1))
}

/// カーソルを、列の幅を変える左右の矢印にする（Issue #64）。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の LoadCursorW / SetCursor を呼ぶ（ADR 0002 の 4）"
)]
fn show_column_cursor() {
    // SAFETY: 系統の共有のカーソルを読み、今のカーソルにするだけ。
    unsafe {
        if let Ok(cursor) = LoadCursorW(None, IDC_SIZEWE) {
            let _previous = SetCursor(Some(cursor));
        }
    }
}

/// `WM_PAINT`: 描き直しの要求を受け取り（`BeginPaint` / `EndPaint`）、Direct2D で描く。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の BeginPaint / EndPaint を呼ぶ（ADR 0002 の 4）"
)]
fn paint(state: &mut WindowState<'_>, window: HWND) {
    let mut info = PAINTSTRUCT::default();
    // SAFETY: info はこの関数の局所。BeginPaint と EndPaint は対になる。描くのは Direct2D なので DC は使わない。
    unsafe {
        let _dc = BeginPaint(window, &raw mut info);
        state.paint();
        let _ended = EndPaint(window, &raw const info);
    }
}

/// 窓の中身の大きさ。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の GetClientRect を呼ぶ（ADR 0002 の 4）"
)]
fn client_size(window: HWND) -> PixelSize {
    let mut rect = RECT::default();
    // SAFETY: rect はこの関数の局所。
    let _measured: Result<(), _> = unsafe { GetClientRect(window, &raw mut rect) };
    let span = |from: i32, to: i32| u32::try_from(to.saturating_sub(from)).unwrap_or(0);
    PixelSize::new(span(rect.left, rect.right), span(rect.top, rect.bottom))
}

/// 修飾キー `key` が押されているか。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の GetKeyState を呼ぶ（ADR 0002 の 4）"
)]
fn key_down(key: u16) -> bool {
    // SAFETY: このスレッドのキーの状態を読むだけ。負なら押されている。
    unsafe { GetKeyState(i32::from(key)) < 0 }
}

/// いま押されている修飾キー。
fn modifiers() -> Modifiers {
    Modifiers::new(
        key_down(VK_SHIFT.0),
        key_down(VK_CONTROL.0),
        key_down(VK_MENU.0),
    )
}

/// 指が窓（`nonclient` なら窓の部品）から出たら知らせを受け取る。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の TrackMouseEvent を呼ぶ（ADR 0002 の 4）"
)]
fn track_leave(window: HWND, nonclient: bool) {
    let Ok(size) = u32::try_from(size_of::<TRACKMOUSEEVENT>()) else {
        return;
    };
    let flags = if nonclient {
        TME_LEAVE | TME_NONCLIENT
    } else {
        TME_LEAVE
    };
    let mut request = TRACKMOUSEEVENT {
        cbSize: size,
        dwFlags: flags,
        hwndTrack: window,
        dwHoverTime: 0,
    };
    // SAFETY: request はこの関数の局所で、cbSize を正しく入れてある。
    let _tracked: Result<(), _> = unsafe { TrackMouseEvent(&raw mut request) };
}

/// 押したまま窓の外へ引いても、指の動きと離しを受け取る（ドラッグ）。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の SetCapture を呼ぶ（ADR 0002 の 4）"
)]
fn capture(window: HWND) {
    // SAFETY: 自分の窓に指を取り込む。
    let _previous = unsafe { SetCapture(window) };
}

/// 取り込みを外す。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の ReleaseCapture を呼ぶ（ADR 0002 の 4）"
)]
fn release_capture() {
    // SAFETY: このスレッドの取り込みを外すだけ。
    let _released: Result<(), _> = unsafe { ReleaseCapture() };
}

/// このスレッドのループに `WM_QUIT` を積む。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の PostQuitMessage を呼ぶ（ADR 0002 の 4）"
)]
fn post_quit() {
    // SAFETY: 値だけを渡す。
    unsafe {
        PostQuitMessage(0);
    }
}

/// 画面の座標（lparam）を窓の中の座標へ（`WM_NCHITTEST` は画面の座標で届く）。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の ScreenToClient を呼ぶ（ADR 0002 の 4）"
)]
fn client_point(window: HWND, lparam: LPARAM) -> Option<(i32, i32)> {
    let (x, y) = signed_words(lparam);
    let mut point = POINT { x, y };
    // SAFETY: point はこの関数の局所。
    unsafe { ScreenToClient(window, &raw mut point) }
        .as_bool()
        .then_some((point.x, point.y))
}

/// ホイールの wparam の上位語（符号付きの刻み）。
fn wheel_delta(wparam: WPARAM) -> i16 {
    u16::try_from((wparam.0 >> 16_u32) & 0xFFFF).map_or(0, u16::cast_signed)
}

/// lparam の下位語と上位語（符号なし）。
fn packed_words(lparam: LPARAM) -> (u16, u16) {
    let bits = lparam.0.cast_unsigned();
    let low = u16::try_from(bits & 0xFFFF).unwrap_or(0);
    let high = u16::try_from((bits >> 16_u32) & 0xFFFF).unwrap_or(0);
    (low, high)
}

/// lparam の下位語と上位語（符号付き・座標）。窓の外は負になりうる。
fn signed_words(lparam: LPARAM) -> (i32, i32) {
    let (low, high) = packed_words(lparam);
    (i32::from(low.cast_signed()), i32::from(high.cast_signed()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key_intent::Modifiers;

    /// `key_intent` が Alt だけで意図を返す英字と Enter は、すべて `WM_SYSKEYDOWN` で受け（`ALT_KEYS`）、続く `WM_SYSCHAR`
    /// も受ける（`ALT_CHARS`・大文字と小文字）。受けるキーはどれも意図になる（GitHub Issue #199 の独立のレビューで、
    /// Alt+Z・Alt+K を `ALT_KEYS` に足し忘れて Windows で効かなかった）。
    #[test]
    fn every_alt_key_is_received() {
        let alt = Modifiers::new(false, false, true);
        let letters = (u16::from(b'A')..=u16::from(b'Z')).chain([VK_RETURN.0]);
        for key in letters {
            let bound = key_intent(key, alt).is_some();
            assert_eq!(bound, ALT_KEYS.contains(&key), "{key:#x}");
            if bound && key != VK_RETURN.0 {
                let upper = u8::try_from(key).unwrap_or(0);
                assert!(ALT_CHARS.contains(&upper), "{key:#x}");
                assert!(ALT_CHARS.contains(&upper.to_ascii_lowercase()), "{key:#x}");
            }
        }
    }
}
