//! http api, for mythril items

///
use crate::bindgen::wasi::keyvalue::store::open;
use axum::{Json, Router, extract::Path, response::IntoResponse, routing::get};
use serde::{Deserialize, Serialize};

// These all prefix to /api/<ITEM>
pub fn api_router() -> Router {
    Router::new()
        .route("/worklist", get(worklist))
        .route("/prefill", get(prefill))
        .route("/worklist/{id}/{status}", get(change_status))
}

#[derive(Deserialize,Serialize)]
pub enum Status {
    Running,
    Stopped,
    Error,
}

#[derive(Deserialize,Serialize)]
pub struct WorkItem {
    name: String,
    id: usize,
    status: Status,
}

pub async fn worklist() -> Json<Vec<WorkItem>> {
    let bucket = open("testing").expect("no keyspace");
    let items = bucket.list_keys(None).expect("no keys");
    let data = items.keys.clone();

    let mut info: Vec<WorkItem> = vec![];

    for item in data.iter() {
        match bucket.get(item).unwrap(){
            Some(val) => {
                let wi = serde_json::from_slice(val.as_slice()).unwrap();
                info.push(wi);
            },
            None => todo!(),
        }
    }
    Json(info)
}

pub async fn prefill() {
    let data = open("testing").expect("no keyspace");
    // test data
    for i in 0..100 {
        let wi = WorkItem {
            name: i.to_string(),
            id: 0,
            status: Status::Stopped,
        };
        let wi_encoded = serde_json::to_string(&wi).unwrap();
        let _ = data.set(&i.to_string(), &wi_encoded.as_bytes());
    }
}

pub async fn change_status(Path((id, action)): Path<(String, String)>) -> impl IntoResponse{
    let bucket = open("testing").expect("no keyspace");
    match bucket.get(id.as_str()).expect("no_key"){
        Some(data) => {
            let mut wi: WorkItem = serde_json::from_slice(data.as_slice()).unwrap();
            match action.as_str() {
                "Running" => wi.status = Status::Running,
                "Stopped" => wi.status = Status::Stopped,
                _ => {}
            }
            let out_data = serde_json::to_string(&wi).unwrap();
            let _ = bucket.set(id.as_str(),&out_data.as_bytes());
        },
        None => {},
    }
    "ok"
}
