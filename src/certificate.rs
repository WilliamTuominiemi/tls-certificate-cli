use chrono::{DateTime, Utc};

pub struct Certificate {
    pub subject: String,
    pub issuer: String,
    pub validity_period: ValidityPeriod,
    pub subject_alternative_name: String,
    pub basic_constraints: String,
    pub key_usage: String,
}

pub struct ValidityPeriod {
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
    pub days_left: i64,
}
