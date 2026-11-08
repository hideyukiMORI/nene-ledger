//! 窓の下の状態の行の文言。

use alloc::format;
use alloc::string::String;

use nene_ledger_core::line_ending_counts::LineEndingCounts;
use nene_ledger_core::quoting::Quoting;
use nene_ledger_core::text_encoding::TextEncoding;

use crate::columns_notice::ColumnsNotice;
use crate::find_notice::FindNotice;
use crate::format_summary::FormatSummary;
use crate::job_state::JobState;
use crate::notice::Notice;
use crate::paste_notice::PasteNotice;
use crate::read_only_notice::ReadOnlyNotice;
use crate::replace_notice::ReplaceNotice;
use crate::rows_notice::RowsNotice;
use crate::save_problem::SaveProblem;
use crate::selection_stats::SelectionStats;
use crate::selection_totals::SelectionTotals;
use nene_ledger_core::decimal::Decimal;

/// 検索の知らせ。
fn find_status(notice: FindNotice) -> String {
    String::from(match notice {
        FindNotice::Wrapped => "表の端まで探して、反対の端から続けて見つけました",
        FindNotice::NotFound => "見つかりませんでした",
        FindNotice::Stopped => "探すのをやめました",
        FindNotice::Failed => "ファイルを読めなかったので、探すのをやめました",
        FindNotice::NothingToRepeat => "続けて探すものがありません（Ctrl+F で探す）",
        FindNotice::BrokenPattern => {
            "正規表現の式が読めないので探しませんでした（Alt+R で文字どおりに）"
        }
        FindNotice::NoRagged => "列の数が見出しと違う行はありません",
        FindNotice::NoShape => "見出しの行が無いか列を足し消ししたので、列の数を比べられません",
    })
}

/// 置換の知らせ。
fn replace_status(notice: ReplaceNotice) -> String {
    match notice {
        ReplaceNotice::Replaced { cells, skipped } => format!(
            "{} セルを置き換えました{}（Ctrl+Z で戻す）",
            group_digits(u64::try_from(cells).unwrap_or(u64::MAX)),
            skipped_note(skipped)
        ),
        ReplaceNotice::NothingToReplace { skipped } => {
            format!("置き換えるセルがありませんでした{}", skipped_note(skipped))
        }
        ReplaceNotice::TooMany => {
            String::from("100 万セルを越えるので置き換えませんでした（範囲や条件を狭めてください）")
        }
        ReplaceNotice::Changed => String::from(
            "数えている間に表が変わったので置き換えませんでした（もう一度数えてください）",
        ),
        ReplaceNotice::Stopped => String::from("置き換えるのをやめました"),
        ReplaceNotice::Failed => {
            String::from("ファイルを読めなかったので、置き換えるのをやめました")
        }
    }
}

/// 全部の置換が 1 MiB で切り詰めたレコードを飛ばしたなら、その数（無ければ空）。
pub(crate) fn skipped_note(skipped: u64) -> String {
    if skipped == 0 {
        return String::new();
    }
    format!(
        "・1 MiB で切り詰めた {} 行は値の全体が見えないので飛ばしました",
        group_digits(skipped)
    )
}

/// 表への貼り付けの知らせ。
fn paste_status(notice: PasteNotice) -> String {
    match notice {
        PasteNotice::Pasted {
            rows,
            columns,
            added_rows,
            added_columns,
            moved,
        } => pasted_status(rows, columns, (added_rows, added_columns), moved),
        PasteNotice::Cut { rows, columns } => format!(
            "{} 行 × {columns} 列を切り取りました（貼るまで元は残ります・Esc でやめる）",
            group_digits(rows)
        ),
        PasteNotice::Unread => {
            String::from("貼る所の行をファイルから読めなかったので貼りませんでした")
        }
        PasteNotice::CutManyRanges => {
            String::from("離れた範囲は切り取れません（1 つの範囲を選んでください）")
        }
        PasteNotice::TooLarge => {
            String::from("貼り付けが大きすぎるので貼りませんでした（100 万セル・64 MiB まで）")
        }
        PasteNotice::AwaitIndex => String::from(
            "表の終わりを越えるかもしれないので、行を数え終えてから貼ってください（今は見えている所までしか数えていません）",
        ),
    }
}

