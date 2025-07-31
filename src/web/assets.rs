//! Get assets in blobs by collection path 
//! 

use std::path::PathBuf;
use rocket::http::{ContentType, Status};
use rocket::{fairing::AdHoc, State};
use rocket::response::Responder;
use rocket::response::Response;
use std::io::Cursor;

use crate::{store::FileSet, templates::FilePageTemplate};


pub fn stage() -> AdHoc {
    AdHoc::on_ignite("File Browser", |rocket| async {
        rocket.mount(
            "/",
            routes![
                coll,
                files,
                inner_files,
            ],
        )
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
        section: "files".to_string()
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
        section: "files".to_string()
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
        section: "files".to_string()
    }
}


