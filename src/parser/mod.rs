pub struct Parser {
    raw_certificate_chain: String,
}

impl Parser {
    pub fn new(raw_certificate_chain: String) -> Self {
        Parser {
            raw_certificate_chain,
        }
    }

    pub fn certificate_chain_to_individual_certificates(&self) -> Vec<String> {
        if let Some(start_index) = self.raw_certificate_chain.find("Certificate chain") {
            let certificate_chain_start_trimmed =
                &self.raw_certificate_chain[(start_index)..self.raw_certificate_chain.len()];

            if let Some(end_index) = self.raw_certificate_chain.find("Server certificate") {
                let certificate_chain_start_and_end_trimmed =
                    &certificate_chain_start_trimmed[18..end_index];

                let mut split_certificate_chain: Vec<String> =
                    certificate_chain_start_and_end_trimmed
                        .split("END CERTIFICATE")
                        .map(|s| s.strip_prefix("-----\n").unwrap_or(s).to_string())
                        .collect();

                split_certificate_chain.truncate(split_certificate_chain.len() - 1);

                return split_certificate_chain;
            }
        }

        vec![]
    }
}

#[cfg(test)]
mod tests {
    use std::assert_eq;

    use super::*;

    #[test]
    fn test_certificate_chain_to_individual_certificates() {
        let certificate_chain = "
            ---
            Certificate chain
            0: first
            -----BEGIN CERTIFICATE-----
            123
            -----END CERTIFICATE-----
            1: second
            -----BEGIN CERTIFICATE-----
            321
            -----END CERTIFICATE-----
            2. third
            -----BEGIN CERTIFICATE-----
            231
            -----END CERTIFICATE-----
            ---
            Server certificate
            ";

        let parser = Parser::new(certificate_chain.to_string());

        assert_eq!(
            parser.certificate_chain_to_individual_certificates().len(),
            3
        )
    }
}
