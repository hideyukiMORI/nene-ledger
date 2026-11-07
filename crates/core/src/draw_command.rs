//! 描画命令。

use alloc::string::String;

use crate::color::Color;
use crate::pixel_point::PixelPoint;
use crate::pixel_rect::PixelRect;
use crate::text_role::TextRole;

/// 1 つの描画命令。UI は順に描くだけで、配置を判断しない（ARC-011）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrawCommand {
    /// 矩形を塗る。
    Fill {
        /// 塗る矩形。
        rect: PixelRect,
        /// 色。
        color: Color,
    },
    /// 文字を書く。`baseline` は最初の字の基準線の左端。`clip` の外は描かない。
    Text {
        /// 基準線の左端。
        baseline: PixelPoint,
        /// 書く文字列（1 行）。
        text: String,
        /// 色。
        color: Color,
        /// 切り取りの矩形。
        clip: PixelRect,
        /// どちらの字の大きさで書くか（ADR 0020）。
        role: TextRole,
    },
    /// 線を引く。`from` と `to` は端の画素で、UI は画素の中心を通して引く（縦と横の線がにじまない）。
    Line {
        /// 始まりの画素。
        from: PixelPoint,
        /// 終わりの画素。
        to: PixelPoint,
        /// 太さ（物理画素）。
        width: u32,
        /// 色。
        color: Color,
    },
    /// 字の一部の背景を塗る（検索で当たった字・GitHub Issue #189）。基準線の左端 `baseline` から `text` を `role` の字で
    /// 書いたときの、`text` の `start..end` バイトの字が占める横の範囲を、`band` の縦の範囲で塗る。`band` の外は塗らない。
    /// 字の幅は UI が字体で測る（比例の字体では半角 1・全角 2 の近似と合わないので）。範囲が字の境目に無ければ塗らない。
    TextSpan {
        /// 字を書く基準線の左端（同じ字を書く `Text` と同じ）。
        baseline: PixelPoint,
        /// 書く文字列（1 行）。
        text: String,
        /// 塗る字の始まり（`text` のバイト）。
        start: usize,
        /// 塗る字の終わり（`text` のバイト・含まない）。
        end: usize,
        /// 塗る縦の範囲と、切り取りの矩形。
        band: PixelRect,
        /// 色。
        color: Color,
        /// どちらの字の大きさで測るか。
        role: TextRole,
    },
}

impl DrawCommand {
    /// 右へ `dx`・下へ `dy` ずらした命令（中身の命令を窓の飾りの内側へ置くとき）。
    #[must_use]
    pub fn offset(self, dx: u32, dy: u32) -> Self {
        let shift = |point: PixelPoint| {
            PixelPoint::new(point.x().saturating_add(dx), point.y().saturating_add(dy))
        };
        let shift_rect = |rect: PixelRect| PixelRect::new(shift(rect.origin()), rect.size());
        match self {
            Self::Fill { rect, color } => Self::Fill {
                rect: shift_rect(rect),
                color,
            },
            Self::Text {
                baseline,
                text,
                color,
                clip,
                role,
            } => Self::Text {
                baseline: shift(baseline),
                text,
                color,
                clip: shift_rect(clip),
                role,
            },
            Self::Line {
                from,
                to,
                width,
                color,
            } => Self::Line {
                from: shift(from),
                to: shift(to),
                width,
                color,
            },
            Self::TextSpan {
                baseline,
                text,
                start,
                end,
                band,
                color,
                role,
            } => Self::TextSpan {
                baseline: shift(baseline),
                text,
                start,
                end,
                band: shift_rect(band),
                color,
                role,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pixel_size::PixelSize;

    #[test]
    fn every_command_shifts_by_the_offset() {
        let color = Color::new(1, 2, 3);
        let rect = PixelRect::new(PixelPoint::new(1, 2), PixelSize::new(3, 4));
        let moved = PixelRect::new(PixelPoint::new(11, 22), PixelSize::new(3, 4));
        assert_eq!(
            DrawCommand::Fill { rect, color }.offset(10, 20),
            DrawCommand::Fill { rect: moved, color }
        );
        assert_eq!(
            DrawCommand::Text {
                baseline: PixelPoint::new(5, 6),
                text: String::from("a"),
                color,
                clip: rect,
                role: TextRole::Content,
            }
            .offset(10, 20),
            DrawCommand::Text {
                baseline: PixelPoint::new(15, 26),
                text: String::from("a"),
                color,
                clip: moved,
                role: TextRole::Content,
            }
        );
        assert_eq!(
            DrawCommand::Line {
                from: PixelPoint::new(0, 0),
                to: PixelPoint::new(u32::MAX, 1),
                width: 2,
                color,
            }
            .offset(10, 20),
            DrawCommand::Line {
                from: PixelPoint::new(10, 20),
                to: PixelPoint::new(u32::MAX, 21),
                width: 2,
                color,
            }
        );
        let span = |baseline, band| DrawCommand::TextSpan {
            baseline,
            text: String::from("abc"),
            start: 1,
            end: 2,
            band,
            color,
            role: TextRole::Content,
        };
        assert_eq!(
            span(PixelPoint::new(5, 6), rect).offset(10, 20),
            span(PixelPoint::new(15, 26), moved)
        );
    }
}
