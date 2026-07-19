//! Persistent user configuration, stored as TOML in the platform's standard
//! config directory.

use crate::model::Account;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    /// Directory ledgr scans for downloaded import files.
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

    /// Reference household accounts (e.g. a partner's) — known by sort
    /// code/account number only, never imported, never given a balance or
    /// transaction history — so spend ledger derivation recognises
    /// transfers to them as internal rather than spend. Imported accounts
    /// are household members automatically and don't need listing here.
    /// See ADR 0008 and the "Account registry" section of
    /// doc/implementation-notes/spend-ledger-design.md. Hand-edit the
    /// config file to add one; no CLI command yet.
    #[serde(default)]
    pub household_accounts: Vec<HouseholdAccountRef>,

    /// Registered external payers (an employer, a tax authority) driving a
    /// high-confidence Income classification — see the **Income Source**
    /// ubiquitous-language entry. Hand-edit the config file to add one; no
    /// CLI command yet.
    #[serde(default)]
    pub income_sources: Vec<IncomeSourceRef>,

    /// Registered external individuals (family/friends) so their payments
    /// classify consistently — see the **Registered Person**
    /// ubiquitous-language entry. An unexplained inbound payment from a
    /// registered person defaults to a spend-ledger reimbursement, not
    /// income. Hand-edit the config file to add one; no CLI command yet.
    #[serde(default)]
    pub registered_people: Vec<RegisteredPersonRef>,

    /// Registered external institutions/schemes (e.g. a health cash plan
    /// like SimplyHealth) whose payouts default to a spend-ledger
    /// reimbursement rather than income — the non-person counterpart to
    /// `registered_people`. `kind` is free text for display only (e.g.
    /// `"Health Scheme"`) — behaviour is identical regardless of kind.
    /// Hand-edit the config file to add one; no CLI command yet.
    #[serde(default)]
    pub reimbursement_sources: Vec<ReimbursementSourceRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HouseholdAccountRef {
    pub sort_code: String,
    pub account_number: String,
    /// Free-text label for the user's own reference (e.g. "Partner's
    /// account") — not used for matching.
    #[serde(default)]
    pub label: Option<String>,
    /// The household member's full name (e.g. `"ROMINA SCARAMAGLI"`), used
    /// to recognise a person-to-person `NAME` field that carries no sort
    /// code/account number at all — Barclays shows these as either the full
    /// name (when you're paying them, from your saved payee nickname) or
    /// `"<Surname> <First initial>"` (when they're paying you, the sender
    /// name Faster Payments echoes back) — see
    /// `crate::derive::matches_household_member_name`. `None` means this
    /// entry is only matched by sort code/account number, as before.
    #[serde(default)]
    pub name: Option<String>,
}

/// Truncation-tolerant match of `sort`/`account` (as decoded from a `NAME`
/// field) against a list of Reference Household Accounts — shared by
/// `Config::household_account_matches` and `Db::monthly_transfer_totals`
/// (which only has the account list, not a whole `Config`, to hand).
pub fn household_accounts_contain(accounts: &[HouseholdAccountRef], sort: &str, account: &str) -> bool {
    accounts.iter().any(|a| {
        a.sort_code == sort
            && (a.account_number == account || a.account_number.starts_with(account))
    })
}

/// A registered external payer — see `Config::income_sources`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncomeSourceRef {
    /// Matched at the very start of a transaction's description (e.g. the
    /// payroll processor's own name, `"AZIMO LTD"`, or `"HMRC PAYE"`),
    /// same word-boundary matching as `RegisteredPersonRef`/
    /// `HouseholdAccountRef.name`.
    pub name: String,
    pub kind: IncomeSourceKind,
    /// Free-text label for the user's own reference (e.g. the actual
    /// employer's name, distinct from the payroll processor's name that
    /// `name` matches on) — shown in `ledgr status`, not used for matching.
    #[serde(default)]
    pub label: Option<String>,
    /// The entity's true/full proper name (e.g. `"Pleo Technologies"`,
    /// distinct from `name`, which may be a payment processor's name or a
    /// truncated form chosen for matching) — shown in `ledgr status`, not
    /// used for matching. `None` when nothing more specific than `name`/
    /// `label` is known.
    #[serde(default)]
    pub full_name: Option<String>,
}

/// What kind of Income Source this is — drives the classification rule and
/// confidence applied in `derive::classify`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncomeSourceKind {
    Salary,
    TaxAuthority,
    Prize,
}

