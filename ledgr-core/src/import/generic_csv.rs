use super::{ImportError, StatementParser};
use crate::model::{Id, NewTransaction};
use std::path::Path;

/// Placeholder parser for a generic `date,description,amount` CSV, in the
/// currency given at construction time. Real bank export formats (which
/// vary a lot in column layout, date format, and sign conventions) get
/// their own parsers alongside this one.
pub struct GenericCsvParser {
    pub currency: String,
}

impl StatementParser for GenericCsvParser {
    fn name(&self) -> &'static str {
        "Generic CSV"
    }

    fn parse(&self, path: &Path, account_id: Id) -> Result<Vec<NewTransaction>, ImportError> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_path(path)?;

        let mut transactions = Vec::new();
        for record in reader.records() {
            let record = record?;
            let posted_at = record
                .get(0)
                .ok_or_else(|| ImportError::Parse("missing date column".into()))?
                .to_string();
            let description = record
                .get(1)
                .ok_or_else(|| ImportError::Parse("missing description column".into()))?
                .to_string();
            let amount_str = record
                .get(2)
                .ok_or_else(|| ImportError::Parse("missing amount column".into()))?;
            let amount_minor = parse_amount_minor(amount_str)
                .ok_or_else(|| ImportError::Parse(format!("invalid amount: {amount_str}")))?;

            transactions.push(NewTransaction {
                account_id,
                statement_id: None,
                posted_at,
                amount_minor,
                currency: self.currency.clone(),
                description: description.clone(),
                raw_description: Some(description),
                category_id: None,
                external_id: None,
            });
        }
        Ok(transactions)
    }
}

impl From<csv::Error> for ImportError {
    fn from(e: csv::Error) -> Self {
        ImportError::Parse(e.to_string())
    }
}

/// Parses a decimal amount string like `"-25.99"` into signed minor units
/// (e.g. pence), avoiding floating point.
fn parse_amount_minor(s: &str) -> Option<i64> {
    let s = s.trim();
    let negative = s.starts_with('-');
    let s = s.trim_start_matches(['-', '+']);
    let mut parts = s.splitn(2, '.');
    let whole: i64 = parts.next()?.parse().ok()?;
    let frac_str = parts.next().unwrap_or("0");
    let frac_str = if frac_str.len() >= 2 {
        &frac_str[..2]
    } else {
        frac_str
    };
    let frac: i64 = format!("{frac_str:0<2}").parse().ok()?;
    let minor = whole * 100 + frac;
    Some(if negative { -minor } else { minor })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parses_generic_csv() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("statement.csv");
        let mut file = std::fs::File::create(&path).expect("create file");
        writeln!(file, "date,description,amount").unwrap();
        writeln!(file, "2026-07-01,Tesco Stores,-25.99").unwrap();
        writeln!(file, "2026-07-02,Salary,1500").unwrap();

        let parser = GenericCsvParser {
            currency: "GBP".into(),
        };
        let txs = parser.parse(&path, 1).expect("parse");
        assert_eq!(txs.len(), 2);
        assert_eq!(txs[0].amount_minor, -2599);
        assert_eq!(txs[1].amount_minor, 150000);
    }

    #[test]
    fn parse_amount_minor_handles_signs_and_fractions() {
        assert_eq!(parse_amount_minor("-25.99"), Some(-2599));
        assert_eq!(parse_amount_minor("25.9"), Some(2590));
        assert_eq!(parse_amount_minor("25"), Some(2500));
        assert_eq!(parse_amount_minor("+10.5"), Some(1050));
    }
}
