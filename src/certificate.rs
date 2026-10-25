use chrono::{DateTime, Utc};

pub struct Certificate {
    pub subject: String,
    pub issuer: String,
    pub auth: String,
    pub validity_period: ValidityPeriod,
}

#[derive(PartialEq, Debug)]
pub struct ValidityPeriod {
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
    pub days_left: i64,
}