/// 貼った（切り取りなら動かした）知らせ。表の端を越えて足した行と列があれば添える。
fn pasted_status(
    rows: u64,
    columns: usize,
    (added_rows, added_columns): (u64, usize),
    moved: bool,
) -> String {
    let verb = if moved {
        "動かしました"
    } else {
        "貼りました"
    };
    let shape = format!("{} 行 × {columns} 列を{verb}", group_digits(rows));
    match (added_rows, added_columns) {
        (0, 0) => shape,
        (0, added) => format!("{shape}（{added} 列を足しました）"),
        (added, 0) => format!("{shape}（{} 行を足しました）", group_digits(added)),
        (rows_added, columns_added) => format!(
            "{shape}（{} 行と {columns_added} 列を足しました）",
            group_digits(rows_added)
        ),
    }
}

/// 索引の状態を、窓の下の行の文言にする。🔴 表示言語の文言の表（ADR 0011）を入れるまでの仮の日本語。
#[must_use]
pub fn index_status(state: JobState) -> String {
    match state {
        JobState::Waiting => String::from("索引を作る準備をしています"),
        JobState::Running(progress) => match progress.long_quoted_cell() {
            Some(quote) => format!(
                "索引を作っています {}% · {} バイト目の引用符が閉じていないかもしれません",
                progress.percent(),
                group_digits(quote)
            ),
            None => format!("索引を作っています {}%", progress.percent()),
        },
        JobState::CancelRequested(progress) => {
            format!("索引を止めています（{}% まで）", progress.percent())
        }
        JobState::Finished(summary) => match summary.unclosed_quote() {
            Some(quote) => format!(
                "{} 行 · {} バイト目の引用符が最後まで閉じていません",
                group_digits(summary.records()),
                group_digits(quote)
            ),
            None => format!("{} 行", group_digits(summary.records())),
        },
        JobState::Failed => String::from("索引を作れませんでした"),
        JobState::Cancelled => String::from("索引を中止しました"),
    }
}

/// 読み方の文言（文字コード · 区切り · 引用の扱い · 改行）。引用の扱いは既定（RFC 4180）でないときだけ（Issue #68）。
/// 🔴 表示言語の文言の表（ADR 0011）を入れるまでの仮の日本語。
#[must_use]
pub fn format_status(summary: FormatSummary) -> String {
    let format = summary.format();
    let dialect = format.dialect();
    let quoting = quoting_status(dialect.quoting(), dialect.quote())
        .map(|name| format!(" · {name}"))
        .unwrap_or_default();
    format!(
        "{} · {}{} · {}",
        encoding_name(format.encoding()),
        delimiter_name(dialect.delimiter().byte()),
        quoting,
        line_ending_name(summary.line_endings())
    )
}

/// 状態の行の、引用の扱いの短い名前。既定（`"` を `""` で書く・RFC 4180）なら `None`。
fn quoting_status(quoting: Quoting, quote: u8) -> Option<String> {
    ((quoting, quote) != (Quoting::Doubled, b'"')).then(|| quoting_short_name(quoting, quote))
}

/// 引用の扱いの短い名前（状態の行と、判別の候補の一覧で使う）。
pub(crate) fn quoting_short_name(quoting: Quoting, quote: u8) -> String {
    match (quoting, quote) {
        (Quoting::Doubled, b'"') => String::from("\"\" で引用符（RFC 4180）"),
        (Quoting::Doubled, other) => format!("{} で引用", char::from(other)),
        (Quoting::QuotedBackslash, other) => {
            format!("{} で引用・中は \\ で逃がす", char::from(other))
        }
        (Quoting::Backslash, _) => String::from("\\ で逃がす"),
        (Quoting::Plain, _) => String::from("引用なし"),
    }
}

/// カーソルの位置の文言。`record` はファイルの中のレコードの番号、`column` は列の番号（どちらも 0 始まり・1 始まりで見せる）。
#[must_use]
pub fn cursor_status(record: u64, column: usize) -> String {
    format!(
        "{} 行目 · {} 列目",
        group_digits(record.saturating_add(1)),
        column.saturating_add(1)
    )
}

