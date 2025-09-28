//! Get assets in blobs by collection path
//!

use iroh::Watcher;
use iroh_blobs::{BlobFormat, BlobsProtocol};
use iroh_blobs::ticket::BlobTicket;
use rocket::response::Responder;
use rocket::{State, fairing::AdHoc};
use std::path::PathBuf;

use crate::{
    store::{FileSet, RenderType},
    templates::FilePageTemplate,
};

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("File Browser", |rocket| async {
        rocket.mount("/", routes![coll, files, inner_files,])
    })
}

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

#[get("/files")]
pub async fn files<'r>(fileset: &State<FileSet>) -> impl Responder<'r, 'static> {
    let coll = fileset.list_roots();
    FilePageTemplate {
        items: coll,
        path: "".to_string(),
        segments: vec![],
        prefixes: vec![],
        section: "files".to_string(),
        ticket: None
    }
}

#[get("/files/<collection>")]
pub async fn coll<'r>(collection: &str, fileset: &State<FileSet>,blobs: &State<BlobsProtocol>) -> impl Responder<'r, 'static> {
    let res = fileset.get(collection.to_string(), &PathBuf::new()).await;
    match res {
        Ok(res) => {
            let mut ticket_opt: Option<String> = None;
            if let Some(hash) = fileset.get_hash(collection.to_string()).await.unwrap(){ 
                let addr = blobs.endpoint().node_addr().initialized().await;
                let ticket = BlobTicket::new(addr,hash, BlobFormat::HashSeq);
                ticket_opt = Some(ticket.to_string());
            }
            
            if let Some(item) = res {
                match item {
                    RenderType::File { file_name: _ } => return Err(()),
                    RenderType::Folder { items } => {
                        let mut path = PathBuf::new();
                        path.push(&collection);
                        let (pref, seg) = split_path(&path);
                        return Ok(FilePageTemplate {
                            items: items,
                            path: path.display().to_string(),
                            segments: seg,
                            prefixes: pref,
                            section: "files".to_string(),
                            ticket: ticket_opt
                        });
                    }
                }
            } else {
                Err(())
            }
        }
        Err(_) => return Err(()),
    }
}

#[get("/files/<collection>/<path..>", rank = 2)]
pub async fn inner_files<'r>(
    collection: &str,
    path: PathBuf,
    fileset: &State<FileSet>,
) -> impl Responder<'r, 'static> {
    let res = fileset.get(collection.to_string(), &path).await;
    match res {
        Ok(res) => {
            if let Some(item) = res {
                match item {
                    RenderType::File { file_name: _  } => todo!(),
                    RenderType::Folder { items } => {
                        let mut full_path = PathBuf::new();
                        full_path.push(&collection);
                        full_path.push(&path);
                        let (pref, entries) = split_path(&full_path);
                        return Ok(FilePageTemplate {
                            items: items,
                            path: full_path.display().to_string(),
                            segments: entries,
                            prefixes: pref,
                            section: "files".to_string(),
                            ticket: None
                        });
                    }
                }
            } else {
                return Err(())
            }
        }
        Err(_) => return Err(()),
    }
}
