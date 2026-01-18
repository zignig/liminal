///! A web interface to iroh and friends.
///! Using rocket and tokio
///! Testing ground docs,blobs and gossip
///! It should be a usefull interface
use std::{
    net::{Ipv4Addr, SocketAddrV4},
    str::FromStr,
};

use clap::Parser;
use iroh::{Endpoint, EndpointId, RelayMode, SecretKey};
use iroh_blobs::{ALPN as BLOBS_ALPN, Hash, store::fs::FsStore};
use iroh_docs::{ALPN as DOCS_ALPN, AuthorId, protocol::Docs};
use iroh_gossip::{
    net::{GOSSIP_ALPN, Gossip},
    proto::TopicId,
};
use iroh_tickets::endpoint::EndpointTicket;
use n0_future::StreamExt;
use n0_snafu::{Result, ResultExt, format_err};
use n0_watcher::Watcher;
use std::path::PathBuf;
use tokio::signal::ctrl_c;

mod cli;
mod config;
// mod fren;
mod notes;
mod replicate;
mod store;
mod templates;
mod web;

use cli::Command;
use cli::Ticket;
// use fren::FrenApi;
// use crate::fren::FREN_ALPN;

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
        Err(e) => return Err(format_err!("{} bad database!", e)),
    };

    // --random cli entry will generate a new node id
    // Or use a fixed one from config
    let secret_key = match &args.random {
        true => SecretKey::generate(&mut rand::rng()),
        false => match conf.get_secret_key() {
            Ok(secret) => secret.to_owned(),
            Err(_) => return Err(format_err!("Bad secret")),
        },
    };

    // build our magic endpoint
    let endpoint = iroh::Endpoint::builder()
        .secret_key(secret_key.clone())
        .relay_mode(RelayMode::Default)
        .bind()
        .await
        .unwrap();

    // this needs to have a timeout
    endpoint.online().await;
    let node_ticket = iroh_tickets::endpoint::EndpointTicket::new(endpoint.addr());
    println!("ticket \n\n {:?}", node_ticket.to_string());

    // Stash some nodes
    let _ = conf.add_node(endpoint.id());
    let _ = conf.list_nodes();

    // print a ticket that includes our own node id and endpoint addresses
    // let ticket = {
    //     let me = endpoint.id();
    //     let peers = peers.iter().cloned().chain([me]).collect();
    //     Ticket { peers }
    // };

    // println!("\n\n> ticket to join us: {ticket}");

    // create the gossip protocol
    let gossip = Gossip::builder().spawn(endpoint.clone());

    // BLOBS!
    let path = PathBuf::from("data/blobs");
    let store = FsStore::load(path).await.unwrap();
    let blobs = iroh_blobs::BlobsProtocol::new(&store, None);

    // Path browser
    let fileset = store::FileSet::new(blobs.clone());
    // TODO make this prettier.
    // Get the file roots
    fileset.fill("col").await;
    // fileset.fill("archive").await;
    // fileset.fill("notes").await;

    // clear out some old tags ( carefull )
    //fileset.del_tags("col-17").await.unwrap();

    // DOCS !
    let docs_path = PathBuf::from("data/");
    let docs = Docs::persistent(docs_path)
        .spawn(endpoint.clone(), (*blobs).clone(), gossip.clone())
        .await
        .unwrap();

    let mut d = docs.list().await.unwrap();
    while let Some(docs) = d.next().await {
        let docs = docs.unwrap();
        println!("{:#?}", docs);
    }

    // FREN !
    // let fren_api = FrenApi::spawn();

    // setup router
    let router = iroh::protocol::Router::builder(endpoint.clone())
        .accept(GOSSIP_ALPN, gossip.clone())
        .accept(BLOBS_ALPN, blobs.clone())
        .accept(DOCS_ALPN, docs.clone())
        // .accept(FREN_ALPN, fren_api.expose().unwrap())
        .spawn();

    // // make sure we are connected for tickets
    // let addr = router.endpoint().node_addr().initialized().await;
    // let node_ticket = NodeTicket::new(addr);
    // // join the gossip topic by connecting to known peers, if any
    // let peer_ids: Vec<NodeId> = peers.iter().map(|p| p.node_id).collect();

    // if peers.is_empty() {
    //     println!("> waiting for peers to join us...");
    // } else {
    //     println!("> trying to connect to {} peers...", peers.len());
    //     // add the peer addrs from the ticket to our endpoint's addressbook so that they can be dialed
    //     for peer in peers.into_iter() {
    //         // Stash the peers in the config
    //         let _ = conf.add_node(peer.id);
    //         endpoint.add(peer)?;
    //     }
    // };

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

    // during testing , clean out the docs users
    // with random notes joining , it can get messy.
    // let _e = base_notes.leave().await;
    // let _e = base_notes.share().await;
    let _e = base_notes.run().await;

    // let val = base_notes.delete_hidden().await;
    // println!("{:#?}", val);

    let val = base_notes.bounce_down().await;
    println!("{:#?}", val);

    // let val = base_notes.bounce_up("notes-1759074698").await;
    // println!("{:#?}", val);

    // Set liminal, hashed as the topic

    let peer_ids = vec![];
    let topic = TopicId::from_bytes(*Hash::new("liminal::").as_bytes());
    let repl = replicate::Replicator::new(
        gossip.clone(),
        blobs.clone(),
        topic,
        peer_ids,
        secret_key.clone(),
        vec!["col".to_string(), "notes".to_string(),"archive".to_string()],
    )
    .await?;
    repl.run().await?;

    let mut lit = docs.list().await.unwrap();
    while let Some(d) = lit.next().await {
        let d = d.unwrap();
        println!("{:#?}", d);
    }
    // Web interface
    // println!("{}", node_ticket);
    if args.web {
        let rocket_secret_key: [u8; 32] = conf.rocket_key().unwrap();
        println!("starting web server ");
        // start the web server
        let figment = rocket::Config::figment()
            .merge(("address", "0.0.0.0"))
            .merge(("port", 8080))
            .merge(("secret_key", rocket_secret_key))
            .merge(("log_level", "critical"))
            .merge(("cli_colors", "false"));

        let _result = rocket::custom(figment)
            .manage(base_notes.clone())
            .manage(fileset.clone())
            .manage(blobs.clone())
            .manage(endpoint.clone())
            .manage(docs.clone())
            .register("/", catchers![web::auth::unauthorized])
            .attach(web::stage())
            .attach(web::assets::stage())
            .attach(web::services::stage())
            .attach(web::notes::stage())
            .attach(web::replica::stage())
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
