//! 分けて受け取っている途中の貼り付けの中身（`INCR`・ICCCM）。

/// 受け取る上限（ADR 0029 の提案と同じ 64 MiB）。越えれば捨てて、終わりまで読み流す。
const LIMIT: usize = 64 << 20;

/// 分けて受け取っている途中の貼り付けの中身。
#[derive(Debug)]
pub(crate) struct IncomingPaste {
    bytes: Vec<u8>,
    overflowed: bool,
}

impl IncomingPaste {
    /// 空で始める（持ち主が `INCR` を返したとき）。
    pub(crate) fn new() -> Self {
        Self {
            bytes: Vec::new(),
            overflowed: false,
        }
    }

    /// 1 つの分 `part` を足す。上限を越えたら、以後は捨てる。
    pub(crate) fn push(&mut self, part: &[u8]) {
        if self.overflowed || self.bytes.len().saturating_add(part.len()) > LIMIT {
            self.overflowed = true;
            self.bytes = Vec::new();
            return;
        }
        self.bytes.extend_from_slice(part);
    }

    /// 受け取り終えた文字列。上限を越えていれば空。UTF-8 でないバイトは置き換える。
    pub(crate) fn finish(self) -> String {
        if self.overflowed {
            return String::new();
        }
        String::from_utf8_lossy(&self.bytes).into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parts_are_joined_and_too_much_is_dropped() {
        let mut incoming = IncomingPaste::new();
        incoming.push(b"a\t");
        incoming.push("東".as_bytes());
        assert_eq!(incoming.finish(), "a\t東");
        let mut huge = IncomingPaste::new();
        huge.push(&vec![b'x'; LIMIT]);
        huge.push(b"y");
        huge.push(b"z");
        assert_eq!(huge.finish(), "");
    }
}
