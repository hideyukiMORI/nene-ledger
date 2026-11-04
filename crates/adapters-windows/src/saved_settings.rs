//! 設定のファイルの読み書き（ARC-009）。設定の置き場の `settings.v1.txt`。形は application の `settings_file`（X11 版と同じファイル）。

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use nene_ledger_application::settings::Settings;
use nene_ledger_application::settings_failure::SettingsFailure;
use nene_ledger_application::settings_file;
use nene_ledger_application::settings_port::SettingsPort;

use crate::adapter_failure::AdapterFailure;
use crate::config_directory;

/// いま書く版のファイル名。版は名前にある（ARC-009）。
const FILE_NAME: &str = "settings.v1.txt";

/// 設定を書く、application のポートの実装。設定のファイルは小さいので、丸ごと書き直す。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsStore {
    path: PathBuf,
}

impl SettingsStore {
    /// 設定の置き場の、いま書く版のファイルへ。
    ///
    /// # Errors
    ///
    /// 設定の置き場を決められない。
    pub fn in_config_directory() -> Result<Self, AdapterFailure> {
        Ok(Self::at(config_directory::file_path(FILE_NAME)?))
    }

    /// 道 `path` のファイルへ。
    #[must_use]
    pub fn at(path: PathBuf) -> Self {
        Self { path }
    }

    /// 設定を読む。ファイルが無ければ初期値。読めない形なら失敗（黙って既定値へ落とさない・ARC-009）。
    ///
    /// # Errors
    ///
    /// ファイルが読めない形・読めない。
    pub fn load(&self) -> Result<Settings, AdapterFailure> {
        match fs::read_to_string(&self.path) {
            Ok(text) => settings_file::parse(&text)
                .ok_or_else(|| AdapterFailure::BadSettingsFile(self.path.clone())),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Settings::initial()),
            Err(error) => Err(AdapterFailure::Io(error)),
        }
    }

    /// 書く先の道。
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl SettingsPort for SettingsStore {
    fn save(&mut self, settings: &Settings) -> Result<(), SettingsFailure> {
        write_to(&self.path, settings).map_err(|_failure| SettingsFailure::Unwritable)
    }
}

/// フォルダを作り、設定を丸ごと書く。
fn write_to(path: &Path, settings: &Settings) -> io::Result<()> {
    if let Some(directory) = path.parent() {
        fs::create_dir_all(directory)?;
    }
    fs::write(path, settings_file::encode(settings))
}

#[cfg(test)]
mod tests {
    use super::*;
    use nene_ledger_application::detail_pane::DetailPane;

    /// 試しの置き場。リポジトリの `target/` の下に作る（プロジェクトの外に書かない）。
    fn scratch(name: &str) -> PathBuf {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/adapters-windows-tests")
            .join(name);
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        root
    }

    #[test]
    fn a_missing_file_is_initial_and_saved_settings_come_back() {
        let root = scratch("round-trip");
        let mut store = SettingsStore::at(root.join("nene-ledger").join(FILE_NAME));
        assert_eq!(store.load().unwrap(), Settings::initial());
        let closed = Settings::initial().with_detail(DetailPane::Closed);
        store.save(&closed).unwrap();
        assert_eq!(store.load().unwrap(), closed);
        assert!(store.path().ends_with(FILE_NAME));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn an_unreadable_file_is_a_failure_not_a_default() {
        let root = scratch("bad");
        fs::create_dir_all(&root).unwrap();
        let path = root.join(FILE_NAME);
        fs::write(&path, "version = 9\n").unwrap();
        assert!(matches!(
            SettingsStore::at(path).load(),
            Err(AdapterFailure::BadSettingsFile(_))
        ));
        // 書けない所（ファイルをフォルダとして使う）は失敗として返す。
        let mut unwritable = SettingsStore::at(root.join(FILE_NAME).join("inside.txt"));
        assert_eq!(
            unwritable.save(&Settings::initial()),
            Err(SettingsFailure::Unwritable)
        );
        fs::remove_dir_all(&root).unwrap();
    }
}
