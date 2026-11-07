//! 配色の元の 3 色（背景・前景・強調）と明暗から、画面の色の役割をすべて整数の補間だけで導く（ADR 0012）。
//!
//! sRGB の値のまま線形に混ぜる（ガンマは扱わない）。百分率の表は明暗ごとに 1 つ。決定的で、浮動小数も乱数も使わない。
//! 補間した色がコントラストの基準に届かなければ、届くまで 1% ずつ寄せる（字と線は明暗の端（ライトは黒・ダークは白）へ・
//! 選択は背景へ）。前景そのものが控えめな配色（solarized など）では、字が前景より少し濃くなる。
//! 🔴 百分率は仮。施主が見て、この表の数字だけを変える。

use crate::appearance::Appearance;
use crate::color::Color;
use crate::contrast::meets;
use crate::palette::Palette;
use crate::palette_role::PaletteRole;
use crate::theme_seed::ThemeSeed;

/// 字と背景の基準（4.5・十分の一の単位）。
pub const TEXT_RATIO: u64 = 45;
/// 線と枠と背景の基準（3.0）。
pub const LINE_RATIO: u64 = 30;

/// 明暗ごとの百分率（背景から前景・強調・白へ、または前景から背景へ寄せる量）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Weights {
    /// 罫線: 背景 → 前景。
    grid: u8,
    /// 面: 背景 → 前景。
    gutter: u8,
    /// 補助の字: 前景 → 背景。
    muted: u8,
    /// タイトルバー: 背景 → 前景。
    title_bar: u8,
    /// 指の乗ったボタン: 背景 → 前景。
    hover: u8,
    /// 窓の縁: 背景 → 前景。
    border: u8,
    /// 選択: 背景 → 強調。
    selection: u8,
    /// タイトルバーの区切り: 背景 → 前景。
    chrome_rule: u8,
    /// 帯: 背景 → 強調。
    band: u8,
    /// スクロールバーのつまみ: 面 → 前景。
    thumb: u8,
    /// 当たったセル: 背景 → 印の色。
    found: u8,
    /// 当たった字: 背景 → 印の色。
    found_text: u8,
}

const LIGHT: Weights = Weights {
    grid: 15,
    gutter: 4,
    muted: 30,
    title_bar: 8,
    hover: 20,
    border: 45,
    selection: 18,
    chrome_rule: 22,
    band: 6,
    thumb: 45,
    found: 18,
    found_text: 70,
};

const DARK: Weights = Weights {
    grid: 18,
    gutter: 6,
    muted: 30,
    title_bar: 10,
    hover: 22,
    border: 40,
    selection: 30,
    chrome_rule: 24,
    band: 10,
    thumb: 45,
    found: 18,
    found_text: 85,
};

/// 閉じるボタンに指を乗せた背景（Windows の閉じるの赤。配色に依らない）と、その上の印。
const CLOSE_RED: Color = Color::new(0xC4, 0x2B, 0x1C);
const WHITE: Color = Color::new(0xFF, 0xFF, 0xFF);
const BLACK: Color = Color::new(0x00, 0x00, 0x00);
/// 検索で当たった所の印の色（蛍光ペンの黄。配色の強調の色に依らず、選択の色と分ける・GitHub Issue #189）。
const MARKER: Color = Color::new(0xFF, 0xC4, 0x00);

