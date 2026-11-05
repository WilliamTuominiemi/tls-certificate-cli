pub struct Certificate {
    pub subject: String,
    pub issuer: String,
    pub validity_period: String,
    pub subject_alternative_name: Option<String>,
    pub basic_constraints: String,
    pub key_usage: String,
    pub extended_key_usage: String,
    pub serial_number: String,
    pub signature_algorithm: String,
    pub public_key_and_algorithm: String,
    pub key_identifiers: KeyIdentifiers,
    pub fingerprint: Fingerprint,
}

pub struct KeyIdentifiers {
    pub ski: String,
    pub aki: String,
}

pub struct Fingerprint {
    pub certificate_fingerprint: String,
    pub spki_pin: String,
}
