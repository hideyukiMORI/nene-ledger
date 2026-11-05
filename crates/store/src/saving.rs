//! 保存の手順（ADR 0025 の決定 1）。

use std::fs::File;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use nene_ledger_application::replace_failure::ReplaceFailure;
use nene_ledger_application::replacing::Replacing;
use nene_ledger_application::source_bytes::SourceBytes;
use nene_ledger_core::save_plan::SavePlan;
use nene_ledger_core::save_step::SaveStep;
use nene_ledger_core::text_encoding::TextEncoding;

use crate::encoding::encoded;
use crate::save_failure::SaveFailure;
use crate::temporary_file::create_beside;

/// 原本を写すときに一度に読む量。
const COPY_CHUNK: usize = 1 << 20;

/// 保存の計画 `plan`（書き換えた文字列は原本の文字コード `encoding` で書く）を、`target` と同じフォルダの一時ファイルに書き、同期してから `replacing` で置き換える。
///
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
    (plan, encoding): (&SavePlan, TextEncoding),
    source: &mut S,
    replacing: &R,
) -> Result<(), SaveFailure>
where
    S: SourceBytes<Failure = ErrorKind>,
    R: Replacing<Location = Path>,
{
    let target =
        std::fs::canonicalize(target).map_err(|error| SaveFailure::Target(error.kind()))?;
    let metadata = std::fs::metadata(&target).map_err(|error| SaveFailure::Target(error.kind()))?;
    refuse_hard_links(&metadata)?;
    let texts = encoded_texts(plan, encoding)?;
    let (file, temporary) = create_beside(&target).map_err(SaveFailure::Write)?;
    if let Err(failure) = write_plan(file, plan, &texts, source) {
        discard(&temporary);
        return Err(failure);
    }
    keep_permissions(&temporary, &metadata);
    replace_with(replacing, temporary, &target)
}

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

/// 計画の文字列を、書き始める前に全部符号化する。
fn encoded_texts(plan: &SavePlan, encoding: TextEncoding) -> Result<Vec<Vec<u8>>, SaveFailure> {
    plan.steps()
        .iter()
        .filter_map(|step| match step {
            SaveStep::Text(text) => Some(
                encoded(text, encoding).ok_or_else(|| SaveFailure::Unrepresentable(text.clone())),
            ),
            SaveStep::Copy { .. } => None,
        })
        .collect()
}

/// 計画どおりに一時ファイルへ書き、同期する。
fn write_plan<S: SourceBytes<Failure = ErrorKind>>(
    mut file: File,
    plan: &SavePlan,
    texts: &[Vec<u8>],
    source: &mut S,
) -> Result<(), SaveFailure> {
    let mut buffer = vec![0_u8; COPY_CHUNK];
    let mut texts = texts.iter();
    for step in plan.steps() {
        match step {
            SaveStep::Copy { offset, length } => {
                copy_range(&mut file, source, &mut buffer, (*offset, *length))?;
            }
            SaveStep::Text(_) => {
                let bytes = texts
                    .next()
                    .ok_or(SaveFailure::Write(ErrorKind::InvalidData))?;
                file.write_all(bytes)
                    .map_err(|error| SaveFailure::Write(error.kind()))?;
            }
        }
    }
    file.sync_all()
        .map_err(|error| SaveFailure::Write(error.kind()))
}

/// 原本の `offset` から `length` バイトを写す。
fn copy_range<S: SourceBytes<Failure = ErrorKind>>(
    file: &mut File,
    source: &mut S,
    buffer: &mut [u8],
    (offset, length): (u64, u64),
) -> Result<(), SaveFailure> {
    let mut done = 0_u64;
    while done < length {
        let wanted = usize::try_from(length.saturating_sub(done)).unwrap_or(usize::MAX);
        let chunk = buffer.get_mut(..wanted.min(COPY_CHUNK)).unwrap_or_default();
        let read = match source.read_at(offset.saturating_add(done), chunk) {
            Ok(0) => return Err(SaveFailure::SourceShrank),
            Ok(read) => read,
            Err(ErrorKind::Interrupted) => continue,
            Err(kind) => return Err(SaveFailure::Read(kind)),
        };
        let written = chunk.get(..read).unwrap_or_default();
        file.write_all(written)
            .map_err(|error| SaveFailure::Write(error.kind()))?;
        done = done.saturating_add(u64::try_from(read).unwrap_or(u64::MAX));
    }
    Ok(())
}

/// 原本の権限（Unix の mode）を一時ファイルに写す。写せなくても保存は続ける（所有者が違うときなど）。
#[cfg(unix)]
fn keep_permissions(temporary: &Path, metadata: &std::fs::Metadata) {
    drop(std::fs::set_permissions(temporary, metadata.permissions()));
}

/// Windows では、ACL と属性は `ReplaceFileW` が引き継ぐ（ADR 0025 の決定 5）。
#[cfg(not(unix))]
fn keep_permissions(_temporary: &Path, _metadata: &std::fs::Metadata) {}
