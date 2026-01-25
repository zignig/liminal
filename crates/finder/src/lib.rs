mod gossip;
mod interface;

use std::time::Duration;

use bytes::Bytes;
use iroh::Signature;
use iroh::{EndpointId, PublicKey, SecretKey};
use iroh_blobs::{BlobsProtocol, Hash};
use iroh_gossip::api::Event;
use iroh_gossip::{Gossip, TopicId};
use n0_error::AnyError;
use n0_future::StreamExt;
use n0_snafu::{Result, ResultExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use tokio::task;

use interface::FinderApi;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FinderMessage {
    WhoHas {
        transactionid: Uuid,
        hash: Hash,
    },
    IHave {
        transactionid: Uuid,
        endpoint: EndpointId,
    },
    UserQuery {
        endpoint: EndpointId,
    },
    Beacon {
        mess: String,
    }, // more here later ( for publishing and user identification)
}

pub struct Finder {
    topic: TopicId,
    blobs: BlobsProtocol,
    gossip: Gossip,
    secret_key: SecretKey,
    api: FinderApi,
}

impl Finder {
    pub fn new(
        topic: TopicId,
        blobs: BlobsProtocol,
        gossip: Gossip,
        secret_key: SecretKey,
    ) -> Self {
        let api = FinderApi::new();
        Self {
            topic,
            blobs,
            gossip,
            secret_key,
            api,
        }
    }

    pub async fn spawn(&self) {
        // Set up the main worker
        // gossip
        //
        task::spawn(runner(
            self.gossip.clone(),
            self.topic,
            self.secret_key.clone(),
        ));
    }
}

pub async fn runner(gossip: Gossip, topic: TopicId, secret_key: SecretKey) -> Result<(), AnyError> {
    let goss = gossip
        .subscribe_and_join(topic, vec![])
        .await
        .expect("bad goss");
    let (tx, mut rx) = goss.split();
    match SignedMessage::sign_and_encode(
        &secret_key,
        &FinderMessage::Beacon {
            mess: "test".to_string(),
        },
    ) {
        Ok(mess) => tx.broadcast(mess).await?,
        Err(e) => {
            println!("{:}", e)
        }
    }
    println!("GOSSIP GO");
    while let Some(event) = rx.try_next().await? {
        // println!("gossip event--> {:#?}", event);
        match event {
            Event::NeighborUp(public_key) => println!("NeighborUp {:?}", public_key),
            Event::NeighborDown(public_key) => println!("NeighborDown {:?}", public_key),
            Event::Received(message) => {
                let content = message.content;
                println!("message -> {:?}", content);
            }
            Event::Lagged => println!("Lagged!!"),
        }
    }
    println!("GOSSIP EXIT");
    Ok(())
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
    pub fn verify_and_decode(bytes: &[u8]) -> Result<(PublicKey, FinderMessage)> {
        let signed_message: Self = postcard::from_bytes(bytes).e()?;
        let key: PublicKey = signed_message.from;
        key.verify(&signed_message.data, &signed_message.signature)
            .e()?;
        let message: FinderMessage = postcard::from_bytes(&signed_message.data).e()?;
        Ok((signed_message.from, message))
    }

    pub fn sign_and_encode(secret_key: &SecretKey, message: &FinderMessage) -> Result<Bytes> {
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
