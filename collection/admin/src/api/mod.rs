//! http api, for mythril items

///
use crate::bindgen::wasi::keyvalue::store::open;
use axum::{Json, Router, extract::Path, response::IntoResponse, routing::get};
use postcard;
use serde::Serialize;

// These all prefix to /api/<ITEM>
pub fn api_router() -> Router {
    Router::new()
        .route("/worklist", get(worklist))
        .route("/prefill", get(prefill))
        .route("/worklist/{id}/{status}", get(change_status))
}

#[derive(Serialize)]
pub enum Status {
    Running,
    Stopped,
    Error,
}

#[derive(Serialize)]
pub struct WorkItem {
    name: String,
    id: usize,
    status: Status,
}

pub async fn worklist() -> Json<Vec<WorkItem>> {
    let data = open("testing").expect("no keyspace");
    let items = data.list_keys(None).expect("no keys");
    let mut data = items.keys.clone();
    let mut info: Vec<WorkItem> = vec![];
    for item in data.iter() {
        let wi = WorkItem {
            name: item.to_owned(),
            id: 0,
            status: Status::Running,
        };
        info.push(wi);
    }
    Json(info)
}

pub async fn prefill() {
    let data = open("testing").expect("no keyspace");
    // test data
    for i in 0..15 {
        let wi = WorkItem {
            name: i.to_string(),
            id: 0,
            status: Status::Stopped,
        };
        let wi_encoded: Vec<u8> = postcard::to_allocvec(&wi).unwrap();
        let _ = data.set(&i.to_string(), &wi_encoded.as_slice());
    }
}

pub async fn change_status(Path((id, action)): Path<(String, String)>) -> impl IntoResponse {
    let data = open("testing").expect("no keyspace");
    let val = data.get(id.as_str()).expect("no_key").unwrap();
    val
    // test data
}
