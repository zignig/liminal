/// The actual routed pages , broken out into a module.
///
use axum::{Router, routing::get};

mod index;
mod login;
mod workloads;

use index::index;
use login::{login, login_post, logout};
use workloads::workload;


// Routing all the pages to endpoints
pub fn page_router() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/login", get(login).post(login_post))
        .route("/logout", get(logout))
        .route("/index", get(index))
}
