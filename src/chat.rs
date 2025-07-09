use std::{
    collections::HashMap,
    fmt,
    net::{Ipv4Addr, SocketAddrV4},
    str::FromStr,
    time::Duration,
};

use bytes::Bytes;
use chrono::Local;
use iroh::{NodeAddr, PublicKey, SecretKey};
use iroh_blobs::{
    BlobFormat, Hash, HashAndFormat, api::Store, net_protocol::Blobs, store::fs::FsStore,
};
use iroh_gossip::{
    api::{Event, GossipReceiver, GossipSender},
    net::{GOSSIP_ALPN, Gossip},
    proto::TopicId,
};

use ed25519_dalek::Signature;

use n0_future::StreamExt;
use n0_snafu::{Result, ResultExt};
use serde::{Deserialize, Serialize};

pub async fn subscribe_loop(mut receiver: GossipReceiver, blobs: Blobs) -> Result<()> {
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
                Message::Upkey { key } => {
                    let conn = blobs
                        .endpoint()
                        .connect(msg.delivered_from, iroh_blobs::protocol::ALPN)
                        .await?;
                    //if blobs.store().has(key).await.expect("") == false {
                    println!("a new key {:?}", key);
                    // thes are hashseq not raw
                    let knf = HashAndFormat::new(key, BlobFormat::HashSeq);
                    let r = blobs.store().remote().fetch(conn, knf).await?;
                    println!("{:?}", r);
                    let dt = Local::now().to_rfc3339().to_owned();
                    blobs.store().tags().set(format!("col-{}", dt), key).await?;
                    //}
                }
            }
        }
    }
    Ok(())
}

pub async fn publish_loop(mut sender: GossipSender, blobs: Blobs, secret: SecretKey) -> Result<()> {
    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;
        println!("boop");
        let mut t = blobs.store().tags().list_prefix("col").await.unwrap();
        while let Some(event) = t.next().await {
            match event {
                Ok(tag) => {
                    let message = Message::Upkey { key: tag.hash };
                    println!("Sending --- {:?}", &message);
                    let encoded_message =
                        SignedMessage::sign_and_encode(&secret, &message)?;
                    sender.broadcast(encoded_message).await?;
                }
                Err(_) => todo!(),
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}

// pub fn input_loop(line_tx: tokio::sync::mpsc::Sender<String>) -> Result<()> {
//     let mut buffer = String::new();
//     let stdin = std::io::stdin(); // We get `Stdin` here.
//     loop {
//         stdin.read_line(&mut buffer).e()?;
//         line_tx.blocking_send(buffer.clone()).e()?;
//         buffer.clear();
//     }
// }

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

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    AboutMe { name: String },
    Message { text: String },
    Upkey { key: Hash },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ticket {
    pub topic: TopicId,
    pub peers: Vec<NodeAddr>,
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
