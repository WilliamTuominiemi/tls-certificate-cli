use crate::certificate::Certificate;

use x509_parser::certificate::X509Certificate;

pub struct Parser<'a> {
    x509_certificates: Vec<X509Certificate<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(x509_certificates: Vec<X509Certificate<'a>>) -> Self {
        Parser { x509_certificates }
    }

    pub fn parse_certificates(&self) -> Vec<Certificate> {
        vec![]
    }
}

#[cfg(test)]
mod tests {}
