//! Base web interface.
//! TODO : make this per user.

use std::str::FromStr;

use crate::store::FileSet;
use crate::templates::{AdminPageTemplate, GltfPageTemplate, HomePageTemplate, IconsPageTemplate};
use crate::web::auth::User;
use chrono::Local;
use iroh::Endpoint;
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
pub mod replica;

// Run these things
pub(crate) fn stage() -> AdHoc {
    AdHoc::on_ignite("Web interface", |rocket| async {
        rocket.mount(
            "/",
            routes![
                index,
                message,
                fixed::dist,
                fixed::favicon,
                viewer,
                auth::login,
                auth::login_post,
                show_icons,
                admin_page
            ],
        )
    })
}

#[get("/")]
pub async fn index<'r>(_user: User) -> impl Responder<'r, 'static> {
    HomePageTemplate {
        section: "".to_string()
    }
}

#[get("/admin")]
pub async fn admin_page<'r>(_user: User) -> impl Responder<'r, 'static> {
    AdminPageTemplate {
        section: "admin".to_string(),
    }
}

// TODO move into utils and make more checks.
pub async fn get_collection(
    encoded: &str,
    blobs: &BlobsProtocol,
    endpoint: &Endpoint,
) -> anyhow::Result<()> {
    match BlobTicket::from_str(encoded) {
        Ok(ticket) => {
            println!("{:#?}", ticket);
            let (node, hash, hashtype) = ticket.into_parts();
            let conn = endpoint.connect(node, iroh_blobs::ALPN).await?;
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
    endpoint: &State<Endpoint>,
    file_set: &State<FileSet>,
) -> &'static str {
    let encoded = web_message.message.trim();
    let r = get_collection(encoded, blobs, endpoint).await;
    file_set.fill("col").await;
    println!("Trans info {:#?}", r);
    "should be an error"
}

#[get("/viewer")]
pub fn viewer<'r>() -> impl Responder<'r, 'static> {
    GltfPageTemplate {
        path: "/static/gltf/train-diesel-a.glb".to_owned(),
        section: "viewer".to_string(),
    }
}

// Show all the fa5 icons for selection
#[get("/icons")]
pub fn show_icons<'r>() -> impl Responder<'r, 'static> {
    let icons_file = fixed::Asset::get("reference/fa5.json");
    let icon_list: Vec<String> = match icons_file {
        Some(icons) => serde_json::from_slice(&icons.data).unwrap(),
        None => vec![],
    };
    IconsPageTemplate {
        section: "admin".to_string(),
        icons: icon_list,
    }
}