/// 配色の元 `seed` の画面の色。
#[must_use]
pub fn derive(seed: &ThemeSeed) -> Palette {
    let (background, foreground) = (seed.background(), seed.foreground());
    let weights = match seed.appearance() {
        Appearance::Light => LIGHT,
        Appearance::Dark => DARK,
    };
    let toward_fg = |percent: u8| mix(background, foreground, percent);
    let extreme = match seed.appearance() {
        Appearance::Light => BLACK,
        Appearance::Dark => WHITE,
    };
    let gutter = toward_fg(weights.gutter);
    let title_bar = toward_fg(weights.title_bar);
    let (selection, band) = tints(seed, weights, gutter);
    // 補助の字は、置かれるすべての面の上で読めるまで明暗の端へ寄せる。
    let muted = ensure(
        mix(foreground, background, weights.muted),
        &[background, gutter, title_bar, selection],
        extreme,
        TEXT_RATIO,
    );
    let text = ensure(
        foreground,
        &[background, gutter, title_bar, selection, band],
        extreme,
        TEXT_RATIO,
    );
    let border = ensure(
        toward_fg(weights.border),
        &[background, gutter],
        extreme,
        LINE_RATIO,
    );
    let (cursor, thumb) = marks_on_lines(seed, weights, (gutter, selection), extreme);
    let (found, found_text, found_ink) =
        marks(seed, weights, text, &[background, gutter, selection, band]);
    Palette::from_roles(|role| match role {
        PaletteRole::Background => background,
        PaletteRole::Text | PaletteRole::TitleText => text,
        PaletteRole::Grid => toward_fg(weights.grid),
        PaletteRole::Gutter => gutter,
        PaletteRole::GutterText => muted,
        PaletteRole::TitleBar => title_bar,
        PaletteRole::ButtonHover => toward_fg(weights.hover),
        PaletteRole::CloseHover => CLOSE_RED,
        PaletteRole::CloseHoverText => WHITE,
        PaletteRole::Border => border,
        PaletteRole::Selection => selection,
        PaletteRole::Cursor => cursor,
        PaletteRole::ChromeRule => toward_fg(weights.chrome_rule),
        PaletteRole::Band => band,
        PaletteRole::ScrollThumb => thumb,
        PaletteRole::Found => found,
        PaletteRole::FoundText => found_text,
        PaletteRole::FoundInk => found_ink,
    })
}

/// カーソルの色とスクロールバーのつまみの色。どちらも線の基準に届くまで明暗の端 `extreme` へ寄せる。
fn marks_on_lines(
    seed: &ThemeSeed,
    weights: Weights,
    (gutter, selection): (Color, Color),
    extreme: Color,
) -> (Color, Color) {
    let (background, foreground) = (seed.background(), seed.foreground());
    // カーソルの色は、スクロールバーの溝（面）の上で選んだ範囲の印にも使う。
    let cursor = ensure(
        seed.accent(),
        &[background, selection, gutter],
        extreme,
        LINE_RATIO,
    );
    // つまみは溝（面）と、溝の外の背景の両方から見分けられるまで明暗の端へ寄せる。
    let thumb = ensure(
        mix(gutter, foreground, weights.thumb),
        &[gutter, background],
        extreme,
        LINE_RATIO,
    );
    (cursor, thumb)
}

/// 検索で当たったセルの背景・当たった字の背景・その上の字（GitHub Issue #189）。セルは背景から印の色へ少し寄せ、字 `text`
/// が読めるまで背景へ戻す。当たった字は印の色へ強く寄せ（蛍光ペン）、その上の字は読めるまで黒へ寄せる（暗い配色でも
/// 明るい黄の上に濃い字・Vim の検索の強調と同じ）。どちらの背景も、ほかの面 `surfaces`（背景・面・選択・帯）と互いに違う色。
fn marks(
    seed: &ThemeSeed,
    weights: Weights,
    text: Color,
    surfaces: &[Color],
) -> (Color, Color, Color) {
    let background = seed.background();
    let found = distinct(
        ensure(
            mix(background, MARKER, weights.found),
            &[text],
            background,
            TEXT_RATIO,
        ),
        surfaces,
        MARKER,
    );
    let mut others = surfaces.to_vec();
    others.push(found);
    let found_text = distinct(mix(background, MARKER, weights.found_text), &others, MARKER);
    let ink = match seed.appearance() {
        Appearance::Light => text,
        Appearance::Dark => background,
    };
    let found_ink = ensure(ink, &[found_text], BLACK, TEXT_RATIO);
    (found, found_text, found_ink)
}

/// 強調の色を混ぜた面の色（選択と帯）。選択は字が読めるまで背景へ戻す。帯は選択より薄く、背景と面 `gutter` のどちらとも
/// 違う色にする。
fn tints(seed: &ThemeSeed, weights: Weights, gutter: Color) -> (Color, Color) {
    let (background, accent) = (seed.background(), seed.accent());
    let selection = ensure(
        mix(background, accent, weights.selection),
        &[seed.foreground()],
        background,
        TEXT_RATIO,
    );
    let band = distinct(
        mix(background, accent, weights.band),
        &[background, gutter, selection],
        accent,
    );
    (selection, band)
}

