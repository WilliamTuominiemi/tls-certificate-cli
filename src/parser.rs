use crate::certificate::Certificate;

use chrono::{DateTime, Utc};
use x509_parser::certificate::{Validity, X509Certificate};
use x509_parser::oid_registry::{OidRegistry, format_oid};
use x509_parser::public_key::PublicKey;
use x509_parser::time::ASN1Time;
use x509_parser::x509::SubjectPublicKeyInfo;

pub struct Parser<'a> {
    x509_certificates: Vec<X509Certificate<'a>>,
    oid_registry: OidRegistry<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(x509_certificates: Vec<X509Certificate<'a>>) -> Self {
        Parser {
            x509_certificates,
            oid_registry: OidRegistry::default().with_all_crypto().with_x509(),
        }
    }

    pub fn parse_certificates(&self) -> Result<Vec<Certificate>, Box<dyn std::error::Error>> {
        let mut certificates = Vec::new();

        for x509_cert in &self.x509_certificates {
            let subject = x509_cert.subject().to_string();

            let issuer = x509_cert.issuer().to_string();

            let validity_period =
                self.parse_validity_period(&x509_cert.validity, chrono::offset::Utc::now());

            let subject_alternative_name = self.parse_subject_alternative_name(
                x509_cert
                    .subject_alternative_name()?
                    .map(|extension| extension.value.clone()),
            );

            let basic_constraints = self.parse_basic_constraints(
                x509_cert
                    .basic_constraints()?
                    .map(|extension| extension.value.clone()),
            );

            let key_usage = x509_cert
                .key_usage()?
                .map(|extension| *extension.value)
                .map_or_else(|| "".to_string(), |k_u| format!("Key usage: {}", k_u));

            let serial_number = x509_cert.raw_serial_as_string();

            let signature_algorithm =
                format_oid(&x509_cert.signature_algorithm.algorithm, &self.oid_registry);

            let public_key_and_algorithm =
                self.parse_public_key_and_algorithm(x509_cert.public_key());

            let certificate = Certificate {
                subject,
                issuer,
                validity_period,
                subject_alternative_name,
                basic_constraints,
                key_usage,
                serial_number,
                signature_algorithm,
                public_key_and_algorithm,
            };
            certificates.push(certificate);
        }

        Ok(certificates)
    }

    fn asn1_time_to_chrono(&self, asn1: &ASN1Time) -> DateTime<Utc> {
        let seconds = asn1.timestamp();
        DateTime::from_timestamp(seconds, 0).unwrap()
    }

    fn parse_basic_constraints(
        &self,
        basic_constraints: Option<x509_parser::prelude::BasicConstraints>,
    ) -> String {
        if let Some(basic_constraints) = basic_constraints {
            if basic_constraints.ca {
                let path_len = match basic_constraints.path_len_constraint {
                    Some(len) => len.to_string(),
                    None => "unlimited".to_string(),
                };
                format!("CA: true, Path length: {}", path_len).to_string()
            } else {
                "CA: false".to_string()
            }
        } else {
            "BasicConstraints not found".to_string()
        }
    }

    fn parse_subject_alternative_name(
        &self,
        subject_alternative_name: Option<x509_parser::prelude::SubjectAlternativeName<'_>>,
    ) -> Option<String> {
        if let Some(san) = subject_alternative_name {
            let names = san
                .general_names
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(", ");

            Some(format!("SAN ({}): {}", san.general_names.len(), names).to_string())
        } else {
            None
        }
    }

    fn parse_validity_period(&self, validity: &Validity, time_now: DateTime<Utc>) -> String {
        let not_before = self.asn1_time_to_chrono(&validity.not_before);
        let not_after = self.asn1_time_to_chrono(&validity.not_after);
        let days_left = (not_after - time_now).num_days();
        let days_left_message = if days_left >= 0 {
            format!("{} days left", days_left).to_string()
        } else {
            "Expired".to_string()
        };

        let s = format!(
            "{} - {} ({})",
            not_before.format("%d/%m/%Y"),
            not_after.format("%d/%m/%Y"),
            days_left_message
        );
        s
    }

    fn parse_public_key_and_algorithm(
        &self,
        subject_public_key_info: &SubjectPublicKeyInfo,
    ) -> String {
        let algorithm = format_oid(
            &subject_public_key_info.algorithm.algorithm,
            &self.oid_registry,
        );

        let key_description = match subject_public_key_info.parsed() {
            Ok(PublicKey::RSA(rsa)) => format!(
                "RSA {} bits, exponent {}",
                rsa.key_size(),
                rsa.try_exponent()
                    .map(|e| e.to_string())
                    .unwrap_or_default()
            ),
            Ok(PublicKey::EC(point)) => format!("EC {} bits", point.key_size()),
            Ok(PublicKey::DSA(y)) => format!("DSA {} bytes", y.len()),
            Ok(other) => format!("{:?}", other),
            Err(e) => format!("unparseable key: {}", e),
        };

        let s = format!("{} ({})", algorithm, key_description);
        s
    }
}

