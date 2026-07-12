use super::{ImportError, ImportFileParser};
use crate::model::{CardIdentity, Id, NewTransaction};
use regex::Regex;
use std::path::Path;
use std::sync::OnceLock;

/// Parses a Barclaycard "Transactions" PDF export (online banking →
/// download transactions as PDF). Chosen over the CSV export because the
/// CSV rounds every amount to whole pounds — this PDF is penny-precise —
/// see doc/kb/barclaycard/pdf-export-structure.md for the full structure
/// write-up and why account identity has to work differently here than for
/// `BarclaysOfxParser`.
///
/// Known gaps, both already true of `GenericCsvParser` for the same
/// underlying reason (no stable per-transaction ID in the source format):
/// - No de-duplication of individual transactions across an overlapping
///   re-export — only whole-file hash dedup applies.
/// - The sign for the `Other` type tag (only ever observed as Barclaycard
///   Cashback, always money in) is an assumption from a small sample, not
///   a documented rule — a genuinely different `Other` transaction could
///   have the wrong sign.
pub struct BarclaycardPdfParser;

impl ImportFileParser for BarclaycardPdfParser {
    fn name(&self) -> &'static str {
        "Barclaycard PDF"
    }

    fn card_identity(&self, path: &Path) -> Result<Option<CardIdentity>, ImportError> {
        let text = extract_text(path)?;
        Ok(parse_card_identity(&text))
    }

    fn balance_snapshot(&self, path: &Path) -> Result<Option<(i64, String)>, ImportError> {
        let text = extract_text(path)?;
        Ok(parse_balance_snapshot(&text))
    }

    fn parse(&self, path: &Path, account_id: Id) -> Result<Vec<NewTransaction>, ImportError> {
        let text = extract_text(path)?;
        Ok(parse_transactions(&text, account_id))
    }
}

fn extract_text(path: &Path) -> Result<String, ImportError> {
    pdf_extract::extract_text(path).map_err(|e| ImportError::Parse(format!("invalid PDF: {e}")))
}

/// Barclaycard's own PDF renderer breaks a mid-transaction line across a
/// page boundary and injects a "Page N of M" footer + stray continuation
/// text right in the middle of it (confirmed against a real export — see
/// the KB article). Stripping these footer lines before pattern-matching
/// keeps a transaction's fields contiguous regardless of where it happens
/// to fall relative to a page break.
fn strip_page_footers(text: &str) -> String {
    text.lines()
        .filter(|line| !is_page_footer(line))
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_page_footer(line: &str) -> bool {
    let line = line.trim();
    let Some(rest) = line.strip_prefix("Page ") else {
        return false;
    };
    let Some((n, m)) = rest.split_once(" of ") else {
        return false;
    };
    n.chars().all(|c| c.is_ascii_digit()) && m.chars().all(|c| c.is_ascii_digit())
}

// Deliberately matched one line at a time (not with a multiline `(?m)`
// anchor across the whole text): `\s+` matches newlines too, so a
// multiline version of this pattern can walk across a line boundary and
// silently pair the wrong line's leading token with this line's trailing
// "VISA 0002" — caught by a real (fictional-data) test fixture.
fn header_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(\S+) (.+) (VISA|MASTERCARD|AMEX|MAESTRO) (\d{4})$").unwrap())
}

fn today_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"Today:\s*(\d{2} [A-Za-z]{3} \d{4})").unwrap())
}

fn balance_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"Current balance:\s*£([\d,]+\.\d{2})").unwrap())
}

/// The one field this export carries that's stable-ish: the card product
/// name and its currently-masked last 4 digits. Not a reliable long-term
/// account key (see the KB article) — `Db::find_or_create_credit_card_account`
/// is responsible for matching it against known card-number history.
fn parse_card_identity(text: &str) -> Option<CardIdentity> {
    let caps = text
        .lines()
        .find_map(|line| header_regex().captures(line.trim()))?;
    // The statement header names the card product ("Barclaycard"), not the
    // institution — Barclaycard is a trading name of Barclays Bank UK, the
    // same institution as the user's other Barclays accounts, so normalise
    // it here rather than showing a distinct "institution" for one account.
    let institution = match &caps[1] {
        "Barclaycard" => "Barclays".to_string(),
        other => other.to_string(),
    };
    Some(CardIdentity {
        institution,
        product_label: caps[2].to_string(),
        last4: caps[4].to_string(),
        currency: "GBP".to_string(),
    })
}

