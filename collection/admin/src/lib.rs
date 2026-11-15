use axum::Router;
use axum::http::StatusCode;
use axum::routing::get;

mod static_files;
mod templates;

use static_files::fixed_handler;

#[wstd_axum::http_server]
fn main() -> Router {
    let pages = templates::page_router();
    Router::new()
        .merge(pages)
        .route("/static/{*file_name}", get(fixed_handler))
        .fallback(not_found)
}

async fn not_found() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not found\n")
}
