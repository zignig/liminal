///! A web interface to iroh and friends.
///! Using rocket and tokio
///! Testing ground docs,blobs and gossip
///! It should be a usefull interface

use std::{
    net::{Ipv4Addr, SocketAddrV4},
    str::FromStr,
};

use clap::Parser;
use iroh::{Endpoint, NodeId, SecretKey};
use iroh_blobs::{ALPN as BLOBS_ALPN, Hash, store::fs::FsStore};
use iroh_docs::{ALPN as DOCS_ALPN, AuthorId, protocol::Docs};
use iroh_gossip::{
    net::{GOSSIP_ALPN, Gossip},
    proto::TopicId,
};

use n0_snafu::{Result, ResultExt, format_err};
use n0_watcher::Watcher;
use std::path::PathBuf;
use tokio::signal::ctrl_c;

mod cli;
mod config;
mod notes;
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

    // --random cli entry will generate a new node id
    // Or use a fixed one from config
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
        .discovery_n0()
        .bind_addr_v4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, args.bind_port))
        .bind()
        .await?;

    // Stash some nodes
    let _ = conf.add_node(endpoint.node_id());
    let _ = conf.list_nodes();

    println!("> our node id: {}", endpoint.node_id());
    for i in endpoint.remote_info_iter() {
        println!("{:?}", i);
    }

    // print a ticket that includes our own node id and endpoint addresses
    let ticket = {
        let me = endpoint.node_addr().initialized().await;
        let peers = peers.iter().cloned().chain([me]).collect();
        Ticket { peers }
    };

    println!("\n\n> ticket to join us: {ticket}");

    // create the gossip protocol
    let gossip = Gossip::builder().spawn(endpoint.clone());

    // BLOBS!
    let path = PathBuf::from("data/blobs");
    let store = FsStore::load(path).await.unwrap();
    let blobs = iroh_blobs::BlobsProtocol::new(&store, endpoint.clone(), None);

    // Path browser
    let fileset = store::FileSet::new(blobs.clone());
    // TODO make this prettier.
    // Get the file roots
    fileset.fill().await;

    // DOCS !
    let docs_path = PathBuf::from("data/");
    let docs = Docs::persistent(docs_path)
        .spawn(endpoint.clone(), (*blobs).clone(), gossip.clone())
        .await
        .unwrap();

    // setup router
    let router = iroh::protocol::Router::builder(endpoint.clone())
        .accept(GOSSIP_ALPN, gossip.clone())
        .accept(BLOBS_ALPN, blobs.clone())
        .accept(DOCS_ALPN, docs.clone())
        .spawn();

    // join the gossip topic by connecting to known peers, if any
    let peer_ids: Vec<NodeId> = peers.iter().map(|p| p.node_id).collect();

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

    // Testing notes interface
    // Stores the doc id in the config and makes a new one
    // if it is not there.
    // create a base author (not default )
    let base_author = match conf.get_notes_author() {
        Ok(id) => AuthorId::from(id),
        Err(_) => {
            let new_author = docs.author_create().await.unwrap();
            let _ = conf.set_author_key("notes", new_author.to_bytes());
            new_author
        }
    };
    let base_notes = match conf.get_notes_id() {
        Ok(id) => notes::Notes::from_id(id, base_author, blobs.clone(), docs.clone())
            .await
            .unwrap(),
        Err(_) => {
            let n = notes::Notes::new(None, base_author, blobs.clone(), docs.clone())
                .await
                .unwrap();
            let _ = conf.set_docs_key("notes", n.id());
            n
        }
    };

    // Some notes noodling
    
    // base_notes.create("_meta".to_string(),"meta".to_string()).await.unwrap();
    // base_notes.add("test".to_string(),"test_data".to_string()).await.unwrap();
    // base_notes.add("chicken wings".to_string(),"MMM tasty".to_string()).await.unwrap();
    // Some fixes
    // base_notes.fix_title("".to_string()).await.unwrap();
    // let val = base_notes.delete_hidden().await;
    // println!("{:#?}", val);
    // //let val = base_notes.bounce_down().await;
    // let h = Hash::from_str("c7c8b609d602b156d9a485ee7d3c543c4f31da255e12177cc88a5d4e10da7d3c")?;
    // let val = base_notes.bounce_up(h).await;
    // println!("{:#?}", val);

    // Set liminal, hashed as the topic
    let topic = TopicId::from_bytes(*Hash::new("liminal::").as_bytes());

    // let (sender, receiver) = gossip.subscribe(topic, peer_ids.clone()).await?.split();

    // Replica gossip
    let mut replica =
        replicate::ReplicaGossip::new(topic, blobs.clone(), gossip.clone(), peer_ids.clone())
            .await
            .unwrap();

    replica.run().await?;

    // Move all this into the replicate
    // subscribe and print loop
    // TODO : this should be a construct
    // task::spawn(replicate::subscribe_loop(receiver, blobs.clone()));

    // task::spawn(replicate::publish_loop(
    //     sender,
    //     blobs.clone(),
    //     endpoint.secret_key().clone(),
    // ));

    // Web interface

    if args.web {
        let rocket_secret_key: [u8; 32] = conf.rocket_key().unwrap();
        println!("starting web server ");
        // start the web server
        let figment = rocket::Config::figment()
            .merge(("address", "0.0.0.0"))
            .merge(("port", 8080))
            .merge(("secret_key", rocket_secret_key))
            .merge(("log_level", "critical"));

        let _result = rocket::custom(figment)
            .manage(base_notes.clone())
            .manage(fileset.clone())
            .manage(blobs.clone())
            .register("/",catchers![web::auth::unauthorized])
            .attach(web::stage())
            .attach(web::assets::stage())
            .attach(web::services::stage())
            .attach(web::notes::stage())
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
