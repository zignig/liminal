use crate::{chat::SignedMessage, templates::HomePageTemplate};
use iroh_blobs::net_protocol::Blobs;
use iroh_gossip::api::GossipSender;
use n0_future::StreamExt;
use rocket::response::Responder;
use rocket::State;
use rocket::form::{Form, FromForm};
use rocket::{post,get};

pub mod fixed;


#[get("/")]
pub fn index<'r>() -> impl Responder<'r, 'static> {
    HomePageTemplate {}
}

#[derive(FromForm)]
pub struct WebMessage<'r> {
    room: &'r str,
    username: &'r str,
    message: &'r str
}

#[post("/message",data="<web_message>")]
pub async fn message<'r>(web_message : Form<WebMessage<'_>> , _sender : &State<GossipSender>) ->  &'static str{
    println!("{:?}",&web_message.message);
    
    //    sender.broadcast(message).await?;
    "zoink"
}

#[get("/tag")]
pub async fn tags(blobs :  &State<Blobs>){
    let mut  tag_scan = blobs.store().tags().list_prefix("col").await.unwrap();
    while let Some(event) = tag_scan.next().await { 
        println!("{:?}",event);
    }
}