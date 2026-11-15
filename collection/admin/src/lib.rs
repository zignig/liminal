use axum::Router;
use axum::http::StatusCode;
use axum::routing::get;

mod static_files;
mod templates;
mod pages;

use static_files::fixed_handler;
use pages::page_router;
use tower_cookies::CookieManagerLayer;

#[wstd_axum::http_server]
fn main() -> Router {
    let pages = page_router();
    Router::new()
        .merge(pages)
        .route("/static/{*file_name}", get(fixed_handler))
        .layer(CookieManagerLayer::new())
        .fallback(not_found)
}

async fn not_found() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not found\n")
}