#[cfg(test)]
mod tests {
    use std::assert_eq;

    use super::*;
    use x509_parser::oid_registry::OID_PKCS1_SHA256WITHRSA;
    use x509_parser::oid_registry::asn1_rs::BitString;
    use x509_parser::{
        extensions::{BasicConstraints, GeneralName, SubjectAlternativeName},
        x509::{AlgorithmIdentifier, SubjectPublicKeyInfo},
    };

    #[test]
    fn test_parse_basic_constraints_with_ca_true_and_path_length() {
        let parser = Parser::new(vec![]);

        let basic_constraints = BasicConstraints {
            ca: true,
            path_len_constraint: Some(2),
        };

        let output = parser.parse_basic_constraints(Some(basic_constraints));

        assert_eq!(output, "CA: true, Path length: 2")
    }

    #[test]
    fn test_parse_basic_constraints_with_ca_false() {
        let parser = Parser::new(vec![]);

        let basic_constraints = BasicConstraints {
            ca: false,
            path_len_constraint: None,
        };

        let output = parser.parse_basic_constraints(Some(basic_constraints));

        assert_eq!(output, "CA: false")
    }

    #[test]
    fn test_parse_subject_alternative_name() {
        let parser = Parser::new(vec![]);

        let subject_alternative_name = SubjectAlternativeName {
            general_names: vec![
                GeneralName::DNSName("example.com"),
                GeneralName::DNSName("www.example.com"),
            ],
        };

        let output = parser
            .parse_subject_alternative_name(Some(subject_alternative_name))
            .unwrap();

        assert_eq!(
            output,
            "SAN (2): DNSName(example.com), DNSName(www.example.com)"
        );
    }

    #[test]
    fn test_parse_subject_alternative_name_missing_san() {
        let parser = Parser::new(vec![]);

        let output = parser.parse_subject_alternative_name(None);

        assert_eq!(output, None);
    }

    #[test]
    fn test_parse_validity_period_still_valid() {
        let parser = Parser::new(vec![]);

        let validity = Validity {
            not_before: ASN1Time::from_timestamp(1_700_000_000).unwrap(),
            not_after: ASN1Time::from_timestamp(1_800_000_000).unwrap(),
        };
        let mock_current_time = DateTime::from_timestamp(1_750_000_000, 0).unwrap();

        let output = parser.parse_validity_period(&validity, mock_current_time);

        assert_eq!(output, "14/11/2023 - 15/01/2027 (578 days left)");
    }

    #[test]
    fn test_parse_validity_period_invalid() {
        let parser = Parser::new(vec![]);

        let validity = Validity {
            not_before: ASN1Time::from_timestamp(1_700_000_000).unwrap(),
            not_after: ASN1Time::from_timestamp(1_800_000_000).unwrap(),
        };
        let mock_current_time = DateTime::from_timestamp(1_800_086_400, 0).unwrap();

        let output = parser.parse_validity_period(&validity, mock_current_time);

        assert_eq!(output, "14/11/2023 - 15/01/2027 (Expired)");
    }

    #[test]
    fn test_parse_public_key_and_algorithm() {
        let parser = Parser::new(vec![]);

        let key_bytes: &[u8] = &[0x04, 0x01, 0x02, 0x03];
        let algorithm = AlgorithmIdentifier {
            algorithm: OID_PKCS1_SHA256WITHRSA,
            parameters: None,
        };

        let subject_public_key_info: SubjectPublicKeyInfo<'_> = SubjectPublicKeyInfo {
            algorithm,
            subject_public_key: BitString::new(0, key_bytes),
            raw: &[],
        };

        let output = parser.parse_public_key_and_algorithm(&subject_public_key_info);

        assert_eq!(
            output,
            "sha256WithRSAEncryption (1.2.840.113549.1.1.11) (Unknown([4, 1, 2, 3]))"
        )
    }
}