/// `color` を、`against` のすべての色との比が `ratio_tenths` に届くまで、`toward` へ 1% ずつ寄せる。
/// 100% まで寄せても届かなければ `toward` そのもの（テストが全配色で届くことを確かめる）。
fn ensure(color: Color, against: &[Color], toward: Color, ratio_tenths: u64) -> Color {
    (0..=100_u8)
        .map(|percent| mix(color, toward, percent))
        .find(|candidate| {
            against
                .iter()
                .all(|&other| meets(*candidate, other, ratio_tenths))
        })
        .unwrap_or(toward)
}

/// `color` が `others` のどれとも同じなら、違う色になるまで `toward` へ 1% ずつ寄せる。
fn distinct(color: Color, others: &[Color], toward: Color) -> Color {
    (0..=100_u8)
        .map(|percent| mix(color, toward, percent))
        .find(|candidate| !others.contains(candidate))
        .unwrap_or(toward)
}

/// `from` から `to` へ `percent` だけ寄せる（0 で `from`、100 で `to`）。
fn mix(from: Color, to: Color, percent: u8) -> Color {
    Color::new(
        channel(from.red(), to.red(), percent),
        channel(from.green(), to.green(), percent),
        channel(from.blue(), to.blue(), percent),
    )
}

fn channel(from: u8, to: u8, percent: u8) -> u8 {
    let percent = u32::from(percent.min(100));
    let mixed = (u32::from(from) * (100 - percent) + u32::from(to) * percent + 50) / 100;
    u8::try_from(mixed).unwrap_or(u8::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme_catalog::{SEEDS, named};

    #[test]
    fn mix_is_linear_and_clamped() {
        let black = Color::new(0, 0, 0);
        assert_eq!(mix(black, WHITE, 0), black);
        assert_eq!(mix(black, WHITE, 100), WHITE);
        assert_eq!(mix(black, WHITE, 50), Color::new(128, 128, 128));
        assert_eq!(mix(black, WHITE, 200), WHITE);
    }

    #[test]
    fn ensuring_moves_only_as_far_as_needed() {
        let (white, black) = (WHITE, BLACK);
        // 白の上の #777777 は 4.48 で届かないので、黒へ 1% 寄せる。
        let grey = Color::new(0x77, 0x77, 0x77);
        assert_eq!(
            ensure(grey, &[white], black, TEXT_RATIO),
            Color::new(0x76, 0x76, 0x76)
        );
        assert_eq!(ensure(black, &[white], black, TEXT_RATIO), black);
        // 届かないまま寄せきったら寄せ先そのもの。
        assert_eq!(ensure(white, &[white], white, TEXT_RATIO), white);
        assert_eq!(
            distinct(white, &[white], black),
            Color::new(0xFC, 0xFC, 0xFC)
        );
        assert_eq!(distinct(white, &[white], white), white);
    }

    #[test]
    fn every_builtin_theme_is_readable() {
        for seed in &SEEDS {
            let palette = derive(seed);
            let name = seed.name();
            let surfaces = [
                palette.background(),
                palette.gutter(),
                palette.title_bar(),
                palette.selection(),
            ];
            for surface in surfaces {
                assert!(
                    meets(palette.text(), surface, TEXT_RATIO),
                    "{name}: 字 {:?} / {surface:?} = {}",
                    palette.text(),
                    crate::contrast::ratio_hundredths(palette.text(), surface)
                );
                assert!(
                    meets(palette.gutter_text(), surface, TEXT_RATIO),
                    "{name}: 補助の字"
                );
            }
        }
    }

    #[test]
    fn every_builtin_theme_has_visible_lines_and_readable_chrome() {
        for seed in &SEEDS {
            let palette = derive(seed);
            let name = seed.name();
            assert!(
                meets(palette.text(), palette.band(), TEXT_RATIO),
                "{name}: 帯の上の字"
            );
            assert!(
                meets(palette.title_text(), palette.title_bar(), TEXT_RATIO),
                "{name}: タイトル"
            );
            assert!(
                meets(
                    palette.close_hover_text(),
                    palette.close_hover(),
                    TEXT_RATIO
                ),
                "{name}: 閉じる"
            );
            assert!(
                meets(palette.cursor(), palette.background(), LINE_RATIO),
                "{name}: カーソル"
            );
            assert!(
                meets(palette.cursor(), palette.selection(), LINE_RATIO),
                "{name}: 選択の上のカーソル"
            );
            assert!(
                meets(palette.border(), palette.background(), LINE_RATIO),
                "{name}: 縁"
            );
        }
    }

    #[test]
    fn every_builtin_theme_has_visible_scrollbars() {
        for seed in &SEEDS {
            let palette = derive(seed);
            let name = seed.name();
            let (track, thumb) = (palette.gutter(), palette.scroll_thumb());
            assert!(meets(thumb, track, LINE_RATIO), "{name}: 溝の上のつまみ");
            assert!(
                meets(thumb, palette.background(), LINE_RATIO),
                "{name}: 背景の隣のつまみ"
            );
            assert!(
                meets(palette.gutter_text(), track, TEXT_RATIO),
                "{name}: 溝の上の指の乗ったつまみ"
            );
            assert!(
                meets(palette.cursor(), track, LINE_RATIO),
                "{name}: 溝の上の選んだ範囲の印"
            );
            assert!(
                meets(palette.text(), track, LINE_RATIO),
                "{name}: 溝の上のカーソルの線"
            );
            // つまみに重なる所のカーソルの線は背景の色で描く（つまみは背景との差を保証してある）。
            assert!(
                meets(palette.background(), palette.gutter_text(), TEXT_RATIO),
                "{name}: 指の乗ったつまみの上のカーソルの線"
            );
        }
    }

    #[test]
    fn surfaces_stay_distinguishable_in_every_theme() {
        for seed in &SEEDS {
            let palette = derive(seed);
            let surfaces = [
                palette.background(),
                palette.gutter(),
                palette.selection(),
                palette.band(),
            ];
            for (index, surface) in surfaces.iter().enumerate() {
                assert!(
                    !surfaces[index + 1..].contains(surface),
                    "{}: 面の色が重なる",
                    seed.name()
                );
            }
        }
    }

    /// 色の離れ（チャンネルごとの差の和）。
    fn distance(left: Color, right: Color) -> u32 {
        [
            (left.red(), right.red()),
            (left.green(), right.green()),
            (left.blue(), right.blue()),
        ]
        .into_iter()
        .map(|(a, b)| u32::from(a.abs_diff(b)))
        .sum()
    }

    /// 検索の印（GitHub Issue #189）：当たったセルの上の字が読め、当たった字は明るい印の上の濃い字で読め、当たった字の背景は
    /// ほかの面（背景・面・選択・帯・当たったセル）からはっきり離れている。
    #[test]
    fn every_builtin_theme_marks_found_cells_and_characters_readably() {
        for seed in &SEEDS {
            let palette = derive(seed);
            let name = seed.name();
            assert!(
                meets(palette.text(), palette.found(), TEXT_RATIO),
                "{name}: 当たったセルの字"
            );
            assert!(
                meets(palette.found_ink(), palette.found_text(), TEXT_RATIO),
                "{name}: 当たった字"
            );
            let surfaces = [
                palette.background(),
                palette.gutter(),
                palette.selection(),
                palette.band(),
            ];
            assert!(
                !surfaces.contains(&palette.found()),
                "{name}: 当たったセルが面と重なる"
            );
            for surface in surfaces.into_iter().chain([palette.found()]) {
                assert!(
                    distance(palette.found_text(), surface) >= 100,
                    "{name}: 当たった字の背景 {:?} が {surface:?} に近い",
                    palette.found_text()
                );
            }
        }
    }

    #[test]
    fn the_default_light_theme_is_derived_from_its_three_colors() {
        let seed = named("neutral-light").unwrap();
        let palette = derive(seed);
        assert_eq!(palette.background(), seed.background());
        assert_eq!(palette.text(), seed.foreground());
        assert_eq!(palette.close_hover(), CLOSE_RED);
        assert_eq!(
            palette.gutter(),
            mix(seed.background(), seed.foreground(), LIGHT.gutter)
        );
    }
}
