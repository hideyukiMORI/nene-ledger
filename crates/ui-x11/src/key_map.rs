//! keycode から keysym を引く表。

/// X のキーの対応表（keycode → keysym）。窓を開くときにサーバから 1 度だけ受け取る。
/// 各 keycode の最初の keysym（修飾なし）だけを見る。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct KeyMap {
    min_keycode: u8,
    per_keycode: usize,
    keysyms: Vec<u32>,
}

impl KeyMap {
    /// 表を作る。`keysyms` は keycode ごとに `per_keycode` 個ずつ並ぶ。
    pub(crate) fn new(min_keycode: u8, per_keycode: u8, keysyms: Vec<u32>) -> Self {
        Self {
            min_keycode,
            per_keycode: usize::from(per_keycode),
            keysyms,
        }
    }

    /// `keycode` の修飾なしの keysym。表の外なら `None`。
    pub(crate) fn keysym(&self, keycode: u8) -> Option<u32> {
        self.column(keycode, 0)
    }

    /// `keycode` を `shifted`（Shift を押した）とき打たれる字。半角の英数字と記号（keysym 0x20〜0x7E は同じ値の字）だけ。
    /// Shift の欄が空（0）なら修飾なしの欄を使う。
    pub(crate) fn typed(&self, keycode: u8, shifted: bool) -> Option<char> {
        let plain = self.column(keycode, 0)?;
        let keysym = if shifted {
            self.column(keycode, 1)
                .filter(|&keysym| keysym != 0)
                .unwrap_or(plain)
        } else {
            plain
        };
        u8::try_from(keysym)
            .ok()
            .filter(|byte| (0x20..=0x7E).contains(byte))
            .map(char::from)
    }

    /// `keycode` の `column` 欄の keysym。
    fn column(&self, keycode: u8, column: usize) -> Option<u32> {
        let offset = usize::from(keycode.checked_sub(self.min_keycode)?);
        if column >= self.per_keycode {
            return None;
        }
        self.keysyms
            .get(offset.checked_mul(self.per_keycode)?.checked_add(column)?)
            .copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn looks_up_the_first_keysym_of_each_keycode() {
        let map = KeyMap::new(8, 2, vec![0x61, 0x41, 0xff52, 0]);
        assert_eq!(map.keysym(8), Some(0x61));
        assert_eq!(map.keysym(9), Some(0xff52));
        assert_eq!(map.keysym(7), None);
        // 打たれる字: 修飾なしは a、Shift は A。矢印（0xff52）は字にならない。
        assert_eq!(map.typed(8, false), Some('a'));
        assert_eq!(map.typed(8, true), Some('A'));
        assert_eq!(map.typed(9, false), None);
        // Shift の欄が空なら修飾なしの欄。
        let digits = KeyMap::new(10, 2, vec![0x31, 0]);
        assert_eq!(digits.typed(10, true), Some('1'));
        assert_eq!(KeyMap::new(10, 1, vec![0x31]).typed(10, true), Some('1'));
        assert_eq!(map.keysym(10), None);
    }
}
