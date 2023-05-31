use tracing::{info, warn, debug};
use tracing_log::AsTrace;
use axum::Router;
use axum::routing::get;
use clap::Parser;

#[tokio::main]
async fn main() {
    let args = hub::Args::parse();
    tracing_subscriber::fmt()
        .with_max_level(args.verbose.log_level_filter().as_trace())
        .with_file(true)
        .with_line_number(true)
        .with_target(true)
        .with_thread_ids(true)
        .init();
    info!("loading config from path: {:#?}", args.config);
    let config = hub::Config::load(&args);
    debug!("running with hub config: {:#?}", config);
    let app = Router::new()
        .route("/", get(|| async { "TODO" }));
    let proxy = hub_proxy_hudsucker::Proxy::new(config.clone().into());
    proxy.start();
    tokio::task::spawn(
        async move {
            warn!(host = config.host, port = config.port, "starting hub server");
            axum::Server::bind(&config.address())
                .serve(app.into_make_service())
                .await
                .unwrap();
        }
    );
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}
