//! 色。

/// 不透明な色（sRGB の 8 ビット × 3）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    red: u8,
    green: u8,
    blue: u8,
}

impl Color {
    /// 色を作る。
    #[must_use]
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    /// 赤。
    #[must_use]
    pub fn red(self) -> u8 {
        self.red
    }

    /// 緑。
    #[must_use]
    pub fn green(self) -> u8 {
        self.green
    }

    /// 青。
    #[must_use]
    pub fn blue(self) -> u8 {
        self.blue
    }

    /// この色から `to` へ `percent` だけ寄せた色（0 でこの色、100 で `to`・100 を超えれば 100・四捨五入）。配色の導出と、
    /// 見えない字の印の色（GitHub Issue #269）の 1 つの計算。
    #[must_use]
    pub fn mixed(self, to: Self, percent: u8) -> Self {
        Self::new(
            channel(self.red, to.red, percent),
            channel(self.green, to.green, percent),
            channel(self.blue, to.blue, percent),
        )
    }
}

/// 1 つの色の成分を `from` から `to` へ `percent` だけ寄せる。
fn channel(from: u8, to: u8, percent: u8) -> u8 {
    let percent = u32::from(percent.min(100));
    let mixed = (u32::from(from) * (100 - percent) + u32::from(to) * percent + 50) / 100;
    u8::try_from(mixed).unwrap_or(u8::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 0 で元の色、100（とそれを超える値）で相手の色、50 で中間（四捨五入）。
    #[test]
    fn mixing_moves_toward_the_other_color() {
        let black = Color::new(0, 0, 0);
        let white = Color::new(255, 255, 255);
        assert_eq!(black.mixed(white, 0), black);
        assert_eq!(black.mixed(white, 100), white);
        assert_eq!(black.mixed(white, 200), white);
        assert_eq!(black.mixed(white, 50), Color::new(128, 128, 128));
        assert_eq!(
            Color::new(10, 20, 30).mixed(Color::new(30, 20, 10), 50),
            Color::new(20, 20, 20)
        );
    }
}
