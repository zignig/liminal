//! This is a gossip channel that keeps a list of hashes
//!
//! TODO : it needs a Downloader and an Actor loop
//!

use std::{sync::Arc, time::Duration};

use bytes::Bytes;
use chrono::Local;
use dashmap::DashMap;
use iroh::{NodeAddr, PublicKey, SecretKey};
use iroh_blobs::{BlobsProtocol, Hash, format::collection::Collection, hashseq::HashSeq};

use iroh_docs::NamespaceId;
use iroh_gossip::{
    api::{Event, GossipReceiver, GossipSender},
    net::Gossip,
    proto::TopicId,
};

use ed25519_dalek::Signature;

use n0_future::StreamExt;
use n0_snafu::{Result, ResultExt};
use serde::{Deserialize, Serialize};
use tokio::task;

// These are the messages that the replica can send
#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Whohas { key: Hash },
    IHave { key: Hash },
    Message { text: String },
    Upkey { key: Hash },
    Document { key: NamespaceId },
}

// TODO , this is wrong need to read some other systems
// and work out the best way.

#[derive(Debug)]
pub struct ReplicaGossip {
    topic: TopicId,
    gossip: Gossip,
    blobs: BlobsProtocol,
    roots: DashMap<Hash, Vec<NodeAddr>>,
    expire: DashMap<u64, Hash>,
    peers : Vec<PublicKey>
}

impl ReplicaGossip {
    pub async fn new(
        topic: TopicId,
        blobs: BlobsProtocol,
        gossip: Gossip,
        peers: Vec<PublicKey>,
    ) -> Result<Self> {

        Ok(Self {
            topic: topic,
            gossip: gossip,
            blobs: blobs.clone(),
            roots: DashMap::new(),
            expire: DashMap::new(),
            peers: peers
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let (sender, receiver) = self.gossip.subscribe(self.topic, self.peers.clone()).await?.split();
        // Start the receiver
        task::spawn(subscribe_loop(receiver));

        Ok(())
    }
}

pub async fn subscribe_loop(mut receiver: GossipReceiver) -> Result<()> {
    // init a peerid -> name hashmap
    while let Some(event) = receiver.try_next().await? {
        if let Event::Received(msg) = event {
            let (_, message) = SignedMessage::verify_and_decode(&msg.content)?;
            match message {
                Message::Whohas { key } => println!("whohas"),
                Message::IHave { key } => println!("ihave"),
                Message::Message { text } => println!("message"),
                Message::Upkey { key } =>println!("uplkey"),
                Message::Document { key } =>println!("document"),
            }
        }
    }
    Ok(()) 
}


pub async fn subscribe_loop_old(mut receiver: GossipReceiver, blobs: BlobsProtocol) -> Result<()> {
    // init a peerid -> name hashmap
    while let Some(event) = receiver.try_next().await? {
        if let Event::Received(msg) = event {
            let (_, message) = SignedMessage::verify_and_decode(&msg.content)?;
            match message {
                Message::Message { text } => {
                    println!("{text}");
                }
                Message::Upkey { key } => {
                    let have = blobs.store().has(key).await.expect("no key");
                    if !have {
                        println!("Fetching key : {}", key);
                        let conn = blobs
                            .endpoint()
                            .connect(msg.delivered_from, iroh_blobs::protocol::ALPN)
                            .await?;
                        // fetch  the root key
                        // TODO : extract into function
                        blobs.store().remote().fetch(conn.clone(), key).await?;
                        let bl = blobs.store().get_bytes(key).await?;
                        let hs = HashSeq::try_from(bl).expect("hash fail");
                        let meta = hs.into_iter().next().context("empty has seq")?;
                        // fetch the meta data
                        blobs.store().remote().fetch(conn.clone(), meta).await?;
                        // tag it as collection
                        let dt = Local::now().to_rfc3339().to_owned();
                        blobs.store().tags().set(format!("col-{}", dt), key).await?;
                        // Check that it is a collection
                        Collection::load(key, blobs.store()).await.expect("woteva");
                        // Just get the whole thing

                        // let knf = HashAndFormat::new(key, BlobFormat::HashSeq);
                        // let local = blobs.store().remote().local(knf).await.expect("msg");
                        // if !local.is_complete() {
                        //     println!("a new key {:?}", key);
                        //     let r = blobs.store().remote().fetch(conn, knf).await?;
                        //     println!("{:?}", r);
                        //     let dt = Local::now().to_rfc3339().to_owned();
                        //     blobs.store().tags().set(format!("col-{}", dt), key).await?;
                        //     let col = Collection::load(key, blobs.store()).await.expect("woteva");
                        //     for (s, _) in col {
                        //         println!("{}", s);
                        //     }
                        // }
                    }
                    // for (s, h) in col {
                    //     println!("{} - {} ", s, h);
                    // }
                }
                Message::Whohas { key: _ } => {}
                Message::IHave { key: _ } => {}
                Message::Document { key: _ } => {}
            }
        }
    }
    Ok(())
}

pub async fn publish_loop(
    sender: GossipSender,
    blobs: BlobsProtocol,
    secret: SecretKey,
) -> Result<()> {
    loop {
        let mut t = blobs.store().tags().list_prefix("col").await.unwrap();
        while let Some(event) = t.next().await {
            match event {
                Ok(tag) => {
                    let message = Message::Upkey { key: tag.hash };
                    // println!("Sending --- {:?}", &message);
                    let encoded_message = SignedMessage::sign_and_encode(&secret, &message)?;
                    sender.broadcast(encoded_message).await?;
                }
                Err(_) => todo!(),
            }
            // tokio::time::sleep(Duration::from_secs(1)).await;
        }
        // TODO : make this self aligning.
        tokio::time::sleep(Duration::from_secs(20)).await;
    }
}

// Stolen from CHAT.
// Message Structs
#[derive(Debug, Serialize, Deserialize)]
pub struct SignedMessage {
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

