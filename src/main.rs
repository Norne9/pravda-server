mod database;
mod database_pg;
mod pravda_handler;
mod utils;

use crate::database_pg::DatabasePg;
use crate::pravda_handler::PravdaHandler;
use axum::extract::State;
use axum::{
    http::{header::HeaderMap, StatusCode},
    routing::post,
    Json, Router,
};
use pravda_protocol::{ProtocolError, Request, Response};
use std::env;
use std::net::SocketAddr;
use tower_http::services::{ServeDir, ServeFile};
use tracing::{error, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize tracing
    tracing_subscriber::fmt::init();
    dotenvy::dotenv()?;

    let database = DatabasePg::connect(env::var("DATABASE_URL")?).await?;
    let handler = PravdaHandler::new(database);

    let dir_server =
        ServeDir::new("assets").not_found_service(ServeFile::new("assets/not_found.html"));
    // let index_server = ServeFile::new("assets/index.html");

    // build our application with a route
    let app = Router::new()
        .route("/api", post(process_request))
        .fallback_service(dir_server)
        .with_state(handler);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

#[axum::debug_handler]
async fn process_request(
    headers: HeaderMap,
    State(handler): State<PravdaHandler<DatabasePg>>,
    Json(request): Json<Request>,
) -> (StatusCode, Json<Response>) {
    let token = match headers.get("P-Token") {
        None => None,
        Some(token) => match token.to_str() {
            Ok(token) => Some(token.to_string()),
            Err(e) => {
                error!("Failed to parse P-Token: {:?}", e);
                None
            }
        },
    };

    let response = handler.process(request, token).await;
    match response {
        Ok(_) => (StatusCode::OK, Json(response)),
        Err(ProtocolError::Unknown(_)) => {
            error!("Unknown error while handling request: {:?}", response);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
        Err(_) => {
            warn!("User error: {:?}", response);
            (StatusCode::BAD_REQUEST, Json(response))
        }
    }
}
