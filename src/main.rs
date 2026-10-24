use std::env;
use std::process::{Command, Stdio};

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
