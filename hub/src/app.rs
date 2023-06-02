use tracing::warn;
use axum::Router;
use axum::routing::get;

use crate::Config;

fn get_app(_config: &Config) -> Router {
    Router::new()
        .route("/", get(|| async { "TODO" }))
}

pub fn start(config: &Config) {
    let app = get_app(config);
    let address = config.address().clone();
    tokio::task::spawn(
        async move {
            warn!(address = address.to_string(), "starting hub server");
            axum::Server::bind(&address)
                .serve(app.into_make_service())
                .await
                .expect("failed to start hub server");
        }
    );
}
