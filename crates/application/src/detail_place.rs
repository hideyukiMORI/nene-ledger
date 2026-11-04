//! 詳細ペインの見出しに出す、カーソルのセルの場所。

/// 詳細ペインの見出しに出す、カーソルのセルの場所。ファイルの中のレコードの番号と列の番号（どちらも 0 始まり）と、
/// 見出しがあればその列の名前（表示の文字列）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DetailPlace<'header> {
    record: u64,
    column: usize,
    name: Option<&'header str>,
}

impl<'header> DetailPlace<'header> {
    pub(crate) fn new(record: u64, column: usize, name: Option<&'header str>) -> Self {
        Self {
            record,
            column,
            name,
        }
    }

    pub(crate) fn record(self) -> u64 {
        self.record
    }

    pub(crate) fn column(self) -> usize {
        self.column
    }

    pub(crate) fn name(self) -> Option<&'header str> {
        self.name
    }
}
