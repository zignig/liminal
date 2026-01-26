use iroh::{Endpoint, EndpointId, SecretKey, protocol::Router};
use iroh_blobs::Hash;
use iroh_gossip::{TopicId, api::GossipReceiver, net::Gossip};
use n0_error::{Result, StdResultExt};
use n0_future::StreamExt;

use serde::{Deserialize, Serialize};
use std::{process, time::Duration};
use tracing::{error, warn};

// use finder::Finder;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    topic: String,
    mother_ship: Option<EndpointId>,
    secret: SecretKey,
}

impl Config {
    const FILE_NAME: &str = "settings.toml";
    fn load() -> Config {
        let content = match std::fs::read_to_string(Config::FILE_NAME) {
            Ok(content) => content,
            Err(e) => {
                error!("config file borked, {} ", e);
                process::exit(1);
            }
        };
        let config: Config = toml::from_str(&content).expect("borked encoding");
        config
    }

    fn save(&self) {
        let contents = toml::to_string(&self).expect("borked config");
        std::fs::write(Config::FILE_NAME, contents).expect("borked file");
    }

    fn new(id: EndpointId, secret: SecretKey) {
        let config = Config {
            topic: "finder".to_string(),
            mother_ship: Some(id),
            secret: secret,
        };
        config.save();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    warn!("awesome");
    let config = Config::load();

    let endpoint = Endpoint::builder()
        .secret_key(config.secret.clone())
        .bind()
        .await?;

    // build the blobs
    let blob_store = iroh_blobs::store::mem::MemStore::new();
    let blobs = iroh_blobs::BlobsProtocol::new(&blob_store, None);

    // build gossip protocol
    let gossip = Gossip::builder().spawn(endpoint.clone());

    // setup router
    let router = Router::builder(endpoint.clone())
        .accept(iroh_blobs::ALPN, blobs.clone())
        .accept(iroh_gossip::ALPN, gossip.clone())
        .spawn();

    
    let _ = endpoint.online().await;
    println!("{:#?}", &endpoint.id());
    // Config::new(endpoint.id(),endpoint.secret_key().clone());
    if let Some(mother_ship) = config.mother_ship {
        let my_topic = make_topic(config.topic.as_str());
        let goss = gossip
            .subscribe_and_join(my_topic, vec![mother_ship])
            .await?;
        let (tx, rx) = goss.split();
        tokio::task::spawn(subscribe_loop(rx));
        println!("GOSSIP GO!! {:#?}", my_topic);
        let mut counter = 0u32;
        loop {
            let mut data: Vec<u8> = vec![0; 32];
            rand::fill(&mut data[..]);
            println!("{:} -> {:?}", counter, data);
            counter += 1;
            tx.broadcast(data.into()).await?;
            // tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }

    // tokio::signal::ctrl_c().await?;

    router.shutdown().await.std_context("shutdown router")?;
    Ok(())
}

pub async fn subscribe_loop(mut receiver: GossipReceiver) -> Result<()> {
    // init a peerid -> name hashmap
    while let Some(event) = receiver.try_next().await? {
        println!("{:#?}", event);
    }
    Ok(())
}

pub fn make_topic(name: &str) -> TopicId {
    TopicId::from_bytes(*Hash::new(name).as_bytes())
}
