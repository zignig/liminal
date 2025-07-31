use std::{
    net::{Ipv4Addr, SocketAddrV4},
    str::FromStr,
};

//use futures_lite::StreamExt;
use clap::Parser;
use iroh::{Endpoint, SecretKey};
use iroh_blobs::{store::fs::FsStore, Hash, ALPN as BLOBS_ALPN};
use iroh_gossip::{
    net::{GOSSIP_ALPN, Gossip},
    proto::TopicId,
};
use n0_future::task;
use n0_snafu::{Result, ResultExt};
use n0_watcher::Watcher;
use std::path::PathBuf;
use tokio::signal::ctrl_c;

mod cli;
mod replicate;
mod store;
mod templates;
mod web;

use cli::Command;
use cli::Ticket;

#[macro_use]
extern crate rocket;

#[rocket::main]
async fn main() -> Result<()> {
    // TODO: make this shorter
    tracing_subscriber::fmt::init();
    let args = cli::Args::parse();

    // parse the cli command
    let peers = match &args.command {
        Command::Open { topic } => {
            let topic = topic.unwrap_or_else(|| TopicId::from_bytes(rand::random()));
            println!("> opening chat room for topic {topic}");
            vec![]
        }
        Command::Join { ticket } => {
            let Ticket { peers } = Ticket::from_str(ticket)?;
            peers
        }
        Command::Upload { path: _ } => {
            let topic = TopicId::from_bytes(rand::random());
            vec![]
        }
    };

    // parse or generate our secret key
    let secret_key = match args.secret_key {
        None => SecretKey::generate(rand::rngs::OsRng),
        Some(key) => key.parse()?,
    };

    // build our magic endpoint
    let endpoint = Endpoint::builder()
        .secret_key(secret_key)
        // .relay_mode(relay_mode)
        .bind_addr_v4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, args.bind_port))
        .discovery_local_network()
        .bind()
        .await?;

    println!("> our node id: {}", endpoint.node_id());
    for i in endpoint.remote_info_iter() {
        println!("{:?}", i);
    }

    // create the gossip protocol
    let gossip = Gossip::builder().spawn(endpoint.clone());

    // print a ticket that includes our own node id and endpoint addresses
    let ticket = {
        let me = endpoint.node_addr().initialized().await?;
        let peers = peers.iter().cloned().chain([me]).collect();
        Ticket { peers }
    };

    println!("> ticket to join us: {ticket}");

    // Blob data store
    let path = PathBuf::from("data/blobs");
    println!("Data store : {}", path.display());

    // Local blob store
    let store = FsStore::load(path).await.unwrap();

    // BLOBS! 
    let blobs = iroh_blobs::BlobsProtocol::new(&store, endpoint.clone(), None);

    // Path browser
    let fileset = store::FileSet::new(blobs.clone());

    // Get the file roots 
    fileset.fill().await;

    // setup router
    let router = iroh::protocol::Router::builder(endpoint.clone())
        .accept(GOSSIP_ALPN, gossip.clone())
        .accept(BLOBS_ALPN, blobs.clone())
        .spawn();

    // join the gossip topic by connecting to known peers, if any

    let peer_ids = peers.iter().map(|p| p.node_id).collect();
    if peers.is_empty() {
        println!("> waiting for peers to join us...");
    } else {
        println!("> trying to connect to {} peers...", peers.len());
        // add the peer addrs from the ticket to our endpoint's addressbook so that they can be dialed
        for peer in peers.into_iter() {
            endpoint.add_node_addr(peer)?;
        }
    };

    // Set liminal, hashed as the topic 
    let topic = TopicId::from_bytes(*Hash::new("liminal::").as_bytes());

    let (sender, receiver) = gossip.subscribe(topic, peer_ids).await?.split();
    println!("> connected!");

    // Move all this into the replicate
    // subscribe and print loop
    task::spawn(replicate::subscribe_loop(receiver, blobs.clone()));
    task::spawn(replicate::publish_loop(
        sender,
        blobs.clone(),
        endpoint.secret_key().clone(),
    ));

    if args.web {
        println!("starting web server ");
        // start the web server
        let figment = rocket::Config::figment()
            .merge(("address", "0.0.0.0"))
            .merge(("port", 8080))
            .merge(("log_level", "normal"));

        let _result = rocket::custom(figment)
            // .manage(sender)
            .manage(fileset.clone())
            .manage(blobs.clone())
            .attach(web::stage())
            .attach(web::assets::stage())
            .attach(web::services::stage())
            .launch()
            .await;
    } else {
        ctrl_c().await.unwrap();
    }
    // Shutdown
    router.shutdown().await.e()?;
    Ok(())
}
