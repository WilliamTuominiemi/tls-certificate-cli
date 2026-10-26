use rustls::pki_types::ServerName;
use rustls::{ClientConnection, StreamOwned};
use std::env;
use std::io::Write;
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::sync::Arc;

mod certificate;
mod parser;
mod ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let host = if args.len() < 2 {
        "www.example.com"
    } else {
        &args[1]
    };

    let root_store =
        rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let server_name = ServerName::try_from(host.to_string())?;
    let connection = ClientConnection::new(Arc::new(config), server_name)?;
    let socket = TcpStream::connect((host, 443))?;
    let mut tls = StreamOwned::new(connection, socket);

    tls.flush()?;

    let tls_connection = &tls.conn;
    if let Some(certificates) = tls_connection.peer_certificates() {
        for (i, der) in certificates.iter().enumerate() {
            let pem = pem::encode(&pem::Pem::new("CERTIFICATE", der.as_ref().to_vec()));
            println!("--- Certificate {i} ---\n{pem}");
        }
    }
    println!("Protocol: {:?}", tls_connection.protocol_version());
    println!("Cipher:   {:?}", tls_connection.negotiated_cipher_suite());

    // let certificate_chain = fetch_certificate(url)?;
    // let parser = parser::Parser::new(certificate_chain);
    // let split_certificate_chain = parser.certificate_chain_to_individual_certificates();
    // let san = fetch_san(url)?;

    // let ui = ui::Ui::new(split_certificate_chain, san);
    // let _ = ui.run();

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
