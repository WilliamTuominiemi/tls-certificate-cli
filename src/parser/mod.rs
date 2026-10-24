pub struct Parser {
    raw_certificates: String,
}

impl Parser {
    pub fn new(raw_certificates: String) -> Self {
        Parser { raw_certificates }
    }

    pub fn certificate_chain_to_individual_certificates(&self) -> Vec<String> {
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

        split_certificate_chain
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
            vjo80fg809adf9fb0d+g9f9
            v8bd89809dfnjkn23908dfs
            ";

        let parser = Parser::new(certificate_chain.to_string());

        assert_eq!(
            parser.certificate_chain_to_individual_certificates().len(),
            3
        )
    }
}
