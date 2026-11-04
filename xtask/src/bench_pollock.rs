//! Pollock の全ファイルに `nene-ledger --dump` をかけ、Pollock の約束の置き場に出力と時間を書く（QLT-014・Issue #24）。
//!
//! 採点は Pollock の `evaluate.py` が行う（記録の形は `docs/quality/benchmarks.md`）。ここは走らせるだけ。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

/// Pollock の結果の置き場での、この製品の名前。
const SUT: &str = "nene";

/// 読み込めなかったときに、Pollock が失敗として数える出力の先頭の行。
const APPLICATION_ERROR: &[u8] = b"Application Error\n";

/// Pollock の `polluted_files/csv` の全ファイルを `binary --dump` にかけ、出力と時間を書く。報告の行を返す。
pub(crate) fn run(pollock: &Path, binary: &Path) -> Result<Vec<String>, String> {
    let output = pollock.join("results").join(SUT).join("polluted_files");
    let loading = output.join("loading");
    fs::create_dir_all(&loading).map_err(|error| format!("{}: {error}", loading.display()))?;
    let files = inputs(&pollock.join("polluted_files").join("csv"))?;
    let mut times = vec![format!("filename,{SUT}_time_0")];
    let mut failures = 0_usize;
    for file in &files {
        let name = file
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        let started = Instant::now();
        let result = Command::new(binary)
            .arg("--dump")
            .arg(file)
            .output()
            .map_err(|error| format!("{}: {error}", binary.display()))?;
        let seconds = started.elapsed().as_secs_f64();
        let body = if result.status.success() {
            result.stdout
        } else {
            failures = failures.saturating_add(1);
            [APPLICATION_ERROR, &result.stderr].concat()
        };
        let converted = loading.join(converted_name(&name));
        fs::write(&converted, body).map_err(|error| format!("{}: {error}", converted.display()))?;
        times.push(format!("{name},{seconds}"));
    }
    let time_file = output.join(format!("{SUT}_time.csv"));
    fs::write(&time_file, times.join("\n") + "\n")
        .map_err(|error| format!("{}: {error}", time_file.display()))?;
    Ok(vec![format!(
        "bench-pollock: {} ファイル・読み込めなかった {failures}・出力 {}",
        files.len(),
        loading.display()
    )])
}

/// 入力の CSV（名前の順）。
fn inputs(directory: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files: Vec<PathBuf> = fs::read_dir(directory)
        .map_err(|error| format!("{}: {error}", directory.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "csv"))
        .collect();
    files.sort();
    Ok(files)
}

/// Pollock が採点のために読む出力の名前。
fn converted_name(name: &str) -> String {
    format!("{name}_converted.csv")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outputs_follow_the_pollock_naming() {
        assert_eq!(
            converted_name("file_no_header.csv"),
            "file_no_header.csv_converted.csv"
        );
        assert!(APPLICATION_ERROR.starts_with(b"Application Error"));
    }
}
