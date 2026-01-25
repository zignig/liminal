mod gossip;
mod interface;

use std::time::Duration;

use iroh::{EndpointId, SecretKey};
use iroh_blobs::{BlobsProtocol, Hash};
use iroh_gossip::{Gossip, TopicId};
use iroh_gossip::api::Event;
use n0_error::AnyError;
use n0_future::StreamExt;
use n0_snafu::Result;
use uuid::Uuid;

use tokio::task;

use interface::FinderApi;

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
    // more here later ( for publishing and user identification)
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
        task::spawn(runner(self.gossip.clone(), self.topic));
    }
}

pub async fn runner(gossip: Gossip, topic: TopicId) -> Result<(), AnyError> {
    let goss = gossip
        .subscribe_and_join(topic, vec![])
        .await
        .expect("bad goss");
    let (tx, mut rx) = goss.split();
    tx.broadcast("it's a me".as_bytes().into()).await?;
    println!("GOSSIP GO");
    while let Some(event) = rx.try_next().await? {
        // println!("gossip event--> {:#?}", event);
        match event {
            Event::NeighborUp(public_key) => println!("NeighborUp {:?}",public_key),
            Event::NeighborDown(public_key) => println!("NeighborDown {:?}",public_key),
            Event::Received(message) => {
                let content = message.content;
                println!("message -> {:?}",content);
            },
            Event::Lagged => println!("Lagged!!")
        }
    }
    println!("GOSSIP EXIT");
    Ok(())
}