/// 読み取り専用の知らせ（EDIT-13・GitHub Issue #238）。
const fn read_only_status(notice: ReadOnlyNotice) -> &'static str {
    match notice {
        ReadOnlyNotice::On => "読み取り専用にしました（表を変える操作はしません）",
        ReadOnlyNotice::Off => "読み取り専用を外しました",
        ReadOnlyNotice::Refused => {
            "読み取り専用です。直すには Ctrl+Shift+P の「読み取り専用を切り替える」で外します"
        }
        ReadOnlyNotice::OpenedLocked => {
            "ほかのアプリ（Excel など）が開いているので、読み取り専用で開きました"
        }
        ReadOnlyNotice::OpenedDenied => "書き込みの権限が無いので、読み取り専用で開きました",
    }
}

/// 知らせの文言。
#[must_use]
pub fn notice_status(notice: Notice) -> String {
    match notice {
        Notice::Copied { rows, columns, cut } => copied_status(rows, columns, cut),
        Notice::TooLargeToCopy => String::from("選択が大きすぎるのでコピーしませんでした"),
        Notice::LooksBinary => String::from("文字のファイルではないようです（NUL のバイトがある）"),
        Notice::RangesNotAligned => {
            String::from("離れた範囲は、行か列が揃っていないとコピーできません")
        }
        Notice::CopyAwaitsIndex => String::from(
            "ファイルの終わりまでの行を数え終えてからコピーしてください（索引を作っています）",
        ),
        Notice::NothingToCopy => String::from("コピーする行がありません"),
        Notice::Zoom(level) => format!("文字の大きさ {}%", level.percent()),
        Notice::SettingsUnsaved => {
            String::from("設定を保存できませんでした（この起動の間だけ有効）")
        }
        Notice::Theme { name, position } => format!(
            "配色 {name}（{} / {}）",
            position.saturating_add(1),
            nene_ledger_core::theme_catalog::COUNT
        ),
        Notice::Frozen { columns: 0 } => String::from("列の固定を外しました"),
        Notice::Frozen { columns } => {
            format!("左の {columns} 列を固定しました（横に動いても残ります）")
        }
        Notice::Saved => String::from("保存しました"),
        Notice::Undone(cells) => format!("元に戻しました（{cells} セル）"),
        Notice::Redone(cells) => format!("やり直しました（{cells} セル）"),
        Notice::NothingToUndo => String::from("元に戻すものがありません"),
        Notice::NothingToRedo => String::from("やり直すものがありません"),
        Notice::NothingToSave => String::from("変えた所が無いので保存しませんでした"),
        Notice::StillSaving => String::from("保存しています。終わるまで直せません"),
        Notice::SaveFailed(problem) => save_problem_status(problem),
        Notice::NewWindowFailed => String::from("新しい表の窓を開けませんでした"),
        Notice::ReadOnly(read_only) => String::from(read_only_status(read_only)),
        Notice::Paste(paste) => paste_status(paste),
        Notice::Find(find) => find_status(find),
        Notice::Replace(replace) => replace_status(replace),
        Notice::ReopenFailed => {
            String::from("保存しました。開き直せなかったので、表示は保存の前のままです")
        }
        Notice::ReadingNeedsFile => {
            String::from("新しい表は読み方を変えられません（名前を付けて保存してから）")
        }
        Notice::ReadingNeedsSave => {
            String::from("変えた所がある間は読み方を変えられません（先に Ctrl+S で保存）")
        }
        Notice::EditCut => String::from(
            "1 MiB で切り詰めた行のセルは直せません（行の全体を読めないので、保存で元に戻せない）",
        ),
        Notice::Rows(rows) => rows_status(rows),
        Notice::Columns(columns) => columns_status(columns),
        Notice::EditMissingCell => String::from("この行には、この列のセルがありません"),
        Notice::DialectUnsure { top, second } => format!(
            "判別に確信なし（1 位 {}%・2 位 {}%）· 候補は Ctrl+Shift+P → candidates",
            top / 10,
            second / 10
        ),
    }
}

/// コピーした知らせの文言（`cut` は 1 MiB で切り詰めた行の数）。
fn copied_status(rows: u64, columns: usize, cut: u64) -> String {
    if cut == 0 {
        format!("{} 行 × {} 列をコピーしました", group_digits(rows), columns)
    } else {
        format!(
            "{} 行 × {} 列をコピーしました（うち {} 行は 1 MiB で切り詰めた値）",
            group_digits(rows),
            columns,
            group_digits(cut)
        )
    }
}

