use tracing::{info, debug};
use tracing_log::AsTrace;
use clap::Parser;

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

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

    hub::app::start(&config);

    #[cfg(feature = "hudsucker")]
    hub::proxy::hudsucker::start(&config, shutdown_signal());

    shutdown_signal().await;
}
