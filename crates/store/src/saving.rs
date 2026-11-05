//! 保存の手順（ADR 0025 の決定 1）。

use std::fs::File;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use nene_ledger_application::file_stamp::FileStamp;
use nene_ledger_application::replace_failure::ReplaceFailure;
use nene_ledger_application::replacing::Replacing;
use nene_ledger_application::save_target::SaveTarget;
use nene_ledger_application::source_bytes::SourceBytes;
use nene_ledger_core::plan_sink::PlanSink;
use nene_ledger_core::save_plan::SavePlan;
use nene_ledger_core::save_step::SaveStep;
use nene_ledger_core::text_encoding::TextEncoding;

use crate::encoding::encoded;
use crate::file_sink::FileSink;
use crate::save_failure::SaveFailure;
use crate::stamping::stamp_of;
use crate::temporary_file::create_beside;

/// 保存の計画 `plan`（書き換えた文字列は原本の文字コード `encoding` で書く）を、`target` と同じフォルダの一時ファイルに書き、同期してから `replacing` で置き換える。
/// 保存先がまだ無い（[`SaveTarget::Absent`]・名前を付けて保存）なら、置き換えずに一時ファイルの名前を変えて作る（`save_absent`）。
///
/// - 原本の印が、開く前に取った印（[`SaveTarget::Existing`]・[`stamp_of`]）と違えば、書かない。書き終えて置き換える直前にも確かめる
/// - Windows では、書く前に置き換えられるか（ほかのアプリのロック）を確かめる
/// - `target` がシンボリックリンクなら、行き先のファイルを保存する（リンクを実体で置き換えない）
/// - 書き換えた文字列は、書き始める前に全部を `encoding` で符号化する。表せない字があれば何も書かない
/// - 書いている途中で失敗したら一時ファイルを消す。🔴 原本には、置き換えの瞬間まで触れない（ADR 0005）
/// - 置き換えに失敗したら、一時ファイルを残して知らせる（[`SaveFailure::Replace`]）
///
/// # Errors
///
/// [`SaveFailure`] のとおり。
pub fn save<S, R>(
    target: &Path,
    (plan, encoding, expected): (&SavePlan, TextEncoding, SaveTarget),
    source: &mut S,
    replacing: &R,
) -> Result<(), SaveFailure>
where
    S: SourceBytes<Failure = ErrorKind>,
    R: Replacing<Location = Path>,
{
    encodable(plan, encoding)?;
    save_streamed(
        target,
        (encoding, expected),
        (source, replacing),
        |sink: &mut dyn PlanSink| {
            plan.replay(sink);
            Ok(())
        },
    )
}

/// [`save`] と同じ手順で、計画を記憶に持たずに書く（GitHub Issue #130）。`produce` が計画の歩を書き先へ順に渡し、
/// 書き先が一時ファイルに直に書く。`produce` が失敗すれば（`SaveFailure::Plan`）書いた一時ファイルを消す。
/// 表せない字は書きながら見つける（呼ぶ側が書き換えた値を先に [`encoded`] で確かめておけば、書く前に止められる）。
///
/// # Errors
///
/// [`SaveFailure`] のとおり。
pub fn save_streamed<S, R, P>(
    target: &Path,
    (encoding, expected): (TextEncoding, SaveTarget),
    (source, replacing): (&mut S, &R),
    produce: P,
) -> Result<(), SaveFailure>
where
    S: SourceBytes<Failure = ErrorKind>,
    R: Replacing<Location = Path>,
    P: FnOnce(&mut dyn PlanSink) -> Result<(), SaveFailure>,
{
    let opened = match expected {
        SaveTarget::Existing(opened) => opened,
        SaveTarget::Absent => return save_absent(target, encoding, source, produce),
    };
    let target =
        std::fs::canonicalize(target).map_err(|error| SaveFailure::Target(error.kind()))?;
    let metadata = std::fs::metadata(&target).map_err(|error| SaveFailure::Target(error.kind()))?;
    unchanged_since(&target, opened)?;
    refuse_hard_links(&metadata)?;
    // Unix では、開いているほかのプロセスがあっても `rename` で置き換えられる（確かめることは無い）。
    #[cfg(windows)]
    crate::lock_probe::probe(&target).map_err(SaveFailure::Blocked)?;
    let (file, temporary) = create_beside(&target).map_err(SaveFailure::Write)?;
    if let Err(failure) = write_streamed(file, encoding, source, produce)
        .and_then(|()| unchanged_since(&target, opened))
        .and_then(|()| source_unchanged(source))
    {
        discard(&temporary);
        return Err(failure);
    }
    keep_permissions(&temporary, &metadata);
    replace_with(replacing, temporary, &target)
}

