use std::env;
use std::process::{Command, Stdio};

mod certificate;
mod parser;
mod ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let url = if args.len() < 2 {
        "www.example.com"
    } else {
        &args[1]
    };

    let certificate_chain = fetch_certificate(url)?;
    let parser = parser::Parser::new(certificate_chain);
    let split_certificate_chain = parser.certificate_chain_to_individual_certificates();
    let san = fetch_san(url)?;

    let ui = ui::Ui::new(split_certificate_chain, san);
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

fn fetch_san(host: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut client = Command::new("openssl")
        .args([
            "s_client",
            "-connect",
            &format!("{}:443", host),
            "-servername",
            host,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;

    let output = Command::new("openssl")
        .args(["x509", "-noout", "-text"])
        .stdin(Stdio::from(client.stdout.take().ok_or("no stdout")?))
        .stderr(Stdio::null())
        .output()?;

    client.wait()?;

    let text = String::from_utf8(output.stdout)?;

    let sans: Vec<String> = text
        .lines()
        .filter(|l| l.contains("DNS:"))
        .flat_map(|l| l.split(','))
        .filter_map(|s| s.trim().strip_prefix("DNS:").map(str::to_string))
        .collect();

    Ok(sans)
}
