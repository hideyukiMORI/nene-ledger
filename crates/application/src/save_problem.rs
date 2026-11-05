//! 保存できなかった理由のうち、状態の行で利用者に示すもの（ADR 0025 の決定 2・3・ADR 0026 の決定 6）。

/// 保存できなかった理由（状態の行の言葉を選ぶため）。どれでも、原本は書いていない（`Interrupted` を除く）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveProblem {
    /// 開いた後に、ほかのアプリがファイルを変えた（FILE-01）。
    ChangedOutside,
    /// ほかのアプリがファイルを開いている・置き換えを拒まれた（FILE-04）。
    Locked,
    /// 文字コードで表せない字がある（最初の 1 字・SAVE-04）。
    Unrepresentable(char),
    /// ハードリンクのあるファイル（施主の判断 2）。
    HardLinked,
    /// 置き換えが途中で止まった。新しい内容の一時ファイルを残した。
    Interrupted,
    /// 書き換えた行を読めない・長すぎる・その方言で書けない値。
    Unplannable,
    /// 原本のファイルが無い新しい表（名前を付けて保存が要る・ADR 0028）。
    Untitled,
    /// それ以外（書けない・読めない・スレッドを作れない）。
    Other,
}
