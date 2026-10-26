use rustls::pki_types::ServerName;
use rustls::{ClientConnection, StreamOwned};
use std::env;
use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;
use x509_parser::prelude::*;

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
            let res: (&[u8], X509Certificate<'_>) = X509Certificate::from_der(der)?;
            let cert = res.1;
            println!("Subject: {}", cert.subject());
            println!("Issuer:  {}", cert.issuer());
            println!(
                "Valid:   {} to {}",
                cert.validity().not_before,
                cert.validity().not_after
            );
        }
    }
    println!("Protocol: {:?}", tls_connection.protocol_version());
    println!("Cipher:   {:?}", tls_connection.negotiated_cipher_suite());

    // let ui = ui::Ui::new(split_certificate_chain, san);
    // let _ = ui.run();

    Ok(())
}
