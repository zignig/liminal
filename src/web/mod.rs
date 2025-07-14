use crate::templates::FilePageTemplate;
use crate::templates::HomePageTemplate;
use iroh_blobs::format::collection::Collection;
use iroh_blobs::net_protocol::Blobs;
use n0_future::StreamExt;
use rocket::State;
use rocket::get;
use rocket::response::Responder;

pub mod fixed;

#[get("/")]
pub fn index<'r>() -> impl Responder<'r, 'static> {
    HomePageTemplate {}
}

// #[post("/message", data = "<web_message>")]
// pub async fn message<'r>(web_message: Form<WebMessage<'_>>) -> &'static str {
//     println!("{:?}", &web_message.message);

//     //    sender.broadcast(message).await?;
//     "zoink"
// }

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
    if let Ok(item) = blobs.store().tags().get(item).await{ 
        if let Some(val) = item { 
            if let Ok(c) = Collection::load(val.hash,blobs.store()).await{
                for (item,_) in c{ 
                    coll.push(item)
                }
            }
        }
    }
    
    FilePageTemplate { items: coll }
}
