use crate::certificate::{self, Certificate};

pub struct Parser {
    raw_certificates: String,
}

impl Parser {
    pub fn new(raw_certificates: String) -> Self {
        Parser { raw_certificates }
    }

    pub fn certificate_chain_to_individual_certificates(&self) -> Vec<Certificate> {
        let raw = &self.raw_certificates;

        let Some(chain_start) = raw.find("Certificate chain") else {
            return vec![];
        };
        let Some(chain_end) = raw.find("Server certificate") else {
            return vec![];
        };
        let certificate_chain = &raw[chain_start + 18..chain_end];

        let mut split_certificate_chain: Vec<String> = certificate_chain
            .split("END CERTIFICATE")
            .map(|s| s.strip_prefix("-----\n").unwrap_or(s).to_string())
            .collect();
        split_certificate_chain.truncate(split_certificate_chain.len() - 1);

        self.parse_raw_certificate_list(split_certificate_chain)
    }

    fn parse_raw_certificate_list(&self, split_certificate_chain: Vec<String>) -> Vec<Certificate> {
        let mut certificates = Vec::new();

        for certificate in split_certificate_chain {
            let Some(subject_start) = certificate.find("s:") else {
                break;
            };
            let Some(subject_end) = certificate[subject_start..].find("\n") else {
                break;
            };

            let Some(issuer_start) = certificate.find("i:") else {
                break;
            };
            let Some(issuer_end) = certificate[issuer_start..].find("\n") else {
                break;
            };

            let Some(auth_start) = certificate.find("a:") else {
                break;
            };
            let Some(auth_end) = certificate[auth_start..].find("\n") else {
                break;
            };

            let Some(validity_period_start) = certificate.find("v:") else {
                break;
            };
            let Some(validity_period_end) = certificate[validity_period_start..].find("\n") else {
                break;
            };

            let certificate = certificate::Certificate {
                subject: certificate[subject_start + 2..subject_start + subject_end].to_string(),
                issuer: certificate[issuer_start + 2..issuer_start + issuer_end].to_string(),
                auth: certificate[auth_start + 2..auth_start + auth_end].to_string(),
                validity_period: certificate
                    [validity_period_start + 2..validity_period_start + validity_period_end]
                    .to_string(),
            };

            certificates.push(certificate);
        }

        certificates
    }
}

#[cfg(test)]
mod tests {
    use std::assert_eq;

    use super::*;

    #[test]
    fn test_certificate_chain_to_individual_certificates() {
        let certificate_chain = "
            vjo80fg809adf9fb0d+g9f9
            v8bd89809dfnjkn23908dfs
            ---
            Certificate chain
            0:  s:
                i:
                a:
                v:
            -----BEGIN CERTIFICATE-----
            123
            -----END CERTIFICATE-----
            1:  s:
                i:
                a:
                v:
            -----BEGIN CERTIFICATE-----
            321
            -----END CERTIFICATE-----
            2:  s:
                i:
                a:
                v:
            -----BEGIN CERTIFICATE-----
            231
            -----END CERTIFICATE-----
            ---
            Server certificate
            vjo80fg809adf9fb0d+g9f9
            v8bd89809dfnjkn23908dfs
            ";

        let parser = Parser::new(certificate_chain.to_string());

        assert_eq!(
            parser.certificate_chain_to_individual_certificates().len(),
            3
        )
    }

    #[test]
    fn test_parse_raw_certificate_list() {
        let raw_certificates = "
            ---
            Certificate chain
            0:  s:SUBJECT
                i:ISSUER
                a:AUTH
                v:VALIDITY_PERIOD
            -----BEGIN CERTIFICATE-----
            123
            -----END CERTIFICATE-----
            ---
            Server certificate
            ";
        let parser = Parser::new(raw_certificates.to_string());
        let individual_certificates = parser.certificate_chain_to_individual_certificates();

        assert_eq!(individual_certificates[0].subject, "SUBJECT");
        assert_eq!(individual_certificates[0].issuer, "ISSUER");
        assert_eq!(individual_certificates[0].auth, "AUTH");
        assert_eq!(
            individual_certificates[0].validity_period,
            "VALIDITY_PERIOD"
        );
    }
}
