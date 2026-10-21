use crossterm::event;
use std::process::{Command, Stdio};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let certificate_chain = fetch_certificate("www.example.com")?;
    ratatui::run(|mut terminal| {
        loop {
            terminal.draw(render)?;
            if event::read()?.is_key_press() {
                break Ok(());
            }
        }
    })
}

fn render(frame: &mut ratatui::Frame) {
    frame.render_widget("TLS certificate", frame.area());
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
