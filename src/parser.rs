use crate::certificate::{Certificate, ValidityPeriod};

use chrono::{DateTime, Utc};
use x509_parser::certificate::X509Certificate;
use x509_parser::time::ASN1Time;

pub struct Parser<'a> {
    x509_certificates: Vec<X509Certificate<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(x509_certificates: Vec<X509Certificate<'a>>) -> Self {
        Parser { x509_certificates }
    }

    pub fn parse_certificates(&self) -> Result<Vec<Certificate<'_>>, Box<dyn std::error::Error>> {
        let mut certificates = Vec::new();

        for x509_cert in &self.x509_certificates {
            let subject = x509_cert.subject().to_string();

            let issuer = x509_cert.issuer().to_string();

            let not_before = self.asn1_time_to_chrono(&x509_cert.validity().not_before);
            let not_after = self.asn1_time_to_chrono(&x509_cert.validity().not_after);
            let days_left = (not_after - chrono::offset::Utc::now()).num_days();
            let validity_period = ValidityPeriod {
                not_before,
                not_after,
                days_left,
            };

            let subject_alternative_name = x509_cert.subject_alternative_name()?;
            let san = subject_alternative_name.map(|extension| extension.value.clone());

            let certificate = Certificate {
                subject,
                issuer,
                validity_period,
                san,
            };
            certificates.push(certificate);
        }

        Ok(certificates)
    }

    fn asn1_time_to_chrono(&self, asn1: &ASN1Time) -> DateTime<Utc> {
        let seconds = asn1.timestamp();
        DateTime::from_timestamp(seconds, 0).unwrap()
    }
}
