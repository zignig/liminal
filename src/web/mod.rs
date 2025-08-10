//! Base web interface.
//! TODO : make this per user.

use std::str::FromStr;

use crate::notes::Notes;
use crate::store::FileSet;
use crate::templates::{GltfPageTemplate, HomePageTemplate, NetworkPageTemplate, NodePageTemplate};
use chrono::Local;
use iroh_blobs::ticket::BlobTicket;
use iroh_blobs::{BlobsProtocol, HashAndFormat};
use rocket::State;
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::get;
use rocket::response::Responder;

pub mod assets;
pub mod auth;
pub mod fixed;
pub mod notes;
pub mod services;

// Run these things
pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Web interface", |rocket| async {
        rocket.mount(
            "/",
            routes![
                index,
                message,
                fixed::dist,
                fixed::favicon,
                viewer,
                network,
                nodes,
                auth::login
            ],
        )
    })
}

#[get("/")]
pub async fn index<'r>() -> impl Responder<'r, 'static> {
    HomePageTemplate {
        section: "".to_string(),
    }
}

// TODO move into utils and make more checks.
pub async fn get_collection(encoded: &str, blobs: &BlobsProtocol) -> anyhow::Result<()> {
    match BlobTicket::from_str(encoded) {
        Ok(ticket) => {
            println!("{:#?}", ticket);
            let (node, hash, hashtype) = ticket.into_parts();
            let conn = blobs
                .endpoint()
                .connect(node, iroh_blobs::protocol::ALPN)
                .await?;
            let knf = HashAndFormat::new(hash, hashtype);
            let local = blobs.store().remote().local(knf).await?;
            if !local.is_complete() {
                println!("a new key {:?}", hash);
                let r = blobs.store().remote().fetch(conn, knf).await?;
                println!("{:?}", r);
                let dt = Local::now().to_rfc3339().to_owned();
                blobs
                    .store()
                    .tags()
                    .set(format!("col-{}", dt), hash)
                    .await?;
            }
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

#[derive(FromForm)]
pub struct BlobUpload<'v> {
    message: &'v str,
}

#[post("/blob", data = "<web_message>")]
pub async fn message<'r>(
    web_message: Form<BlobUpload<'_>>,
    blobs: &State<BlobsProtocol>,
    file_set: &State<FileSet>,
) -> &'static str {
    let encoded = web_message.message.trim();
    let r = get_collection(encoded, blobs).await;
    file_set.fill().await;
    println!("Trans info {:#?}", r);
    "should be an error"
}

#[get("/network")]
pub fn network<'r>(blobs: &State<BlobsProtocol>) -> impl Responder<'r, 'static> {
    let remotes = blobs.endpoint().remote_info_iter();
    let mut nodes: Vec<String> = Vec::new();
    for i in remotes {
        nodes.push(i.node_id.fmt_short())
    }
    NetworkPageTemplate {
        nodes: nodes,
        section: "network".to_string(),
    }
}

#[get("/network/<node_id>")]
pub fn nodes<'r>(node_id: String, blobs: &State<BlobsProtocol>) -> impl Responder<'r, 'static> {
    let mut remote = blobs.endpoint().remote_info_iter();
    let info = remote.find(|node| node_id == node.node_id.fmt_short());
    println!("{:#?}", info);
    NodePageTemplate {
        node_id: node_id,
        section: "network".to_string(),
    }
}

#[get("/viewer")]
pub fn viewer<'r>() -> impl Responder<'r, 'static> {
    GltfPageTemplate {
        path: "/static/gltf/train-diesel-a.glb".to_owned(),
        section: "viewer".to_string(),
    }
}
