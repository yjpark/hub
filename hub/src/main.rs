use tracing::info;
use axum::Router;
use axum::routing::get;
use clap::Parser;

use hub::args::HubArgs;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = HubArgs::parse();
    let address = args.address();
    info!(address, "Starting Hub Server");
    let app = Router::new()
        .route("/", get(|| async { "TODO" }));
    axum::Server::bind(&address.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