/// まだ無い `target` に、計画 `plan` を書いて作る（名前を付けて保存・ADR 0028 の決定 5）。
///
/// 同じフォルダの一時ファイルに書いて同期し、`target` がまだ無いことを確かめてから名前を変える（Linux は `rename`、
/// Windows は std の `MoveFileExW`）。`ReplaceFileW` は置き換え先が無いと使えないので、ここでは置き換えのポートを使わない。
/// 書く前か名前を変える直前に `target` ができていれば、書かない（[`SaveFailure::ChangedOutside`]）。
/// 🔴 確かめてから名前を変えるまでの間に、ほかのアプリが同じ名前で作れば、それを上書きする（依存を足さずに
/// 「無ければ作る」を原子的にする手が std に無い・ADR 0025 の判断 6 の `rustix` と一緒に決める）。
///
/// # Errors
///
/// [`SaveFailure`] のとおり。
fn save_absent<S, P>(
    target: &Path,
    encoding: TextEncoding,
    source: &mut S,
    produce: P,
) -> Result<(), SaveFailure>
where
    S: SourceBytes<Failure = ErrorKind>,
    P: FnOnce(&mut dyn PlanSink) -> Result<(), SaveFailure>,
{
    let name = target
        .file_name()
        .ok_or(SaveFailure::Target(ErrorKind::InvalidInput))?;
    let folder = match target.parent() {
        Some(folder) if !folder.as_os_str().is_empty() => folder,
        Some(_) | None => Path::new("."),
    };
    let folder =
        std::fs::canonicalize(folder).map_err(|error| SaveFailure::Target(error.kind()))?;
    let created = folder.join(name);
    still_absent(&created)?;
    let (file, temporary) = create_beside(&created).map_err(SaveFailure::Write)?;
    if let Err(failure) = write_streamed(file, encoding, source, produce)
        .and_then(|()| still_absent(&created))
        .and_then(|()| source_unchanged(source))
    {
        discard(&temporary);
        return Err(failure);
    }
    std::fs::rename(&temporary, &created).map_err(|_error| SaveFailure::Replace {
        failure: ReplaceFailure::Other,
        temporary,
    })?;
    sync_folder(&folder);
    Ok(())
}

/// 原本が読み始めたときのままか（名前を付けて保存で、原本が保存先と別のとき・GitHub Issue #126）。
fn source_unchanged<S: SourceBytes<Failure = ErrorKind>>(
    source: &mut S,
) -> Result<(), SaveFailure> {
    if source.unchanged() {
        Ok(())
    } else {
        Err(SaveFailure::ChangedOutside)
    }
}

/// `target` がまだ無いか（リンクも数える）。
fn still_absent(target: &Path) -> Result<(), SaveFailure> {
    match std::fs::symlink_metadata(target) {
        Ok(_) => Err(SaveFailure::ChangedOutside),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(SaveFailure::Target(error.kind())),
    }
}

/// 名前の変更を記憶装置に届ける（Unix）。失敗しても、名前の変更そのものは済んでいるので知らせない。
#[cfg(unix)]
fn sync_folder(folder: &Path) {
    if let Ok(handle) = File::open(folder) {
        drop(handle.sync_all());
    }
}

