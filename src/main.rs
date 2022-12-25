use std::{str::FromStr, sync::Arc, borrow::Borrow};

use axum::{
    extract::{Json, State},
    http::StatusCode,
    routing::post,
    Router,
};
use clap::Parser;
use sled::Db;
use serde::{Deserialize, Serialize};
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
struct Cli {
    #[arg(default_value = "0.0.0.0:3000")]
    server_addr: String,
    #[arg(default_value = "db")]
    db_path: std::path::PathBuf,
}

#[tokio::main]
async fn main() {
    let filter_layer =
        Targets::from_str(std::env::var("RUST_LOG").as_deref().unwrap_or("info")).unwrap();
    let format_layer = tracing_subscriber::fmt::layer();
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(format_layer)
        .init();

    let args = Cli::parse();
    let state = Arc::new(sled::open(args.db_path).unwrap());
    let app = Router::new().route("/", post(handler)).with_state(state);

    // run it with hyper on localhost:3000
    axum::Server::bind(&args.server_addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Deserialize)]
struct Request {
    key: String,
    value: String,
}

#[derive(Serialize)]
struct Response {
    values: Vec<String>,
}

async fn handler(
    State(state): State<Arc<Db>>,
    Json(payload): Json<Request>,
) -> Result<Json<Response>, StatusCode> {
    Ok(Json(handler_internal(state, payload)?))
}

fn handler_internal(
    state: Arc<Db>,
    Request { key, value }: Request,
) -> Result<Response, StatusCode> {
    let item: Vec<_> = key.bytes().chain(value.bytes()).collect();
    state.insert(item, value.as_bytes()).map_err(|e| {
        tracing::error!("failed to write to storage: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let iter = state.scan_prefix(key.as_bytes());
    let values = iter
        .filter_map(Result::ok)
        .map(|(_, v)| std::str::from_utf8(v.borrow()).map(|s|s.to_owned()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            tracing::error!("failed to read from storage: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Response { values })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use super::{Request, handler_internal};
    #[test]
    fn test_handler() {
        let state = Arc::new(sled::open("test").unwrap());
        let response = handler_internal(state.clone(), Request{key:"test".to_owned(), value:"value".to_owned()}).unwrap();
        assert_eq!(response.values, vec!["value".to_owned()]);
        let response = handler_internal(state, Request{key:"test".to_owned(), value:"value2".to_owned()}).unwrap();
        assert_eq!(response.values, vec!["value".to_owned(), "value2".to_owned()]);
    }
}
