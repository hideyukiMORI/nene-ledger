//! Direct2D / DirectWrite でフレームの描画命令を窓に描き、字の寸法を測る（ADR 0002・NeNeDocket の `pen.rs` の写し）。
//!
//! 1 DIP = 1 物理画素（レンダーターゲットの dpi を 96 に固定）。画面の倍率は字の大きさと core の `px` で掛ける（Issue #58）。
//! 命令の順と意味は X11 版の `frame_surface` と同じ。

use nene_ledger_application::zoom_level::ZoomLevel;
use nene_ledger_core::color::Color;
use nene_ledger_core::display_scale::DisplayScale;
use nene_ledger_core::draw_command::DrawCommand;
use nene_ledger_core::frame::Frame;
use nene_ledger_core::glyph_metrics::GlyphMetrics;
use nene_ledger_core::pixel_point::PixelPoint;
use nene_ledger_core::pixel_rect::PixelRect;
use nene_ledger_core::pixel_size::PixelSize;
use nene_ledger_core::text_role::TextRole;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Direct2D::Common::{
    D2D_RECT_F, D2D_SIZE_U, D2D1_ALPHA_MODE_UNKNOWN, D2D1_COLOR_F, D2D1_PIXEL_FORMAT,
};
use windows::Win32::Graphics::Direct2D::{
    D2D1_ANTIALIAS_MODE_ALIASED, D2D1_DRAW_TEXT_OPTIONS_NONE, D2D1_FACTORY_TYPE_SINGLE_THREADED,
    D2D1_FEATURE_LEVEL_DEFAULT, D2D1_HWND_RENDER_TARGET_PROPERTIES, D2D1_PRESENT_OPTIONS_NONE,
    D2D1_RENDER_TARGET_PROPERTIES, D2D1_RENDER_TARGET_TYPE_DEFAULT, D2D1_RENDER_TARGET_USAGE_NONE,
    D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE, D2D1CreateFactory, ID2D1Factory, ID2D1HwndRenderTarget,
    ID2D1SolidColorBrush,
};
use windows::Win32::Graphics::DirectWrite::{
    DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL,
    DWRITE_FONT_WEIGHT_NORMAL, DWRITE_LINE_METRICS, DWRITE_TEXT_METRICS, DWriteCreateFactory,
    IDWriteFactory, IDWriteTextFormat, IDWriteTextLayout,
};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_UNKNOWN;
use windows::core::w;
use windows_numerics::Vector2;

use crate::pixels::to_pixels;
use crate::win32_failure::Win32Failure;

/// 字の家族。日本語の字を持つ Windows の UI の字体（字の無い字は DirectWrite の代わりの字体に任せる）。
/// 🔴 表示言語と字体の設定（ADR 0011）を入れるまでの仮。
const FONT_FAMILY: windows::core::PCWSTR = w!("Yu Gothic UI");
/// 地域名（代わりの字体の漢字の字形を日本のものにする）。🔴 表示言語（ADR 0011）を入れるまでの仮。
const LOCALE: windows::core::PCWSTR = w!("ja-jp");
/// 字の大きさ（画素・倍率 100% のとき）。窓の部品の字と、拡大 100% の中身の字（X11 版と同じ）。
const FONT_SIZE: f32 = 14.0;
/// 字の配置の最大の幅と高さ（1 行）。
const LAYOUT_LIMIT: f32 = 100_000.0;

/// Direct2D の窓のレンダーターゲットと DirectWrite の工場。
#[derive(Debug)]
pub(crate) struct Pen {
    target: ID2D1HwndRenderTarget,
    words: IDWriteFactory,
}

