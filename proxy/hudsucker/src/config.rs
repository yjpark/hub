use hudsucker::{certificate_authority::RcgenAuthority, rustls::{PrivateKey, Certificate}};
use secrecy::{Secret, ExposeSecret};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub private_key: Secret<String>,
    pub certificate: Secret<String>, 
}

impl Config {
    pub const CA_CACHE_SIZE: u64 = 1000;

    pub fn address(&self) -> std::net::SocketAddr {
        std::net::SocketAddr::from((self.host.parse::<std::net::IpAddr>().expect("invalid host"), self.port))
    }

    pub fn get_ca(&self) -> RcgenAuthority {
        use rustls_pemfile as permfile; 

        let mut private_key_bytes: &[u8] = self.private_key.expose_secret().as_bytes().clone();
        let mut certificate_bytes: &[u8] = self.certificate.expose_secret().as_bytes().clone();

        let private_key = PrivateKey(
            permfile::pkcs8_private_keys(&mut private_key_bytes)
                .expect("failed to parse private key")
                .remove(0)
        );
        let ca_cert = Certificate(
            permfile::certs(&mut certificate_bytes)
                .expect("failed to parse CA certificate")
                .remove(0)
        );
        RcgenAuthority::new(private_key, ca_cert, Self::CA_CACHE_SIZE)
            .expect("failed to create CA")
    }
}