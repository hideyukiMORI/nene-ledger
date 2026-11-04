//! フレームを cairo の画面外の面に描く。字の寸法を測る。

use cairo::{Context, FontSlant, FontWeight, Format, ImageSurface, LineCap};
use nene_ledger_application::zoom_level::ZoomLevel;
use nene_ledger_core::color::Color;
use nene_ledger_core::draw_command::DrawCommand;
use nene_ledger_core::frame::Frame;
use nene_ledger_core::glyph_metrics::GlyphMetrics;
use nene_ledger_core::pixel_rect::PixelRect;
use nene_ledger_core::text_role::TextRole;

use crate::pixels::to_pixels;
use crate::ui_failure::UiFailure;

/// 字体の名前。🔴 設定（表示言語・字体）を入れるまでの仮。日本語の字を持つ字体を名指しする
/// （cairo の簡易な文字の API は字ごとの代わりの字体を探さない）。
const FONT_FAMILY: &str = "Noto Sans CJK JP";

/// 字の大きさ（物理画素）。窓の部品の字と、100% の中身の字。X11 の画面の倍率は当面 100%（Xft.dpi などを読むのは別の Issue・Issue #58）。
const FONT_SIZE: f64 = 14.0;

/// 拡大 `zoom` の中身の字の大きさ（物理画素）。
fn content_size(zoom: ZoomLevel) -> f64 {
    FONT_SIZE * f64::from(zoom.percent()) / 100.0_f64
}

/// 画面外の面に、フレームの命令を順に描く。面は 24 ビットの色・32 ビットの画素（X の `ZPixmap` と同じ並び）。
/// 文字は種類ごとの大きさで書く（窓の部品は `FONT_SIZE`、中身は拡大 `zoom` に従う・ADR 0020）。
pub(crate) fn paint(frame: &Frame, zoom: ZoomLevel) -> Result<ImageSurface, UiFailure> {
    let width = i32::try_from(frame.size().width())?;
    let height = i32::try_from(frame.size().height())?;
    let surface = ImageSurface::create(Format::Rgb24, width.max(1_i32), height.max(1_i32))?;
    {
        let context = Context::new(&surface)?;
        set_font(&context, FONT_SIZE);
        for command in frame.commands() {
            draw(&context, command, zoom)?;
        }
    }
    surface.flush();
    Ok(surface)
}

/// 字体から、拡大 `zoom` の中身の字の寸法を測る（100% なら窓の部品の字と同じ）。半角の幅は `0` の送り幅。
pub(crate) fn measure(zoom: ZoomLevel) -> Result<GlyphMetrics, UiFailure> {
    let surface = ImageSurface::create(Format::Rgb24, 1_i32, 1_i32)?;
    let context = Context::new(&surface)?;
    set_font(&context, content_size(zoom));
    let font = context.font_extents()?;
    let zero = context.text_extents("0")?;
    Ok(GlyphMetrics::new(
        to_pixels(zero.x_advance()),
        to_pixels(font.height()),
        to_pixels(font.ascent()),
    ))
}

fn set_font(context: &Context, size: f64) {
    context.select_font_face(FONT_FAMILY, FontSlant::Normal, FontWeight::Normal);
    context.set_font_size(size);
}

fn draw(context: &Context, command: &DrawCommand, zoom: ZoomLevel) -> Result<(), UiFailure> {
    match command {
        DrawCommand::Fill { rect, color } => {
            set_color(context, *color);
            add_rect(context, *rect);
            context.fill()?;
        }
        DrawCommand::Text {
            baseline,
            text,
            color,
            clip,
            role,
        } => {
            context.save()?;
            context.set_font_size(match role {
                TextRole::Chrome => FONT_SIZE,
                TextRole::Content => content_size(zoom),
            });
            add_rect(context, *clip);
            context.clip();
            set_color(context, *color);
            context.move_to(f64::from(baseline.x()), f64::from(baseline.y()));
            context.show_text(text)?;
            context.restore()?;
        }
        DrawCommand::Line {
            from,
            to,
            width,
            color,
        } => {
            // 画素の中心を通し、端を四角く伸ばして、端の画素まで塗る。
            let center = |value: u32| f64::from(value) + 0.5_f64;
            set_color(context, *color);
            context.set_line_width(f64::from(*width));
            context.set_line_cap(LineCap::Square);
            context.move_to(center(from.x()), center(from.y()));
            context.line_to(center(to.x()), center(to.y()));
            context.stroke()?;
        }
    }
    Ok(())
}

fn set_color(context: &Context, color: Color) {
    let channel = |value: u8| f64::from(value) / 255.0_f64;
    context.set_source_rgb(
        channel(color.red()),
        channel(color.green()),
        channel(color.blue()),
    );
}

fn add_rect(context: &Context, rect: PixelRect) {
    context.rectangle(
        f64::from(rect.origin().x()),
        f64::from(rect.origin().y()),
        f64::from(rect.size().width()),
        f64::from(rect.size().height()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use nene_ledger_core::pixel_point::PixelPoint;
    use nene_ledger_core::pixel_size::PixelSize;

    #[test]
    fn a_filled_rect_lands_on_its_pixels_in_the_x_byte_order() {
        let frame = Frame::new(
            PixelSize::new(4, 2),
            vec![DrawCommand::Fill {
                rect: PixelRect::new(PixelPoint::new(1, 0), PixelSize::new(2, 1)),
                color: Color::new(0x12, 0x34, 0x56),
            }],
        );
        let surface = paint(&frame, ZoomLevel::hundred()).unwrap();
        assert_eq!(surface.stride(), 16_i32);
        surface
            .with_data(|bytes| {
                // RGB24 はリトルエンディアンで B・G・R・未使用の順。塗っていない所は 0。
                assert_eq!(&bytes[0..4], &[0_u8; 4]);
                assert_eq!(&bytes[4..7], &[0x56, 0x34, 0x12]);
                assert_eq!(&bytes[8..11], &[0x56, 0x34, 0x12]);
                assert_eq!(&bytes[12..15], &[0_u8; 3]);
                assert_eq!(&bytes[16..19], &[0_u8; 3]);
            })
            .unwrap();
    }

    #[test]
    fn a_line_covers_its_end_pixels_without_bleeding() {
        let frame = Frame::new(
            PixelSize::new(4, 3),
            vec![DrawCommand::Line {
                from: PixelPoint::new(1, 1),
                to: PixelPoint::new(2, 1),
                width: 1,
                color: Color::new(0xFF, 0xFF, 0xFF),
            }],
        );
        let surface = paint(&frame, ZoomLevel::hundred()).unwrap();
        surface
            .with_data(|bytes| {
                let pixel = |x: usize, y: usize| bytes[y * 16 + x * 4];
                assert_eq!(
                    (pixel(0, 1), pixel(1, 1), pixel(2, 1), pixel(3, 1)),
                    (0, 0xFF, 0xFF, 0)
                );
                assert_eq!((pixel(1, 0), pixel(1, 2)), (0, 0));
            })
            .unwrap();
    }

    #[test]
    fn metrics_are_positive_and_consistent() {
        let metrics = measure(ZoomLevel::hundred()).unwrap();
        let larger =
            measure(ZoomLevel::hundred().stepped(nene_ledger_application::zoom_step::ZoomStep::In))
                .unwrap();
        assert!(larger.line_height() > metrics.line_height());
        assert!(metrics.half_width() > 0);
        assert!(metrics.line_height() >= metrics.ascent());
        assert!(metrics.ascent() > 0);
    }
}