/// 行の足し消しの知らせの文言。
fn rows_status(notice: RowsNotice) -> String {
    match notice {
        RowsNotice::Inserted(rows) => format!("{} 行を足しました", group_digits(rows)),
        RowsNotice::Removed(rows) => {
            format!("{} 行を消しました（Ctrl+Z で戻す）", group_digits(rows))
        }
        RowsNotice::Undone(rows) => format!("元に戻しました（{} 行）", group_digits(rows)),
        RowsNotice::Redone(rows) => format!("やり直しました（{} 行）", group_digits(rows)),
        RowsNotice::AwaitIndex => {
            String::from("表の末尾は、行の数が分かってから扱えます（索引を作っています）")
        }
    }
}

/// 列の足し消しの知らせの文言。
fn columns_status(notice: ColumnsNotice) -> String {
    match notice {
        ColumnsNotice::Inserted(columns) => format!("{columns} 列を足しました"),
        ColumnsNotice::Removed(columns) => format!("{columns} 列を消しました（Ctrl+Z で戻す）"),
        ColumnsNotice::Undone(columns) => format!("元に戻しました（{columns} 列）"),
        ColumnsNotice::Redone(columns) => format!("やり直しました（{columns} 列）"),
        ColumnsNotice::Reordered => String::from(
            "入れ替えた列が離れているので足し消しできません（先に「列の並びを元に戻す」）",
        ),
        ColumnsNotice::KeepOne => String::from("すべての列は消せません"),
    }
}

/// 保存できなかった理由の文言。書き換えは残っていることを添える。
fn save_problem_status(problem: SaveProblem) -> String {
    let reason = match problem {
        SaveProblem::ChangedOutside => {
            String::from("開いた後にほかのアプリがファイルを変えたので、上書きしませんでした")
        }
        SaveProblem::Locked => {
            String::from("ほかのアプリがファイルを開いているので保存できませんでした")
        }
        SaveProblem::Unrepresentable(character) => {
            format!("「{character}」はファイルの文字コードで表せないので保存できませんでした")
        }
        SaveProblem::HardLinked => {
            String::from("ハードリンクのあるファイルなので保存しませんでした")
        }
        SaveProblem::Interrupted => String::from(
            "置き換えが途中で止まりました。新しい内容は同じフォルダの .nene-ledger-*.tmp に残しました",
        ),
        SaveProblem::Unplannable => String::from("書き換えた行を読めないので保存できませんでした"),
        SaveProblem::Cancelled => String::from("保存をやめました。ファイルは元のままです"),
        SaveProblem::Untitled => String::from(
            "新しい表にはまだファイルがありません。Ctrl+Shift+S で名前を付けて保存します",
        ),
        SaveProblem::Other => String::from("保存できませんでした"),
    };
    // ほかのアプリが開いているなら、どうすればよいか（閉じてからもう一度・別名で保存）を添える（ADR 0025 の施主の判断 3
    // 「再試行・別名で保存・やめるを選ばせる」・FILE-04・GitHub Issue #336）。
    match problem {
        SaveProblem::Locked => {
            format!("{reason}（直した値は残っています · 閉じてから Ctrl+S · 別名は Ctrl+Shift+S）")
        }
        SaveProblem::ChangedOutside
        | SaveProblem::Unrepresentable(_)
        | SaveProblem::HardLinked
        | SaveProblem::Interrupted
        | SaveProblem::Unplannable
        | SaveProblem::Cancelled
        | SaveProblem::Untitled
        | SaveProblem::Other => format!("{reason}（直した値は残っています）"),
    }
}

/// 選んだ範囲の集計の文言。出さないなら `None`。数として読めないセルの数は必ず出す。
#[must_use]
pub fn totals_status(totals: SelectionTotals) -> Option<String> {
    match totals {
        SelectionTotals::Hidden => None,
        SelectionTotals::Counted(stats) => Some(stats_status(stats)),
        SelectionTotals::Counting(progress) => Some(format!(
            "合計を計算しています {}%（Esc で止める）",
            progress.percent()
        )),
        SelectionTotals::Stopped => Some(String::from("合計を止めました")),
        SelectionTotals::Failed => Some(String::from("合計を出せませんでした")),
    }
}

/// 範囲に列の数が見出しと違う行があるときの、集計の頭の言葉（状態の行はこれで始まる集計を注意の色にする・GitHub Issue #329）。
pub const RAGGED_TOTALS: &str = "列の数が違う行";

