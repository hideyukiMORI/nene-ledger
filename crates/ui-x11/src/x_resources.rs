//! X の資源（root の `RESOURCE_MANAGER`）から、OS の文字の大きさを読む（A11Y-03・GitHub Issue #263）。

/// 100% の `Xft.dpi`。
const BASE_DPI: u32 = 96;

/// `RESOURCE_MANAGER` の文字列 `resources` の `Xft.dpi` を、文字の倍率（百分率・96 で 100）にする。無い・読めない・
/// 0 なら `None`。値は小数のこともある（`Xft.dpi:\t120.5`）ので、整数の部分だけを使う。
pub(crate) fn text_scale(resources: &str) -> Option<u32> {
    let dpi = resources.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        (name.trim() == "Xft.dpi").then(|| value.trim())
    })?;
    let whole: u32 = dpi.split('.').next()?.parse().ok()?;
    (whole > 0).then(|| whole.saturating_mul(100) / BASE_DPI)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `Xft.dpi` の行だけを読み、96 を 100% とする。無い・壊れた値は `None`。
    #[test]
    fn xft_dpi_becomes_a_percentage() {
        assert_eq!(text_scale("Xft.antialias:\t1\nXft.dpi:\t96\n"), Some(100));
        assert_eq!(text_scale("Xft.dpi: 144"), Some(150));
        assert_eq!(text_scale("Xft.dpi:\t120.5\nXcursor.size:\t24"), Some(125));
        assert_eq!(text_scale("Xcursor.size:\t24\n"), None);
        assert_eq!(text_scale("Xft.dpi:\tlarge"), None);
        assert_eq!(text_scale("Xft.dpi:\t0"), None);
        assert_eq!(text_scale(""), None);
    }
}