fn parse_balance_snapshot(text: &str) -> Option<(i64, String)> {
    let balance_minor = parse_amount_minor(&balance_regex().captures(text)?[1])?;
    let as_of = parse_date(&today_regex().captures(text)?[1])?;
    Some((balance_minor, as_of))
}

// The date and type tag ("Purchase"/"Payment received"/"Other") are
// adjacent for a given transaction, but pdf-extract doesn't always
// recover them in reading order — a handful of real rows extract as
// "<type> <date>" instead of the usual "<date> <type>" (confirmed against
// a real 205-transaction export: 4 rows came out reversed). Matching
// both orders, rather than assuming one, is what gets every row.
fn transaction_regex() -> &'static Regex {
    const DATE: &str = r"\d{2} (?:Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec) \d{4}";
    const TYPE: &str = r"Purchase|Payment received|Other";
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(&format!(
            r"(?:(?P<date1>{DATE}) (?P<type1>{TYPE})|(?P<type2>{TYPE}) (?P<date2>{DATE})) (?P<desc>.+?) £(?P<amount>[\d,]+\.\d{{2}})"
        ))
        .unwrap()
    })
}

fn parse_transactions(text: &str, account_id: Id) -> Vec<NewTransaction> {
    let normalized = strip_page_footers(text);
    let normalized = normalized.split_whitespace().collect::<Vec<_>>().join(" ");

    transaction_regex()
        .captures_iter(&normalized)
        .filter_map(|caps| {
            let date_str = caps
                .name("date1")
                .or_else(|| caps.name("date2"))
                .unwrap()
                .as_str();
            let type_tag = caps
                .name("type1")
                .or_else(|| caps.name("type2"))
                .unwrap()
                .as_str()
                .to_string();
            let posted_at = parse_date(date_str)?;
            let description = caps["desc"].trim().to_string();
            let magnitude = parse_amount_minor(&caps["amount"])?;
            // "Purchase" is money out; "Payment received" and "Other"
            // (so far only ever seen as Barclaycard Cashback) are money in.
            let amount_minor = if type_tag == "Purchase" {
                -magnitude
            } else {
                magnitude
            };

            Some(NewTransaction {
                account_id,
                import_id: None,
                posted_at,
                amount_minor,
                currency: "GBP".to_string(),
                description: description.clone(),
                raw_description: Some(description),
                trn_type: Some(type_tag),
                external_id: None,
                notes: None,
            })
        })
        .collect()
}

/// "10 Jul 2026" -> "2026-07-10".
fn parse_date(s: &str) -> Option<String> {
    let mut parts = s.split_whitespace();
    let day = parts.next()?;
    let month = parts.next()?;
    let year = parts.next()?;
    let month_num = match month {
        "Jan" => "01",
        "Feb" => "02",
        "Mar" => "03",
        "Apr" => "04",
        "May" => "05",
        "Jun" => "06",
        "Jul" => "07",
        "Aug" => "08",
        "Sep" => "09",
        "Oct" => "10",
        "Nov" => "11",
        "Dec" => "12",
        _ => return None,
    };
    Some(format!("{year}-{month_num}-{day}"))
}

