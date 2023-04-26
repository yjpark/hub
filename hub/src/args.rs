use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct HubArgs {
    #[arg(long, default_value = "0.0.0.0")]
    pub host: String,

    #[arg(short, long, default_value = "1080")]
    pub port: u16,
}

impl HubArgs {
    pub fn address(&self) -> String {
        let host = &self.host;
        let port = self.port;
        format!("{host}:{port}")
    }
}