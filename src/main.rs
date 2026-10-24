use std::process::{Command, Stdio};

mod ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let certificate_chain = fetch_certificate("www.example.com")?;
    let split_certificate_chain = certificate_chain_to_individual_certificates(&certificate_chain);

    let ui = ui::Ui::new(split_certificate_chain);
    let _ = ui.run();

    Ok(())
}

fn fetch_certificate(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("openssl")
        .arg("s_client")
        .arg("-showcerts")
        .arg("-servername")
        .arg(url)
        .arg("-connect")
        .arg(format!("{}:443", url))
        .stdin(Stdio::null())
        .output()?;

    Ok(match str::from_utf8(&output.stdout) {
        Ok(val) => val.to_string(),
        Err(_) => panic!("got non UTF-8 data from openssl"),
    })
}

fn certificate_chain_to_individual_certificates(certificate_chain: &str) -> Vec<String> {
    if let Some(start_index) = certificate_chain.find("Certificate chain") {
        let certificate_chain_start_trimmed =
            &certificate_chain[(start_index)..certificate_chain.len()];

        if let Some(end_index) = certificate_chain.find("Server certificate") {
            let certificate_chain_start_and_end_trimmed =
                &certificate_chain_start_trimmed[18..end_index];

            let mut split_certificate_chain: Vec<String> = certificate_chain_start_and_end_trimmed
                .split("END CERTIFICATE")
                .map(|s| s.strip_prefix("-----\n").unwrap_or(s).to_string())
                .collect();

            split_certificate_chain.truncate(split_certificate_chain.len() - 1);

            return split_certificate_chain;
        }
    }

    vec![]
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

        assert_eq!(
            certificate_chain_to_individual_certificates(&certificate_chain.to_owned()).len(),
            3
        )
    }
}
