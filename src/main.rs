use std::{
    net::{Ipv4Addr, SocketAddrV4},
    str::FromStr,
};

//use futures_lite::StreamExt;
use clap::Parser;
use iroh::{Endpoint, RelayMode, SecretKey};
use iroh_blobs::{ALPN as BLOBS_ALPN, store::fs::FsStore};
use iroh_gossip::{
    net::{GOSSIP_ALPN, Gossip},
    proto::TopicId,
};
use n0_future::StreamExt;
use n0_future::task;
use n0_snafu::{Result, ResultExt};
use n0_watcher::Watcher;
use snafu::whatever;
use std::path::PathBuf;
//use tokio::{signal::ctrl_c, sync::mpsc};

mod chat;
mod cli;
mod config;
mod importer;
mod replicate;
mod templates;
mod web;

use chat::Message;
use chat::Ticket;
use cli::Command;

use crate::chat::SignedMessage;

#[macro_use]
extern crate rocket;

#[rocket::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = cli::Args::parse();

    // parse the cli command
    let (topic, peers) = match &args.command {
        Command::Open { topic } => {
            let topic = topic.unwrap_or_else(|| TopicId::from_bytes(rand::random()));
            println!("> opening chat room for topic {topic}");
            (topic, vec![])
        }
        Command::Join { ticket } => {
            let Ticket { topic, peers } = Ticket::from_str(ticket)?;
            println!("> joining chat room for topic {topic}");
            (topic, peers)
        }
        Command::Upload { path: _ } => {
            let topic = TopicId::from_bytes(rand::random());
            (topic, vec![])
        }
    };

    // parse or generate our secret key
    let secret_key = match args.secret_key {
        None => SecretKey::generate(rand::rngs::OsRng),
        Some(key) => key.parse()?,
    };

    println!(
        "> our secret key: {}",
        data_encoding::HEXLOWER.encode(&secret_key.to_bytes())
    );

    // configure our relay map
    let relay_mode = match (args.no_relay, args.relay) {
        (false, None) => RelayMode::Default,
        (false, Some(url)) => RelayMode::Custom(url.into()),
        (true, None) => RelayMode::Disabled,
        (true, Some(_)) => {
            whatever!("You cannot set --no-relay and --relay at the same time");
        }
    };
    println!("> using relay servers: {}", fmt_relay_mode(&relay_mode));

    // build our magic endpoint
    let endpoint = Endpoint::builder()
        .secret_key(secret_key)
        .relay_mode(relay_mode)
        .bind_addr_v4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, args.bind_port))
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
        Ticket { topic, peers }
    };
    println!("> ticket to join us: {ticket}");

    println!("blobs!");
    let path = PathBuf::from("data/blobs");
    println!("Data store : {}", path.display());

    let store = FsStore::load(path).await.unwrap();
    let blobs = iroh_blobs::net_protocol::Blobs::new(&store, endpoint.clone(), None);
    match args.command {
        Command::Upload { path } => {
            if let Some(p) = path {
                println!("UPLOAD ! -- {:?}", p.display());
                let good = importer::import(p, &store).await;
                match good {
                    Ok(_) => println!("all good"),
                    Err(e) => println!("{:?}", e),
                }
            }
            // if let Some(p) = path {
            //     let f = store.add_path(&p).await?;
            //     println!("{:#?}", f);
            //     store.tags().set(p.display().to_string(), f).await?;
            // }
        }
        _ => {}
    }

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

    let (mut sender, receiver) = gossip.subscribe(topic, peer_ids).await?.split();
    println!("> connected!");

    // broadcast our name, if set
    if let Some(name) = args.name {
        let message = Message::AboutMe { name };
        let encoded_message = SignedMessage::sign_and_encode(endpoint.secret_key(), &message)?;
        sender.broadcast(encoded_message).await?;
    }

    let mut t = blobs.store().tags().list_prefix("col").await.unwrap();
    while let Some(event) = t.next().await {
        println!("tags {:?}", event);
        // let b = match event {
        //     Ok(t) => {
        //         let c = Collection::load(t.hash, store.as_ref()).await.expect("fail");
        //         println!("{:?}", c);
        //     }
        //     Err(e) => println!("{:?}", e),
        // };
        match event {
            Ok(tag) => {
                let message = Message::Upkey { key: tag.hash };
                println!("{:?}",&message);
                let encoded_message =
                    SignedMessage::sign_and_encode(endpoint.secret_key(), &message)?;
                sender.broadcast(encoded_message).await?;
            }
            Err(_) => todo!(),
        }
    }
    // subscribe and print loop
    task::spawn(chat::subscribe_loop(receiver));

    if args.web {
        println!("starting web server ");
        // start the web server
        let figment = rocket::Config::figment()
            .merge(("address", "0.0.0.0"))
            .merge(("port", 8080))
            .merge(("log_level", "normal"));

        let _result = rocket::custom(figment)
            .manage(sender)
            .manage(blobs.clone())
            .mount(
                "/",
                routes![web::index, web::fixed::dist, web::message, web::tags],
            )
            .launch()
            .await;
    } else {
        // spawn an input thread that reads stdin
        // not using tokio here because they recommend this for "technical reasons"
        let (line_tx, mut line_rx) = tokio::sync::mpsc::channel(1);
        std::thread::spawn(move || chat::input_loop(line_tx));

        // // broadcast each line we type
        println!("> type a message and hit enter to broadcast...");
        while let Some(text) = line_rx.recv().await {
            let message = Message::Message { text: text.clone() };
            let encoded_message = SignedMessage::sign_and_encode(endpoint.secret_key(), &message)?;
            sender.broadcast(encoded_message).await?;
            println!("> sent: {text}");
        }

        // ctrl_c().await.unwrap();
    }
    // Shutdown
    router.shutdown().await.e()?;
    Ok(())
}

// helpers

fn fmt_relay_mode(relay_mode: &RelayMode) -> String {
    match relay_mode {
        RelayMode::Disabled => "None".to_string(),
        RelayMode::Default => "Default Relay (production) servers".to_string(),
        RelayMode::Staging => "Default Relay (staging) servers".to_string(),
        RelayMode::Custom(map) => map
            .urls()
            .map(|url| url.to_string())
            .collect::<Vec<_>>()
            .join(", "),
    }
}