/// 数えた集計の文言（件数 · 合計 · 平均 · 記号や指数を読んだ数 · 数でない）。数が 1 つも無ければ合計と平均を出さない。
fn stats_status(stats: SelectionStats) -> String {
    let mut parts = alloc::vec::Vec::new();
    // 列の数が見出しと違う行が範囲にあれば、ずれた値を足しているかもしれないので頭に言う（GitHub Issue #329）。
    if stats.ragged_rows() > 0 {
        parts.push(format!(
            "{RAGGED_TOTALS} {} を含む",
            group_digits(stats.ragged_rows())
        ));
    }
    parts.push(format!("件数 {}", group_digits(stats.cells())));
    if stats.numbers() > 0 {
        let sum = stats
            .sum()
            .map_or_else(|| String::from("桁あふれ"), decimal_status);
        parts.push(format!("合計 {sum}"));
        parts.extend(
            stats
                .average()
                .map(|average| format!("平均 {}", decimal_status(average))),
        );
    }
    // 書き方を読み替えて数にしたセルがあれば、その数を添える（¥・桁区切り・指数・全角など・GitHub Issue #319）。
    if stats.rewritten() > 0 {
        parts.push(format!(
            "記号や指数を読んだ {}",
            group_digits(stats.rewritten())
        ));
    }
    parts.push(format!("数でない {}", group_digits(stats.not_numbers())));
    parts.join(" · ")
}

/// 10 進の数を、整数の部分を 3 桁ごとにカンマで区切って見せる（-1,234.50）。
#[must_use]
pub fn decimal_status(number: Decimal) -> String {
    let plain = number.to_plain();
    let (sign, unsigned) = match plain.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", plain.as_str()),
    };
    let (whole, fraction) = match unsigned.split_once('.') {
        Some((whole, fraction)) => (whole, Some(fraction)),
        None => (unsigned, None),
    };
    let mut out = String::from(sign);
    push_grouped(&mut out, whole);
    if let Some(fraction) = fraction {
        out.push('.');
        out.push_str(fraction);
    }
    out
}

/// 文字コードの名前。BOM があれば添える。
pub(crate) fn encoding_name(encoding: TextEncoding) -> &'static str {
    match encoding {
        TextEncoding::Utf8 => "UTF-8",
        TextEncoding::Utf8WithBom => "UTF-8（BOM）",
        TextEncoding::Utf16LeWithBom => "UTF-16LE（BOM）",
        TextEncoding::Utf16BeWithBom => "UTF-16BE（BOM）",
        TextEncoding::Utf16Le => "UTF-16LE",
        TextEncoding::Utf16Be => "UTF-16BE",
        TextEncoding::ShiftJis => "Shift_JIS（CP932）",
        TextEncoding::EucJp => "EUC-JP",
        TextEncoding::Windows1252 => "Windows-1252",
    }
}

/// セルの値のバイト数に添える文字コードの名前（BOM はセルに関わらないので添えない）。
pub(crate) fn byte_encoding_name(encoding: TextEncoding) -> &'static str {
    match encoding {
        TextEncoding::Utf8 | TextEncoding::Utf8WithBom => "UTF-8",
        TextEncoding::Utf16LeWithBom | TextEncoding::Utf16Le => "UTF-16LE",
        TextEncoding::Utf16BeWithBom | TextEncoding::Utf16Be => "UTF-16BE",
        TextEncoding::ShiftJis => "Shift_JIS",
        TextEncoding::EucJp => "EUC-JP",
        TextEncoding::Windows1252 => "Windows-1252",
    }
}

/// 区切りの名前。名前の無い区切りは字をそのまま見せる。
pub(crate) fn delimiter_name(byte: u8) -> String {
    match byte {
        b',' => String::from("カンマ区切り"),
        b'\t' => String::from("タブ区切り"),
        b';' => String::from("セミコロン区切り"),
        b'|' => String::from("縦棒区切り"),
        b' ' => String::from("空白区切り"),
        other => format!("「{}」区切り", char::from(other)),
    }
}

/// 改行の名前。混ざっていれば内訳を見せる。標本に改行が無ければそう言う。
fn line_ending_name(counts: LineEndingCounts) -> String {
    if counts.mixed() {
        return format!(
            "改行が混在（CRLF {} · LF {} · CR {}）",
            group_digits(u64::try_from(counts.crlf()).unwrap_or(u64::MAX)),
            group_digits(u64::try_from(counts.lf()).unwrap_or(u64::MAX)),
            group_digits(u64::try_from(counts.cr()).unwrap_or(u64::MAX))
        );
    }
    let name = if counts.crlf() > 0 {
        "CRLF"
    } else if counts.lf() > 0 {
        "LF"
    } else if counts.cr() > 0 {
        "CR"
    } else {
        "改行なし"
    };
    String::from(name)
}