/// Unix の外ではフォルダを開いて同期できない（`MoveFileExW` は書き込みを待つ）。
#[cfg(not(unix))]
fn sync_folder(_folder: &Path) {}

/// 置き換えに失敗して残った一時ファイルで、もう一度置き換える（「再試行」を選んだとき）。
///
/// # Errors
///
/// 置き換えの失敗（一時ファイルは残る）。
pub fn retry_replace<R: Replacing<Location = Path>>(
    replacing: &R,
    temporary: PathBuf,
    target: &Path,
) -> Result<(), SaveFailure> {
    replace_with(replacing, temporary, target)
}

/// 置き換えに失敗して残った一時ファイルを消す（「やめる」を選んだとき）。消せなくても知らせない。
pub fn discard(temporary: &Path) {
    drop(std::fs::remove_file(temporary));
}

fn replace_with<R: Replacing<Location = Path>>(
    replacing: &R,
    temporary: PathBuf,
    target: &Path,
) -> Result<(), SaveFailure> {
    replacing
        .replace(&temporary, target)
        .map_err(|failure: ReplaceFailure| SaveFailure::Replace { failure, temporary })
}

/// 原本が、開く前に取った印 `opened` のままか（書く前と置き換えの直前に確かめる・FILE-01）。
fn unchanged_since(target: &Path, opened: FileStamp) -> Result<(), SaveFailure> {
    let now = stamp_of(target).map_err(SaveFailure::Target)?;
    if now == opened {
        Ok(())
    } else {
        Err(SaveFailure::ChangedOutside)
    }
}

/// ハードリンクのあるファイル（リンクの数が 2 以上）を拒む（施主の判断 2）。
fn refuse_hard_links(metadata: &std::fs::Metadata) -> Result<(), SaveFailure> {
    let links = hard_links(metadata);
    if links > 1 {
        return Err(SaveFailure::HardLinked(links));
    }
    Ok(())
}

/// リンクの数（Unix）。
#[cfg(unix)]
fn hard_links(metadata: &std::fs::Metadata) -> u64 {
    std::os::unix::fs::MetadataExt::nlink(metadata)
}

/// リンクの数。🔴 Windows の数え方（`GetFileInformationByHandle`）はまだ無い。1 とみなす（実機の確かめと合わせて足す）。
#[cfg(not(unix))]
fn hard_links(_metadata: &std::fs::Metadata) -> u64 {
    1
}

/// 計画の文字列が、すべて文字コード `encoding` で表せるか（書き始める前に止める・SAVE-04）。
fn encodable(plan: &SavePlan, encoding: TextEncoding) -> Result<(), SaveFailure> {
    for step in plan.steps() {
        if let SaveStep::Text(text) = step
            && encoded(text, encoding).is_none()
        {
            return Err(SaveFailure::Unrepresentable(text.clone()));
        }
    }
    Ok(())
}

/// `produce` の渡す歩を一時ファイル `file` に書き、同期する。
fn write_streamed<S, P>(
    file: File,
    encoding: TextEncoding,
    source: &mut S,
    produce: P,
) -> Result<(), SaveFailure>
where
    S: SourceBytes<Failure = ErrorKind>,
    P: FnOnce(&mut dyn PlanSink) -> Result<(), SaveFailure>,
{
    let mut sink = FileSink::new(file, encoding, source);
    let planned = produce(&mut sink);
    sink.finish(planned)
}

/// 原本の権限（Unix の mode）を一時ファイルに写す。写せなくても保存は続ける（所有者が違うときなど）。
#[cfg(unix)]
fn keep_permissions(temporary: &Path, metadata: &std::fs::Metadata) {
    drop(std::fs::set_permissions(temporary, metadata.permissions()));
}

/// Windows では、ACL と属性は `ReplaceFileW` が引き継ぐ（ADR 0025 の決定 5）。
#[cfg(not(unix))]
fn keep_permissions(_temporary: &Path, _metadata: &std::fs::Metadata) {}