/// "1,378.95" -> 137895.
fn parse_amount_minor(s: &str) -> Option<i64> {
    let cleaned = s.replace(',', "");
    let (pounds, pence) = cleaned.split_once('.')?;
    let pounds: i64 = pounds.parse().ok()?;
    let pence: i64 = pence.parse().ok()?;
    Some(pounds * 100 + pence)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Fictional but format-faithful, matching the real extracted-text shape
    // confirmed against a real Barclaycard PDF export (see the KB article),
    // including the mid-transaction page-break artifact.
    const SAMPLE_TEXT: &str = "
Today: 12 Jul 2026 MR A SAMPLE

Transactions

Barclaycard Rewards VISA 0002

Current balance: £613.73

Available credit: £10,086.27

Credit limit: £10,700.00

Showing transactions from 01/01/2026 to 12/07/202612

Date Description Money in Money out

10 Jul 2026
 Purchase
ANTHROPIC Anthropic,
Anthropic.com        12.71 POUND
STERLING USA
 £12.71

15 Jun 2026 Payment received
PAYMENT, THANK YOU
Payment, Thank You £453.77

10 Mar 2026 Payment received
PAYMENT, THANK YOU £76.00
 Page 10 of 21

Payment, Thank You

08 Apr 2026
 Other
Barclaycard Cashback
Barclaycard Cashback        1.0E-4%
CASH REBATE THIS MONTH
 £2.69

05 Jan 2026 Purchase
AMZNMktplace Amznmktplace,
Amazon.co.uk £86.99

Purchase 06 May 2026 AMZNMktplace*N67WD25U4
Amznmktplace*n67wd25u4, Amazon.co.uk £19.98

Need to view older transactions?
";

    #[test]
    fn parses_card_identity_from_header() {
        let identity = parse_card_identity(SAMPLE_TEXT).expect("card identity");
        assert_eq!(identity.institution, "Barclays");
        assert_eq!(identity.product_label, "Rewards");
        assert_eq!(identity.last4, "0002");
        assert_eq!(identity.currency, "GBP");
    }

    #[test]
    fn parses_balance_snapshot_from_header() {
        let (balance_minor, as_of) = parse_balance_snapshot(SAMPLE_TEXT).expect("balance snapshot");
        assert_eq!(balance_minor, 61373);
        assert_eq!(as_of, "2026-07-12");
    }

    #[test]
    fn parses_transactions_with_correct_signs_and_dates() {
        let txs = parse_transactions(SAMPLE_TEXT, 1);
        assert_eq!(txs.len(), 6);

        assert_eq!(txs[0].posted_at, "2026-07-10");
        assert_eq!(txs[0].amount_minor, -1271, "Purchase is money out");
        assert_eq!(txs[0].trn_type.as_deref(), Some("Purchase"));
        assert!(txs[0].description.starts_with("ANTHROPIC"));

        assert_eq!(txs[1].posted_at, "2026-06-15");
        assert_eq!(txs[1].amount_minor, 45377, "Payment received is money in");

        assert_eq!(txs[4].posted_at, "2026-01-05");
        assert_eq!(txs[4].amount_minor, -8699);
    }

    #[test]
    fn parses_a_transaction_whose_type_tag_was_extracted_before_its_date() {
        // A real quirk of pdf-extract's text ordering, not a made-up edge
        // case — see the module doc comment on `transaction_regex`.
        let txs = parse_transactions(SAMPLE_TEXT, 1);
        let reversed_order = txs
            .iter()
            .find(|t| t.posted_at == "2026-05-06")
            .expect("the type-before-date transaction");
        assert_eq!(reversed_order.amount_minor, -1998);
        assert_eq!(reversed_order.trn_type.as_deref(), Some("Purchase"));
        assert!(reversed_order
            .description
            .starts_with("AMZNMktplace*N67WD25U4"));
    }

    #[test]
    fn survives_a_transaction_split_across_a_page_break() {
        // The "10 Mar 2026 Payment received ... £76.00" row is followed
        // immediately by a "Page 10 of 21" footer and an orphaned
        // continuation line ("Payment, Thank You") before the next real
        // transaction — must parse as exactly one transaction, not merge
        // into or corrupt its neighbours.
        let txs = parse_transactions(SAMPLE_TEXT, 1);
        let split_txn = txs
            .iter()
            .find(|t| t.posted_at == "2026-03-10")
            .expect("the page-break-split transaction");
        assert_eq!(split_txn.amount_minor, 7600);

        let cashback = txs
            .iter()
            .find(|t| t.trn_type.as_deref() == Some("Other"))
            .expect("cashback transaction");
        assert_eq!(cashback.amount_minor, 269, "cashback is money in");
    }

    #[test]
    fn parse_amount_minor_strips_thousands_separators() {
        assert_eq!(parse_amount_minor("1,378.95"), Some(137895));
        assert_eq!(parse_amount_minor("76.00"), Some(7600));
    }
}
