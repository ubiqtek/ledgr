//! Persistent user configuration, stored as TOML in the platform's standard
//! config directory.

use crate::model::Account;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    /// Directory ledgr scans for downloaded statement files to import.
    /// A `processed` subdirectory is created inside it automatically, and
    /// imported files are moved there so they aren't picked up again.
    pub inbox_dir: PathBuf,

    /// User-chosen display names, keyed by the last 4 digits of the account
    /// number (as they appear in the bank-generated account name, e.g.
    /// `"Barclays Current Account (...5678)"` -> key `"5678"`). Set via
    /// `ledgr name-account <last4> "<name>"`. Kept here rather than in the
    /// database so renaming an account never risks breaking the
    /// institution/name match `find_or_create_account` uses to avoid
    /// duplicating accounts on re-import.
    #[serde(default)]
    pub account_names: BTreeMap<String, String>,
}

impl Config {
    /// Path to the config file, XDG-style: `~/.config/ledgr/config.toml` on
    /// every platform (deliberately not the platform-native config
    /// directory — CLI users expect `~/.config`, not
    /// `~/Library/Application Support` on macOS).
    pub fn default_path() -> anyhow::Result<PathBuf> {
        Ok(base_dirs()?.home_dir().join(".config/ledgr/config.toml"))
    }

    /// Loads the config from `path`, writing a default one first if it
    /// doesn't exist yet (pointing `inbox_dir` at `~/.config/ledgr/inbox`,
    /// which the user can then edit to point somewhere else, e.g. a synced
    /// Google Drive folder).
    pub fn load_or_init(path: &Path) -> anyhow::Result<Self> {
        if !path.exists() {
            let default = Config {
                inbox_dir: base_dirs()?.home_dir().join(".config/ledgr/inbox"),
                account_names: BTreeMap::new(),
            };
            default.save(path)?;
            return Ok(default);
        }
        let contents = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&contents)?)
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, toml::to_string_pretty(self)?)?;
        Ok(())
    }

    /// Records a display name for the account whose bank-generated name ends
    /// in `(...last4)`, overriding it wherever accounts are shown.
    pub fn set_account_name(&mut self, last4: &str, name: &str) {
        self.account_names
            .insert(last4.to_string(), name.to_string());
    }

    /// Rewrites `name` on every account for which a display-name override is
    /// configured, matched by the last 4 digits embedded in the
    /// bank-generated name (e.g. `"...5678)"`).
    pub fn apply_account_name_overrides<'a>(&self, accounts: impl IntoIterator<Item = &'a mut Account>) {
        if self.account_names.is_empty() {
            return;
        }
        for account in accounts {
            if let Some(last4) = last4_from_account_name(&account.name) {
                if let Some(custom_name) = self.account_names.get(last4) {
                    account.name = custom_name.clone();
                }
            }
        }
    }
}

/// Extracts the last 4 digits from a bank-generated account name of the form
/// `"... (...5678)"`, as produced by e.g. `BarclaysOfxParser`.
fn last4_from_account_name(name: &str) -> Option<&str> {
    let (_, suffix) = name.rsplit_once("(...")?;
    suffix.strip_suffix(')')
}

fn base_dirs() -> anyhow::Result<directories::BaseDirs> {
    directories::BaseDirs::new()
        .ok_or_else(|| anyhow::anyhow!("could not determine the home directory for this platform"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_or_init_writes_a_default_when_missing() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");

        let config = Config::load_or_init(&path).expect("load_or_init");
        assert!(path.exists());
        assert!(config.inbox_dir.ends_with("inbox"));
    }

    #[test]
    fn load_or_init_reads_an_existing_config() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        let inbox_dir = dir.path().join("my-inbox");
        Config {
            inbox_dir: inbox_dir.clone(),
            account_names: BTreeMap::new(),
        }
        .save(&path)
        .expect("save");

        let config = Config::load_or_init(&path).expect("load_or_init");
        assert_eq!(config.inbox_dir, inbox_dir);
    }

    #[test]
    fn apply_account_name_overrides_matches_on_last4() {
        let mut config = Config {
            inbox_dir: PathBuf::from("/tmp/inbox"),
            account_names: BTreeMap::new(),
        };
        config.set_account_name("5678", "Jim's Account");

        let mut accounts = vec![Account {
            id: 1,
            name: "Barclays Current Account (...5678)".into(),
            institution: Some("Barclays".into()),
            account_type: crate::model::AccountType::Checking,
            currency: "GBP".into(),
        }];

        config.apply_account_name_overrides(&mut accounts);

        assert_eq!(accounts[0].name, "Jim's Account");
    }
}