/// 3 桁ごとにカンマで区切る（1,234,567）。
#[must_use]
pub fn group_digits(number: u64) -> String {
    let mut out = String::new();
    push_grouped(&mut out, &format!("{number}"));
    out
}

/// 数字の並び `digits` を、3 桁ごとにカンマで区切って `out` に足す。
fn push_grouped(out: &mut String, digits: &str) {
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len().saturating_sub(index)).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index_summary::IndexSummary;
    use crate::job_progress::JobProgress;

    #[test]
    fn digits_are_grouped_by_three() {
        assert_eq!(group_digits(0), "0");
        assert_eq!(group_digits(999), "999");
        assert_eq!(group_digits(1_000), "1,000");
        assert_eq!(group_digits(1_234_567), "1,234,567");
        assert_eq!(group_digits(u64::MAX), "18,446,744,073,709,551,615");
    }

    #[test]
    fn decimals_are_grouped_in_the_whole_part_only() {
        assert_eq!(decimal_status(Decimal::new(-123_450, 2)), "-1,234.50");
        assert_eq!(decimal_status(Decimal::new(1_234_567, 0)), "1,234,567");
        assert_eq!(decimal_status(Decimal::new(5, 3)), "0.005");
    }

    fn stats_of(cells: &[&str]) -> SelectionStats {
        let rows: alloc::vec::Vec<alloc::vec::Vec<String>> = cells
            .iter()
            .map(|&cell| alloc::vec![String::from(cell)])
            .collect();
        let slices: alloc::vec::Vec<&[String]> =
            rows.iter().map(alloc::vec::Vec::as_slice).collect();
        SelectionStats::of(&slices, &(0..=0))
    }

    /// 書き方を読み替えて数にしたセル（¥・桁区切り・指数）は、その数を添える。素の数だけなら添えない（GitHub Issue #319）。
    #[test]
    fn totals_say_how_many_values_were_read_as_numbers_by_rewriting() {
        assert_eq!(
            totals_status(SelectionTotals::Counted(stats_of(&[
                "0012", "¥3", "1.5e3", "1"
            ])))
            .as_deref(),
            Some("件数 4 · 合計 1,516 · 平均 379 · 記号や指数を読んだ 2 · 数でない 0")
        );
        assert_eq!(
            totals_status(SelectionTotals::Counted(stats_of(&["0012", "-3.5"]))).as_deref(),
            Some("件数 2 · 合計 8.5 · 平均 4.25 · 数でない 0")
        );
    }

    /// 範囲に列の数が見出しと違う行があれば、集計の頭にその数を言う（GitHub Issue #329）。
    #[test]
    fn totals_say_first_that_rows_with_another_number_of_cells_are_included() {
        let mut stats = stats_of(&["1", "2"]);
        stats.add_shape(3, 3);
        assert!(
            !totals_status(SelectionTotals::Counted(stats))
                .unwrap()
                .starts_with(RAGGED_TOTALS)
        );
        stats.add_shape(4, 3);
        stats.add_shape(2, 3);
        assert_eq!(
            totals_status(SelectionTotals::Counted(stats)).as_deref(),
            Some("列の数が違う行 2 を含む · 件数 2 · 合計 3 · 平均 1.5 · 数でない 0")
        );
    }

    /// ほかのアプリが開いていて保存できなければ、閉じてからもう一度・別名で保存のキーを添える。ほかの理由は値が残ることだけ
    /// （ADR 0025 の施主の判断 3・GitHub Issue #336）。
    #[test]
    fn a_locked_save_says_how_to_retry_or_save_elsewhere() {
        use crate::notice::Notice;
        use crate::save_problem::SaveProblem;
        assert_eq!(
            notice_status(Notice::SaveFailed(SaveProblem::Locked)),
            "ほかのアプリがファイルを開いているので保存できませんでした（直した値は残っています · 閉じてから Ctrl+S · 別名は Ctrl+Shift+S）"
        );
        assert_eq!(
            notice_status(Notice::SaveFailed(SaveProblem::Other)),
            "保存できませんでした（直した値は残っています）"
        );
    }

    #[test]
    fn totals_show_the_count_of_cells_that_are_not_numbers() {
        assert_eq!(totals_status(SelectionTotals::Hidden), None);
        assert_eq!(
            totals_status(SelectionTotals::Counted(stats_of(&["1000.5", "2", "x"]))).as_deref(),
            Some("件数 3 · 合計 1,002.5 · 平均 501.25 · 数でない 1")
        );
        assert_eq!(
            totals_status(SelectionTotals::Counted(stats_of(&["a", "b"]))).as_deref(),
            Some("件数 2 · 数でない 2")
        );
        let huge = "9".repeat(38);
        assert_eq!(
            totals_status(SelectionTotals::Counted(stats_of(&[&huge, &huge]))).as_deref(),
            Some("件数 2 · 合計 桁あふれ · 数でない 0")
        );
        let counting = SelectionTotals::Counting(JobProgress::new(37, 100, None));
        assert_eq!(
            totals_status(counting).as_deref(),
            Some("合計を計算しています 37%（Esc で止める）")
        );
        assert_eq!(
            totals_status(SelectionTotals::Stopped).as_deref(),
            Some("合計を止めました")
        );
        assert_eq!(
            totals_status(SelectionTotals::Failed).as_deref(),
            Some("合計を出せませんでした")
        );
    }

    #[test]
    fn each_state_has_a_line() {
        assert_eq!(
            index_status(JobState::Waiting),
            "索引を作る準備をしています"
        );
        let running = JobState::Running(JobProgress::new(45, 100, None));
        assert_eq!(index_status(running), "索引を作っています 45%");
        let suspicious = JobState::Running(JobProgress::new(50, 100, Some(1_048_577)));
        assert_eq!(
            index_status(suspicious),
            "索引を作っています 50% · 1,048,577 バイト目の引用符が閉じていないかもしれません"
        );
        let stopping = JobState::CancelRequested(JobProgress::new(30, 100, None));
        assert_eq!(index_status(stopping), "索引を止めています（30% まで）");
        let finished = JobState::Finished(IndexSummary::new(1_234_567, None));
        assert_eq!(index_status(finished), "1,234,567 行");
        let unclosed = JobState::Finished(IndexSummary::new(2, Some(4)));
        assert_eq!(
            index_status(unclosed),
            "2 行 · 4 バイト目の引用符が最後まで閉じていません"
        );
        assert_eq!(index_status(JobState::Failed), "索引を作れませんでした");
        assert_eq!(index_status(JobState::Cancelled), "索引を中止しました");
    }

    use alloc::vec;
    use alloc::vec::Vec;
    use nene_ledger_core::delimiter::Delimiter;
    use nene_ledger_core::dialect::Dialect;
    use nene_ledger_core::line_ending::LineEnding;
    use nene_ledger_core::quoting::Quoting;
    use nene_ledger_core::text_format::TextFormat;

    fn summary(encoding: TextEncoding, delimiter: u8, endings: &[LineEnding]) -> FormatSummary {
        let dialect =
            Dialect::new(Delimiter::new(delimiter).unwrap(), Quoting::Doubled, b'"').unwrap();
        FormatSummary::new(
            TextFormat::new(encoding, dialect),
            LineEndingCounts::tally(endings.iter().copied()),
        )
    }

    #[test]
    fn the_format_line_names_the_quoting_only_when_it_is_not_rfc4180() {
        let with = |quoting, quote| {
            let dialect = Dialect::new(Delimiter::TAB, quoting, quote).unwrap();
            format_status(FormatSummary::new(
                TextFormat::new(TextEncoding::Utf8, dialect),
                LineEndingCounts::tally([LineEnding::Lf].into_iter()),
            ))
        };
        assert_eq!(with(Quoting::Doubled, b'"'), "UTF-8 · タブ区切り · LF");
        assert_eq!(
            with(Quoting::Doubled, b'\''),
            "UTF-8 · タブ区切り · ' で引用 · LF"
        );
        assert_eq!(
            with(Quoting::QuotedBackslash, b'"'),
            "UTF-8 · タブ区切り · \" で引用・中は \\ で逃がす · LF"
        );
        assert_eq!(
            with(Quoting::Backslash, b'"'),
            "UTF-8 · タブ区切り · \\ で逃がす · LF"
        );
        assert_eq!(
            with(Quoting::Plain, b'"'),
            "UTF-8 · タブ区切り · 引用なし · LF"
        );
    }

    #[test]
    fn the_format_line_names_encoding_delimiter_and_line_ending() {
        let crlf = [LineEnding::CrLf, LineEnding::CrLf];
        assert_eq!(
            format_status(summary(TextEncoding::Utf8WithBom, b',', &crlf)),
            "UTF-8（BOM） · カンマ区切り · CRLF"
        );
        assert_eq!(
            format_status(summary(TextEncoding::ShiftJis, b'\t', &[LineEnding::Lf])),
            "Shift_JIS（CP932） · タブ区切り · LF"
        );
        assert_eq!(
            format_status(summary(TextEncoding::Utf8, b':', &[LineEnding::Cr])),
            "UTF-8 · 「:」区切り · CR"
        );
        let mixed = [LineEnding::CrLf, LineEnding::Lf, LineEnding::Lf];
        assert_eq!(
            format_status(summary(TextEncoding::EucJp, b';', &mixed)),
            "EUC-JP · セミコロン区切り · 改行が混在（CRLF 1 · LF 2 · CR 0）"
        );
        assert_eq!(
            format_status(summary(TextEncoding::Windows1252, b'|', &[])),
            "Windows-1252 · 縦棒区切り · 改行なし"
        );
    }

    #[test]
    fn every_encoding_and_named_delimiter_has_a_name() {
        let names: Vec<&str> = [
            TextEncoding::Utf16LeWithBom,
            TextEncoding::Utf16BeWithBom,
            TextEncoding::Utf16Le,
            TextEncoding::Utf16Be,
        ]
        .into_iter()
        .map(encoding_name)
        .collect();
        assert_eq!(
            names,
            vec!["UTF-16LE（BOM）", "UTF-16BE（BOM）", "UTF-16LE", "UTF-16BE"]
        );
        assert_eq!(delimiter_name(b' '), "空白区切り");
    }

    /// コピーしなかった理由の文言（GitHub Issue #39）。
    #[test]
    fn copy_refusals_have_lines() {
        assert_eq!(
            notice_status(Notice::CopyAwaitsIndex),
            "ファイルの終わりまでの行を数え終えてからコピーしてください（索引を作っています）"
        );
        assert_eq!(
            notice_status(Notice::NothingToCopy),
            "コピーする行がありません"
        );
    }

    #[test]
    fn notices_have_lines() {
        assert_eq!(
            notice_status(Notice::Copied {
                rows: 12_345,
                columns: 3,
                cut: 0
            }),
            "12,345 行 × 3 列をコピーしました"
        );
        assert_eq!(
            notice_status(Notice::Copied {
                rows: 10,
                columns: 2,
                cut: 1
            }),
            "10 行 × 2 列をコピーしました（うち 1 行は 1 MiB で切り詰めた値）"
        );
        assert_eq!(
            notice_status(Notice::Theme {
                name: "neutral-light",
                position: 1
            }),
            "配色 neutral-light（2 / 125）"
        );
        assert_eq!(
            notice_status(Notice::Zoom(crate::zoom_level::ZoomLevel::hundred())),
            "文字の大きさ 100%"
        );
        assert_eq!(
            notice_status(Notice::SettingsUnsaved),
            "設定を保存できませんでした（この起動の間だけ有効）"
        );
        assert_eq!(
            notice_status(Notice::TooLargeToCopy),
            "選択が大きすぎるのでコピーしませんでした"
        );
        assert_eq!(
            notice_status(Notice::Frozen { columns: 2 }),
            "左の 2 列を固定しました（横に動いても残ります）"
        );
        assert_eq!(
            notice_status(Notice::Frozen { columns: 0 }),
            "列の固定を外しました"
        );
        assert_eq!(
            notice_status(Notice::DialectUnsure {
                top: 412,
                second: 398
            }),
            "判別に確信なし（1 位 41%・2 位 39%）· 候補は Ctrl+Shift+P → candidates"
        );
    }

    #[test]
    fn the_cursor_line_counts_from_one() {
        assert_eq!(cursor_status(0, 0), "1 行目 · 1 列目");
        assert_eq!(cursor_status(1_233, 8), "1,234 行目 · 9 列目");
    }
}
