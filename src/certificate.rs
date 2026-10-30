pub struct Certificate {
    pub subject: String,
    pub issuer: String,
    pub validity_period: String,
    pub subject_alternative_name: Option<String>,
    pub basic_constraints: String,
    pub key_usage: String,
    pub serial_number: String,
    pub signature_algorithm: String,
}