impl Pen {
    /// 窓 `window`（大きさ `size`）に結んだレンダーターゲットを作る。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D / DirectWrite の工場と窓のレンダーターゲットを作る（ADR 0002 の 4）"
    )]
    pub(crate) fn new(window: HWND, size: PixelSize) -> Result<Self, Win32Failure> {
        // SAFETY: 単一スレッドの工場。既定の生成の引数だけを渡す。
        let factory: ID2D1Factory =
            unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None) }?;
        let properties = D2D1_RENDER_TARGET_PROPERTIES {
            r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
            pixelFormat: D2D1_PIXEL_FORMAT {
                format: DXGI_FORMAT_UNKNOWN,
                alphaMode: D2D1_ALPHA_MODE_UNKNOWN,
            },
            dpiX: 96.0,
            dpiY: 96.0,
            usage: D2D1_RENDER_TARGET_USAGE_NONE,
            minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
        };
        let window_properties = D2D1_HWND_RENDER_TARGET_PROPERTIES {
            hwnd: window,
            pixelSize: size_u(size),
            presentOptions: D2D1_PRESENT_OPTIONS_NONE,
        };
        // SAFETY: 2 つの properties はこの関数の局所で、呼び出しの間だけ読まれる。窓はこのアプリのもの。
        let target = unsafe {
            factory.CreateHwndRenderTarget(&raw const properties, &raw const window_properties)
        }?;
        // SAFETY: 共有の DirectWrite の工場。
        let words: IDWriteFactory = unsafe { DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED) }?;
        Ok(Self { target, words })
    }

    /// 窓の大きさが変わった。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D の Resize を呼ぶ（ADR 0002 の 4）"
    )]
    pub(crate) fn resize(&self, size: PixelSize) -> Result<(), Win32Failure> {
        let size = size_u(size);
        // SAFETY: size はこの関数の局所で、呼び出しの間だけ読まれる。
        unsafe { self.target.Resize(&raw const size) }?;
        Ok(())
    }

    /// 画面の倍率 `scale` での、拡大 `zoom` の中身の字の寸法（拡大 100% なら窓の部品の字と同じ）。
    /// 半角の幅は `0` の送り幅（X11 版と同じ）。
    pub(crate) fn measure(
        &self,
        zoom: ZoomLevel,
        scale: DisplayScale,
    ) -> Result<GlyphMetrics, Win32Failure> {
        let format = self.format(content_size(zoom, scale))?;
        let (width, height, baseline) = self.line_metrics(&format, "0")?;
        Ok(GlyphMetrics::new(
            to_pixels(f64::from(width)),
            to_pixels(f64::from(height)),
            to_pixels(f64::from(baseline)),
        )
        .with_scale(scale))
    }

    /// フレームを描き切って窓に出す。字は画面の倍率 `scale` を掛け、中身の字はさらに拡大 `zoom` の大きさ（ADR 0020）。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D の BeginDraw / EndDraw を呼ぶ（ADR 0002 の 4）"
    )]
    pub(crate) fn paint(
        &self,
        frame: &Frame,
        zoom: ZoomLevel,
        scale: DisplayScale,
    ) -> Result<(), Win32Failure> {
        let chrome = self.format(content_size(ZoomLevel::hundred(), scale))?;
        let content = self.format(content_size(zoom, scale))?;
        // SAFETY: 描画は BeginDraw と EndDraw の間だけ。矩形は画素に揃え、字は灰色の AA。
        unsafe {
            self.target.BeginDraw();
            self.target.SetAntialiasMode(D2D1_ANTIALIAS_MODE_ALIASED);
            self.target
                .SetTextAntialiasMode(D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE);
        }
        let mut drawn = Ok(());
        for command in frame.commands() {
            drawn = self.draw(command, (&chrome, &content));
            if drawn.is_err() {
                break;
            }
        }
        // SAFETY: 上の BeginDraw と対になる。
        unsafe { self.target.EndDraw(None, None) }?;
        drawn
    }

    /// 1 つの命令を描く。
    fn draw(
        &self,
        command: &DrawCommand,
        (chrome, content): (&IDWriteTextFormat, &IDWriteTextFormat),
    ) -> Result<(), Win32Failure> {
        match command {
            DrawCommand::Fill { rect, color } => self.fill(*rect, *color),
            DrawCommand::Text {
                baseline,
                text,
                color,
                clip,
                role,
            } => {
                let format = match role {
                    TextRole::Chrome => chrome,
                    TextRole::Content => content,
                };
                self.text(format, (*baseline, text), (*color, *clip))
            }
            DrawCommand::Line {
                from,
                to,
                width,
                color,
            } => self.line((*from, *to), *width, *color),
            DrawCommand::TextSpan {
                baseline,
                text,
                start,
                end,
                band,
                color,
                role,
            } => {
                let format = match role {
                    TextRole::Chrome => chrome,
                    TextRole::Content => content,
                };
                self.text_span(format, (*baseline, text, *start..*end), (*color, *band))
            }
        }
    }

    /// 基準線の左端 `baseline` から書く `text` の、`span` のバイトの字の背景を、`band` の縦の範囲で塗る（`band` の外は
    /// 塗らない・GitHub Issue #189）。字の幅は DirectWrite の配置で測る。範囲が字の境目に無ければ塗らない。
    fn text_span(
        &self,
        format: &IDWriteTextFormat,
        (baseline, text, span): (PixelPoint, &str, core::ops::Range<usize>),
        (color, band): (Color, PixelRect),
    ) -> Result<(), Win32Failure> {
        let (Some(head), Some(through)) = (text.get(..span.start), text.get(..span.end)) else {
            return Ok(());
        };
        let width = |part: &str| -> Result<f32, Win32Failure> {
            if part.is_empty() {
                return Ok(0.0);
            }
            Ok(self.line_metrics(format, part)?.0)
        };
        let origin = to_float(baseline.x());
        let (left, right) = (
            (origin + width(head)?).round(),
            (origin + width(through)?).round(),
        );
        let bounds = rect_f(band);
        let area = D2D_RECT_F {
            left: left.max(bounds.left),
            top: bounds.top,
            right: right.min(bounds.right),
            bottom: bounds.bottom,
        };
        if area.right <= area.left {
            return Ok(());
        }
        self.fill_area(&area, color)
    }

    /// 1 色の筆。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D の CreateSolidColorBrush を呼ぶ（ADR 0002 の 4）"
    )]
    fn brush(&self, color: Color) -> Result<ID2D1SolidColorBrush, Win32Failure> {
        let color = D2D1_COLOR_F {
            r: f32::from(color.red()) / 255.0,
            g: f32::from(color.green()) / 255.0,
            b: f32::from(color.blue()) / 255.0,
            a: 1.0,
        };
        // SAFETY: color はこの関数の局所で、呼び出しの間だけ読まれる。
        let solid = unsafe { self.target.CreateSolidColorBrush(&raw const color, None) }?;
        Ok(solid)
    }

    /// 矩形を塗る。
    fn fill(&self, rect: PixelRect, color: Color) -> Result<(), Win32Failure> {
        self.fill_area(&rect_f(rect), color)
    }

    /// 浮動小数の矩形 `area` を塗る。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D の FillRectangle を呼ぶ（ADR 0002 の 4）"
    )]
    fn fill_area(&self, area: &D2D_RECT_F, color: Color) -> Result<(), Win32Failure> {
        let brush = self.brush(color)?;
        // SAFETY: 描画は paint の中で呼ばれる。area は呼び出しの間だけ読まれる。
        unsafe {
            self.target.FillRectangle(area, &brush);
        }
        Ok(())
    }

    /// 線を引く。端の画素の中心を通す（縦と横の線がにじまない・X11 版と同じ）。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D の DrawLine を呼ぶ（ADR 0002 の 4）"
    )]
    fn line(
        &self,
        (from, to): (PixelPoint, PixelPoint),
        width: u32,
        color: Color,
    ) -> Result<(), Win32Failure> {
        let brush = self.brush(color)?;
        let center = |point: PixelPoint| Vector2 {
            X: to_float(point.x()) + 0.5,
            Y: to_float(point.y()) + 0.5,
        };
        // SAFETY: 描画は paint の中で呼ばれる。点はこの関数の局所。
        unsafe {
            self.target
                .DrawLine(center(from), center(to), &brush, to_float(width), None);
        }
        Ok(())
    }

    /// 1 行の字を、基準線の左端 `baseline` から書く。`clip` の外は描かない。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D の PushAxisAlignedClip / DrawTextLayout / PopAxisAlignedClip を呼ぶ（ADR 0002 の 4）"
    )]
    fn text(
        &self,
        format: &IDWriteTextFormat,
        (baseline, text): (PixelPoint, &str),
        (color, clip): (Color, PixelRect),
    ) -> Result<(), Win32Failure> {
        if text.is_empty() {
            return Ok(());
        }
        let brush = self.brush(color)?;
        let layout = self.layout(format, text)?;
        let (_, _, ascent) = metrics_of(&layout)?;
        let origin = Vector2 {
            X: to_float(baseline.x()),
            Y: to_float(baseline.y()) - ascent,
        };
        let area = rect_f(clip);
        // SAFETY: 描画は paint の中で呼ばれる。切り抜きは Push と Pop が対になる。配置はこの関数の間生きている。
        unsafe {
            self.target
                .PushAxisAlignedClip(&raw const area, D2D1_ANTIALIAS_MODE_ALIASED);
            self.target
                .DrawTextLayout(origin, &layout, &brush, D2D1_DRAW_TEXT_OPTIONS_NONE);
            self.target.PopAxisAlignedClip();
        }
        Ok(())
    }

    /// 字の形式（家族と大きさ）。
    #[expect(
        unsafe_code,
        reason = "RS-010: DirectWrite の CreateTextFormat を呼ぶ（ADR 0002 の 4）"
    )]
    fn format(&self, size: f32) -> Result<IDWriteTextFormat, Win32Failure> {
        // SAFETY: 家族名と地域名は静的な文字列。
        let format = unsafe {
            self.words.CreateTextFormat(
                FONT_FAMILY,
                None,
                DWRITE_FONT_WEIGHT_NORMAL,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                size,
                LOCALE,
            )
        }?;
        Ok(format)
    }

    /// 1 行の字の配置。
    #[expect(
        unsafe_code,
        reason = "RS-010: DirectWrite の CreateTextLayout を呼ぶ（ADR 0002 の 4）"
    )]
    fn layout(
        &self,
        format: &IDWriteTextFormat,
        text: &str,
    ) -> Result<IDWriteTextLayout, Win32Failure> {
        let wide: Vec<u16> = text.encode_utf16().collect();
        // SAFETY: wide はこの関数の局所で、呼び出しの間だけ読まれる。
        let layout = unsafe {
            self.words
                .CreateTextLayout(&wide, format, LAYOUT_LIMIT, LAYOUT_LIMIT)
        }?;
        Ok(layout)
    }

    /// 1 行の字の幅・行の高さ・基準線までの高さ。
    fn line_metrics(
        &self,
        format: &IDWriteTextFormat,
        text: &str,
    ) -> Result<(f32, f32, f32), Win32Failure> {
        metrics_of(&self.layout(format, text)?)
    }
}

