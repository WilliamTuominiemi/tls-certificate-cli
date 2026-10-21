use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::{println, str};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let certificate_chain = fetch_certificate("www.example.com")?;

    println!("{}", certificate_chain);

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
