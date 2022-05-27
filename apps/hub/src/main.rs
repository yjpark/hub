use std::sync::Arc;

use clap::{Parser, Subcommand};
use tonic::{transport::Server};

use link_hub::{proto::link_hub_server, hub_app::DummyApp};
use sink_hub::{proto::sink_hub_server};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Args {
    /// Name of the person to greet
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Run { port : Option<u16> }
}

pub const DEFAULT_PORT: u16 = 1234;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match &args.command {
        Commands::Run { port } => {
            let port = port.unwrap_or(DEFAULT_PORT);
            println!("Running Hub: port = {:?}", port);
            let addr = format!("0.0.0.0:{}", port).parse().unwrap();
            let link_hub = link_hub::link_hub::LinkHub::new(link_hub::link_authenticator::PublicUuidAuthenticator{}, Some(Arc::new(Box::new(DummyApp{}))));
            let sink_hub = sink_hub::sink_hub::SinkHub::new(sink_hub::sink_authenticator::PublicUuidAuthenticator{}, link_hub.get_apps());
            Server::builder()
                .add_service(link_hub_server::LinkHubServer::new(link_hub))
                .add_service(sink_hub_server::SinkHubServer::new(sink_hub))
                .serve(addr)
                .await?;
        }
    }
    Ok(())
}