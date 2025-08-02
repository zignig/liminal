///! A web interface to iroh and friends.
///! Using rocket and tokio
use std::{
    net::{Ipv4Addr, SocketAddrV4},
    str::FromStr,
};

//use futures_lite::StreamExt;
use clap::Parser;
use iroh::{Endpoint, SecretKey};
use iroh_blobs::{ALPN as BLOBS_ALPN, Hash, store::fs::FsStore};
use iroh_gossip::{
    net::{GOSSIP_ALPN, Gossip},
    proto::TopicId,
};
use n0_future::task;

use n0_snafu::{Result, ResultExt, format_err};
use n0_watcher::Watcher;
use std::path::PathBuf;
use tokio::signal::ctrl_c;

mod cli;
mod config;
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
        Command::Open => {
            vec![]
        }
        Command::Join { ticket } => {
            let Ticket { peers } = Ticket::from_str(ticket)?;
            peers
        }
    };

    // Config DB , anyhow vs snafu is weird
    let db_res = config::Info::new(&PathBuf::from("data/config.db"));
    let mut conf = match db_res {
        Ok(conf) => conf,
        Err(_) => return Err(format_err!("bad database!")),
    };

    // Random cli entry will generate a new node id
    // Or use a fixed one from confi
    let secret_key = match &args.random {
        true => SecretKey::generate(rand::rngs::OsRng),
        false => match conf.get_secret_key() {
            Ok(secret) => secret.to_owned(),
            Err(_) => return Err(format_err!("Bad secret")),
        },
    };

    // build our magic endpoint
    let endpoint = Endpoint::builder()
        .secret_key(secret_key)
        .bind_addr_v4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, args.bind_port))
        .bind()
        .await?;

    let _ = conf.add_node(endpoint.node_id());
    let _ = conf.list_nodes();

    println!("> our node id: {}", endpoint.node_id());
    for i in endpoint.remote_info_iter() {
        println!("{:?}", i);
    }

    // create the gossip protocol
    let gossip = Gossip::builder().spawn(endpoint.clone());

    // print a ticket that includes our own node id and endpoint addresses
    let ticket = {
        let me = endpoint.node_addr().initialized().await;
        let peers = peers.iter().cloned().chain([me]).collect();
        Ticket { peers }
    };

    println!("> ticket to join us: {ticket}");

    // Blob data store
    let path = PathBuf::from("data/blobs");
    println!("Data store : {}", path.display());

    // Local blob file store
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
            // Stash the peers in the config
            let _ = conf.add_node(peer.node_id);
            endpoint.add_node_addr(peer)?;
        }
    };

    // Set liminal, hashed as the topic
    let topic = TopicId::from_bytes(*Hash::new("liminal::").as_bytes());

    let (sender, receiver) = gossip.subscribe(topic, peer_ids).await?.split();
    println!("> connected!");

    // Move all this into the replicate
    // subscribe and print loop
    // TODO : this should be a construct

    task::spawn(replicate::subscribe_loop(receiver, blobs.clone()));
    task::spawn(replicate::publish_loop(
        sender,
        blobs.clone(),
        endpoint.secret_key().clone(),
    ));

    // Web interface

    if args.web {
        let rocket_secret_key: [u8; 32] = conf.rocket_key().unwrap();
        println!("starting web server ");
        // start the web server
        let figment = rocket::Config::figment()
            .merge(("address", "0.0.0.0"))
            .merge(("port", 8080))
            .merge(("secret_key", rocket_secret_key))
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
        // Just run and wait.
        ctrl_c().await.unwrap();
    }

    // Shutdown
    router.shutdown().await.e()?;
    Ok(())
}
