//! 裏で書いている仕事の種類。

/// 裏で書いている仕事の種類（進み具合と「終わるまで直せません」の言い方を分ける）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WriteKind {
    /// 保存（Ctrl+S・名前を付けて保存）。終われば窓は書いたファイルを開き直す。
    Save,
    /// 方言を変えた書き出し（ADR 0036）。終わっても窓は元のファイルのまま。
    Export,
}
