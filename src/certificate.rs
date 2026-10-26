use chrono::{DateTime, Utc};
use x509_parser::prelude::SubjectAlternativeName;

pub struct Certificate<'a> {
    pub subject: String,
    pub issuer: String,
    pub validity_period: ValidityPeriod,
    pub san: Option<SubjectAlternativeName<'a>>,
}

pub struct ValidityPeriod {
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
    pub days_left: i64,
}
