use std::{
    collections::HashMap,
    fmt,
    net::{Ipv4Addr, SocketAddrV4},
    str::FromStr,
};

use bytes::Bytes;
use clap::Parser;
use ed25519_dalek::Signature;
//use futures_lite::StreamExt;
use iroh::{Endpoint, NodeAddr, PublicKey, RelayMode, RelayUrl, SecretKey};
use iroh_blobs::{ALPN as BLOBS_ALPN, store::fs::FsStore};
use iroh_gossip::{
    api::{Event, GossipReceiver},
    net::{GOSSIP_ALPN, Gossip},
    proto::TopicId,
};
use n0_future::task;
use n0_snafu::{Result, ResultExt};
use n0_watcher::Watcher;
use serde::{Deserialize, Serialize};
use snafu::whatever;
use std::path::PathBuf;
use tokio::{signal::ctrl_c, sync::mpsc};
use n0_future::{stream,StreamExt};

mod templates;
mod web;

#[macro_use]
extern crate rocket;

/// Chat over iroh-gossip
///
/// This broadcasts signed messages over iroh-gossip and verifies signatures
/// on received messages.
///
/// By default a new node id is created when starting the example. To reuse your identity,
/// set the `--secret-key` flag with the secret key printed on a previous invocation.
///
/// By default, the relay server run by n0 is used. To use a local relay server, run
///     cargo run --bin iroh-relay --features iroh-relay -- --dev
/// in another terminal and then set the `-d http://localhost:3340` flag on this example.
#[derive(Parser, Debug)]
struct Args {
    /// secret key to derive our node id from.
    #[clap(long)]
    secret_key: Option<String>,
    /// Set a custom relay server. By default, the relay server hosted by n0 will be used.
    #[clap(short, long)]
    relay: Option<RelayUrl>,
    /// Disable relay completely.
    #[clap(long)]
    no_relay: bool,
    /// Activate the web server
    #[clap(short, long)]
    web: bool,
    /// Set your nickname.
    #[clap(short, long)]
    name: Option<String>,
    /// Set the bind port for our socket. By default, a random port will be used.
    #[clap(short, long, default_value = "0")]
    bind_port: u16,
    #[clap(subcommand)]
    command: Command,
}

#[derive(Parser, Debug)]
enum Command {
    /// Open a chat room for a topic and print a ticket for others to join.
    ///
    /// If no topic is provided, a new topic will be created.
    Open {
        /// Optionally set the topic id (64 bytes, as hex string).
        topic: Option<TopicId>,
    },
    /// Join a chat room from a ticket.
    Join {
        /// The ticket, as base32 string.
        ticket: String,
    },
    Upload {
        path: Option<PathBuf>,
    },
}

use crate::templates::HomePageTemplate;
use rocket::response::Responder;

#[get("/")]
fn index<'r>() -> impl Responder<'r, 'static> {
    HomePageTemplate {}
}

#[rocket::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

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
        Command::Upload { path } => {
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
            if let Some(p) = path{ 
                println!("{:?}",p.display());
                store.add_path(p).await?;
            }
        },
        _ => {}
    }
        
    let mut t = store.tags().list().await.unwrap();
    while let Some(event) = t.next().await{
        println!("tags {:?}",event);
    }

    // setup router
    let _router = iroh::protocol::Router::builder(endpoint.clone())
        .accept(GOSSIP_ALPN, gossip.clone())
        .accept(BLOBS_ALPN, blobs)
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

    // subscribe and print loop
    task::spawn(subscribe_loop(receiver));

    // spawn an input thread that reads stdin
    // not using tokio here because they recommend this for "technical reasons"
    // let (line_tx, mut line_rx) = tokio::sync::mpsc::channel(1);
    // std::thread::spawn(move || input_loop(line_tx));

    // // broadcast each line we type
    // println!("> type a message and hit enter to broadcast...");
    // while let Some(text) = line_rx.recv().await {
    //     let message = Message::Message { text: text.clone() };
    //     let encoded_message = SignedMessage::sign_and_encode(endpoint.secret_key(), &message)?;
    //     sender.broadcast(encoded_message).await?;
    //     println!("> sent: {text}");
    // }
    // // shutdown
    // router.shutdown().await.e()?;
    if args.web {
        println!("starting web server ");
        // start the web server
        let figment = rocket::Config::figment()
            .merge(("address", "0.0.0.0"))
            .merge(("port", 8080))
            .merge(("log", "normal"));

        let _result = rocket::custom(figment)
            .manage(sender)
            .mount("/", routes![index])
            .mount("/", routes![web::fixed::dist])
            .launch()
            .await;
    } else { 
         ctrl_c().await.unwrap();
    }
    Ok(())
}

async fn subscribe_loop(mut receiver: GossipReceiver) -> Result<()> {
    // init a peerid -> name hashmap
    let mut names = HashMap::new();
    while let Some(event) = receiver.try_next().await? {
        if let Event::Received(msg) = event {
            let (from, message) = SignedMessage::verify_and_decode(&msg.content)?;
            match message {
                Message::AboutMe { name } => {
                    names.insert(from, name.clone());
                    println!("> {} is now known as {}", from.fmt_short(), name);
                }
                Message::Message { text } => {
                    let name = names
                        .get(&from)
                        .map_or_else(|| from.fmt_short(), String::to_string);
                    println!("{name}: {text}");
                }
            }
        }
    }
    Ok(())
}

fn input_loop(line_tx: tokio::sync::mpsc::Sender<String>) -> Result<()> {
    let mut buffer = String::new();
    let stdin = std::io::stdin(); // We get `Stdin` here.
    loop {
        stdin.read_line(&mut buffer).e()?;
        line_tx.blocking_send(buffer.clone()).e()?;
        buffer.clear();
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SignedMessage {
    from: PublicKey,
    data: Bytes,
    signature: Signature,
}

impl SignedMessage {
    pub fn verify_and_decode(bytes: &[u8]) -> Result<(PublicKey, Message)> {
        let signed_message: Self = postcard::from_bytes(bytes).e()?;
        let key: PublicKey = signed_message.from;
        key.verify(&signed_message.data, &signed_message.signature)
            .e()?;
        let message: Message = postcard::from_bytes(&signed_message.data).e()?;
        Ok((signed_message.from, message))
    }

    pub fn sign_and_encode(secret_key: &SecretKey, message: &Message) -> Result<Bytes> {
        let data: Bytes = postcard::to_stdvec(&message).e()?.into();
        let signature = secret_key.sign(&data);
        let from: PublicKey = secret_key.public();
        let signed_message = Self {
            from,
            data,
            signature,
        };
        let encoded = postcard::to_stdvec(&signed_message).e()?;
        Ok(encoded.into())
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum Message {
    AboutMe { name: String },
    Message { text: String },
}

#[derive(Debug, Serialize, Deserialize)]
struct Ticket {
    topic: TopicId,
    peers: Vec<NodeAddr>,
}
impl Ticket {
    /// Deserializes from bytes.
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        postcard::from_bytes(bytes).e()
    }
    /// Serializes to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        postcard::to_stdvec(self).expect("postcard::to_stdvec is infallible")
    }
}

/// Serializes to base32.
impl fmt::Display for Ticket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut text = data_encoding::BASE32_NOPAD.encode(&self.to_bytes()[..]);
        text.make_ascii_lowercase();
        write!(f, "{text}")
    }
}

/// Deserializes from base32.
impl FromStr for Ticket {
    type Err = n0_snafu::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = data_encoding::BASE32_NOPAD
            .decode(s.to_ascii_uppercase().as_bytes())
            .e()?;
        Self::from_bytes(&bytes)
    }
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
