// ticket for the replicator

use iroh_gossip::TopicId;
use iroh_tickets::Ticket;

//

#[derive(Debug, Clone, PartialEq, Eq, derive_more::Display)]
#[display("{}", Ticket::serialize(self))]
pub struct ReplicaTicket {
    addr: NodeId,
    topic: TopicId,
    prefixes: Vec<String>,
}

impl Ticket for ReplicaTicket {
    const KIND: &'static str = "replicas";

    fn to_bytes(&self) -> Vec<u8> {
        let data = 
        postcard::to_stdvec(&data).expect("postcard serialition failed")
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, iroh_tickets::ParseError> {
        todo!()
    }
}
