use std::net::SocketAddr;

pub use self::error::{Error, Result};

use axum::extract::{Path, Query};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, get_service};
use axum::{middleware, Json, Router};
use serde::Deserialize;
use serde_json::json;
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;
use uuid::Uuid;

mod ctx;
mod error;
mod model;
mod web;

#[tokio::main]
async fn main() -> Result<()> {
    // initialize ModelController
    let mc = model::ModelController::new().await?;

    let routes_apis = web::routes_tickets::routes(mc.clone())
        .route_layer(middleware::from_fn(web::mw_auth::mw_require_auth));

    let routes_all = Router::new()
        .merge(routes_hello())
        .merge(web::routes_login::routes())
        .nest("/api", routes_apis)
        .layer(middleware::map_response(main_response_mapper))
        .layer(middleware::from_fn_with_state(
            mc.clone(),
            web::mw_auth::mw_ctx_resolver,
        ))
        .layer(CookieManagerLayer::new())
        .fallback_service(routes_static());

    //start server on localhost:8000
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("Server running on {}", addr);
    axum::Server::bind(&addr)
        .serve(routes_all.into_make_service())
        .await
        .unwrap();

    Ok(())
}

// middleware to log response and add spaces in console
async fn main_response_mapper(res: Response) -> Response {
    println!("->> {:<12} - MAIN_RESP_MAPPER", "HANDLER");
    let uuid = Uuid::new_v4();

    let service_error = res.extensions().get::<Error>();
    let client_status_error = service_error.map(|e| e.client_status_and_error());

    let error_response =
        client_status_error.as_ref()
        .map(|(status_code, client_error)| {
            let client_error_body = json!({
                "error": {
                    "message": client_error.as_ref(),
                    "uuid": uuid.to_string(),
                }
            });
            println!(" ->> client error body: {client_error_body}");

            (*status_code, Json(client_error_body)).into_response()
        });

        println!(" ->> server log line - {uuid} - Error : {service_error:?}");

    println!();
    error_response.unwrap_or(res)
}

fn routes_static() -> Router {
    Router::new().nest_service("/", get_service(ServeDir::new("./")))
}

fn routes_hello() -> Router {
    Router::new()
        .route("/hello", get(handler_hello))
        .route("/hello/:name", get(handler_hello2))
}

#[derive(Debug, Deserialize)]
struct HelloParams {
    name: Option<String>,
}

// handler for /hello
async fn handler_hello(Query(params): Query<HelloParams>) -> impl IntoResponse {
    println!("->> {:<12} - Handler_hello", "HANDLER");

    let name = params.name.as_deref().unwrap_or("World");
    Html(format!("Hello {name}! "))
}

async fn handler_hello2(Path(name): Path<String>) -> impl IntoResponse {
    println!("->> {:<12} - Handler_hello2", "HANDLER");
    Html(format!("Hello {name}!"))
}