impl IncomeSourceKind {
    pub fn rule_name(&self) -> &'static str {
        match self {
            IncomeSourceKind::Salary => "employment_income",
            IncomeSourceKind::TaxAuthority => "tax_refund",
            IncomeSourceKind::Prize => "prize_win",
        }
    }

    pub fn confidence(&self) -> f64 {
        match self {
            IncomeSourceKind::Salary => 0.95,
            IncomeSourceKind::TaxAuthority => 0.8,
            IncomeSourceKind::Prize => 0.9,
        }
    }

    pub fn display(&self) -> &'static str {
        match self {
            IncomeSourceKind::Salary => "Salary",
            IncomeSourceKind::TaxAuthority => "Tax Authority",
            IncomeSourceKind::Prize => "Prizes",
        }
    }
}

/// A registered external individual — see `Config::registered_people`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisteredPersonRef {
    /// Matched against the start of a transaction's description, same
    /// word-boundary matching as `HouseholdAccountRef.name` — see
    /// `derive::matches_person_name`.
    pub name: String,
    /// Free-text label for the user's own reference (e.g. "Ma") — shown in
    /// `ledgr status`, not used for matching.
    #[serde(default)]
    pub label: Option<String>,
    /// The person's true/full name, when `name` is a truncated form chosen
    /// for matching (e.g. Barclays' 32-char `NAME` cap cutting a long
    /// surname short) — shown in `ledgr status`, not used for matching.
    /// `None` when `name` already is the full name.
    #[serde(default)]
    pub full_name: Option<String>,
}

/// A registered external institution/scheme — see
/// `Config::reimbursement_sources`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReimbursementSourceRef {
    /// Matched at the very start of a transaction's description, same
    /// word-boundary matching as `IncomeSourceRef.name`.
    pub name: String,
    /// Free text describing what kind of entity this is (e.g. `"Health
    /// Scheme"`) — display only, doesn't affect classification.
    pub kind: String,
    /// Free-text label for the user's own reference — shown in
    /// `ledgr status`, not used for matching.
    #[serde(default)]
    pub label: Option<String>,
    /// The entity's true/full proper name — shown in `ledgr status`, not
    /// used for matching.
    #[serde(default)]
    pub full_name: Option<String>,
}

impl Config {
    /// Whether `sort`/`account` (as decoded from a `NAME` field, possibly
    /// truncated) identifies one of the configured Reference Household
    /// Accounts. Same truncation-tolerant match as
    /// `derive::household_contains` (a stored account number may be longer
    /// than the truncated digits observed on a statement), scoped to just
    /// the reference accounts rather than the full tracked+reference
    /// household set, so callers can tell "this leg's counterpart is a
    /// reference account, permanently unpairable by design" apart from
    /// "no counterpart found at all".
    pub fn household_account_matches(&self, sort: &str, account: &str) -> bool {
        household_accounts_contain(&self.household_accounts, sort, account)
    }

    /// Registers a new Registered Person — see the TUI's `a` "add reference"
    /// form on `Screen::IncomeMonth`, which builds one from an
    /// otherwise-unrecognised inbound payment's description.
    pub fn add_registered_person(&mut self, person: RegisteredPersonRef) {
        self.registered_people.push(person);
    }

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
                household_accounts: Vec::new(),
                income_sources: Vec::new(),
                registered_people: Vec::new(),
                reimbursement_sources: Vec::new(),
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
    pub fn apply_account_name_overrides<'a>(
        &self,
        accounts: impl IntoIterator<Item = &'a mut Account>,
    ) {
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
            household_accounts: Vec::new(),
            income_sources: Vec::new(),
            registered_people: Vec::new(),
            reimbursement_sources: Vec::new(),
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
            household_accounts: Vec::new(),
            income_sources: Vec::new(),
            registered_people: Vec::new(),
            reimbursement_sources: Vec::new(),
        };
        config.set_account_name("5678", "Jim's Account");

        let mut accounts = vec![Account {
            id: 1,
            name: "Barclays Current Account (...5678)".into(),
            institution: Some("Barclays".into()),
            account_type: crate::model::AccountType::Current,
            currency: "GBP".into(),
            sort_code: None,
            account_number: None,
        }];

        config.apply_account_name_overrides(&mut accounts);

        assert_eq!(accounts[0].name, "Jim's Account");
    }
}
