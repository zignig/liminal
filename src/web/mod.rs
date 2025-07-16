use std::any;
use std::str::FromStr;

use crate::chat::Ticket;
use crate::templates::FilePageTemplate;
use crate::templates::HomePageTemplate;
use crate::web;
use chrono::Local;
use iroh_blobs::HashAndFormat;
use iroh_blobs::format::collection::Collection;
use iroh_blobs::net_protocol::Blobs;
use iroh_blobs::ticket::BlobTicket;
use n0_future::StreamExt;
use rocket::State;
use rocket::form::Form;
use rocket::get;
use rocket::response::Responder;

pub mod fixed;

#[get("/")]
pub fn index<'r>() -> impl Responder<'r, 'static> {
    HomePageTemplate {}
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
pub async fn message<'r>(web_message: Form<BlobUpload<'_>>, blobs: &State<Blobs>) -> &'static str {
    let encoded = web_message.message.trim();
    let r = get_collection(encoded, blobs).await;
    println!("Trans info {:#?}", r);
    "should be an error"
}

#[get("/files")]
pub async fn files<'r>(blobs: &State<Blobs>) -> impl Responder<'r, 'static> {
    let mut tag_scan = blobs.store().tags().list_prefix("col").await.unwrap();
    let mut coll: Vec<String> = Vec::new();
    while let Some(event) = tag_scan.next().await {
        let tag = event.unwrap();
        let tag_name = str::from_utf8(&tag.name.0).unwrap().to_owned();
        //let tag_name = tag.name.to_string();
        println!("{}", &tag_name);
        coll.push(tag_name);
    }

    FilePageTemplate { items: coll }
}

#[get("/files/<item>")]
pub async fn coll<'r>(item: String, blobs: &State<Blobs>) -> impl Responder<'r, 'static> {
    let mut coll: Vec<String> = Vec::new();
    if let Ok(item) = blobs.store().tags().get(item).await {
        if let Some(val) = item {
            if let Ok(c) = Collection::load(val.hash, blobs.store()).await {
                for (item, _) in c {
                    coll.push(item)
                }
            }
        }
    }

    FilePageTemplate { items: coll }
}