/// 配置の 1 行目の幅・高さ・基準線までの高さ。
#[expect(
    unsafe_code,
    reason = "RS-010: DirectWrite の GetMetrics / GetLineMetrics を呼ぶ（ADR 0002 の 4）"
)]
fn metrics_of(layout: &IDWriteTextLayout) -> Result<(f32, f32, f32), Win32Failure> {
    let mut metrics = DWRITE_TEXT_METRICS::default();
    let mut line = [DWRITE_LINE_METRICS::default()];
    let mut lines = 0_u32;
    // SAFETY: 出力はこの関数の局所。1 行分の受け皿を渡す。
    unsafe {
        layout.GetMetrics(&raw mut metrics)?;
        layout.GetLineMetrics(Some(&mut line), &raw mut lines)?;
    }
    let first = line.first().copied().unwrap_or_default();
    Ok((
        metrics.widthIncludingTrailingWhitespace,
        first.height,
        first.baseline,
    ))
}

/// 画面の倍率 `scale` での、拡大 `zoom` の中身の字の大きさ（窓の部品の字は拡大 100%）。
fn content_size(zoom: ZoomLevel, scale: DisplayScale) -> f32 {
    FONT_SIZE * to_float(zoom.percent()) / 100.0 * to_float(scale.percent()) / 100.0
}

/// 画素の数を浮動小数へ（大きな数は丸める）。
fn to_float(value: u32) -> f32 {
    f32::from(u16::try_from(value).unwrap_or(u16::MAX))
}

/// 整数座標の矩形を Direct2D の矩形へ。
fn rect_f(rect: PixelRect) -> D2D_RECT_F {
    let (x, y) = (to_float(rect.origin().x()), to_float(rect.origin().y()));
    D2D_RECT_F {
        left: x,
        top: y,
        right: x + to_float(rect.size().width()),
        bottom: y + to_float(rect.size().height()),
    }
}

/// 大きさを Direct2D の大きさへ。
fn size_u(size: PixelSize) -> D2D_SIZE_U {
    D2D_SIZE_U {
        width: size.width(),
        height: size.height(),
    }
}
