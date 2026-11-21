//! The administrator interface for the Liminal system 

use axum::Router;
use axum::http::StatusCode;
use axum::routing::get;

mod pages;
mod api;
mod static_files;
mod templates;

use pages::page_router;
use api::api_router;

use static_files::fixed_handler;
use tower_cookies::CookieManagerLayer;
use wstd::http::{Body, Error, Request, Response};

// Generate the bindings for the wit
pub mod bindgen {
    wit_bindgen::generate!({
        world: "admin-app",
        generate_all,
    });
}

// Main router
#[wstd::http_server]
async fn main(request: Request<Body>) -> Result<Response<Body>, Error> {
    let pages = page_router();
    let api = api_router();
    let service = Router::new()
        .nest("/api",api)
        .merge(pages)
        .route("/static/{*file_name}", get(fixed_handler))
        .layer(CookieManagerLayer::new())
        .fallback(not_found);
    wstd_axum::serve(request, service).await
}

async fn not_found() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not found\n")
}
