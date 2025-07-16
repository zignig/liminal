use std::io::Cursor;
use std::path::PathBuf;
// use std::path::PathBuf;
use std::str::FromStr;

use crate::store::FileSet;
use crate::templates::FilePageTemplate;
use crate::templates::HomePageTemplate;
use crate::templates::NotesPageTemplate;
use chrono::Local;
use iroh_blobs::HashAndFormat;
use iroh_blobs::net_protocol::Blobs;
use iroh_blobs::ticket::BlobTicket;
use rocket::State;
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::get;
use rocket::http::{ContentType, Status};
use rocket::response::Responder;
use rocket::response::Response;

pub mod fixed;

fn split_path(path: &PathBuf) -> (Vec<String>, Vec<String>) {
    let v: Vec<String> = path
        .display()
        .to_string()
        .split("/")
        .map(|v| v.to_string())
        .collect();
    // scan and bake
    let mut prefixes: Vec<String> = Vec::new();
    let mut items: Vec<String> = Vec::new();
    for (index, name) in v.iter().enumerate() {
        let pref = v[0..index].join("/");
        prefixes.push(pref);
        items.push(name.to_string())
    }
    (prefixes, items)
}

#[get("/")]
pub fn index<'r>(blobs: &State<Blobs>) -> impl Responder<'r, 'static> {
    let remotes = blobs.endpoint().remote_info_iter();
    let mut nodes: Vec<String> = Vec::new();
    for i in remotes {
        println!("{:#?}", i);
        nodes.push(i.node_id.fmt_short())
    }
    HomePageTemplate { nodes: nodes }
}

pub async fn get_collection(encoded: &str, blobs: &Blobs) -> anyhow::Result<()> {
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
    blobs: &State<Blobs>,
    fileSet: &State<FileSet>,
) -> &'static str {
    let encoded = web_message.message.trim();
    let r = get_collection(encoded, blobs).await;
    fileSet.fill().await;
    println!("Trans info {:#?}", r);
    "should be an error"
}

#[get("/files")]
pub async fn files<'r>(fileset: &State<FileSet>) -> impl Responder<'r, 'static> {
    let coll = fileset.list_roots();
    FilePageTemplate {
        items: coll,
        path: "".to_string(),
        segments: vec![],
        prefixes: vec![],
    }
}

#[get("/files/<collection>")]
pub async fn coll<'r>(collection: &str, fileset: &State<FileSet>) -> impl Responder<'r, 'static> {
    let res = fileset.get(collection.to_string(), &PathBuf::new()).await;
    let mut coll = Vec::new();
    match res {
        Ok(op) => {
            if let Some(r) = op {
                coll = r;
            }
        }
        Err(_) => {}
    }
    let mut path = PathBuf::new();
    path.push(&collection);
    let (pref, items) = split_path(&path);
    FilePageTemplate {
        items: coll,
        path: path.display().to_string(),
        segments: items,
        prefixes: pref,
    }
}

#[get("/files/<collection>/<path..>", rank = 2)]
pub async fn inner_files<'r>(
    collection: &str,
    path: PathBuf,
    fileset: &State<FileSet>,
) -> impl Responder<'r, 'static> {
    let res = fileset.get(collection.to_string(), &path).await;
    let mut items = Vec::new();
    match res {
        Ok(op) => {
            match op {
                Some(r) => items = r,
                None => {
                    // Means no children , is a file
                    let fr = fileset
                        .get_file(collection.to_string(), &path)
                        .await
                        .unwrap();
                    // println!("{:?}",fr);
                    let response = Response::build()
                        .status(Status::Accepted)
                        .header(ContentType::Plain)
                        .sized_body(fr.len(), Cursor::new(fr))
                        .finalize();
                    //return response;
                }
            }
        }
        Err(_) => {}
    }
    let mut full_path = PathBuf::new();
    full_path.push(&collection);
    full_path.push(&path);

    let (pref, entries) = split_path(&full_path);

    FilePageTemplate {
        items: items,
        path: full_path.display().to_string(),
        segments: entries,
        prefixes: pref,
    }
}

#[get("/notes")]
pub fn notes<'r>() -> impl Responder<'r, 'static> {
    NotesPageTemplate {}
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Web interface", |rocket| async {
        rocket.mount(
            "/",
            routes![index, coll, notes, files, message, fixed::dist, inner_files],
        )
    })
}
