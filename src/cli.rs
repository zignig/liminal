use std::{fmt, str::FromStr};

use clap::Parser;
use iroh::NodeAddr;
use n0_snafu::ResultExt;
use n0_snafu::Result;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
pub struct Args {
    // Random node id 
    #[clap(short,long)]
    pub random: bool,
    #[clap(short, long)]
    pub web: bool,
    /// Do a full replica of all the data 
    #[clap(short, long)]
    pub duplicate: bool,
    /// Set your nickname.
    #[clap(short, long)]
    pub name: Option<String>,
    /// Set the bind port for our socket. By default, a random port will be used.
    #[clap(short, long, default_value = "0")]
    pub bind_port: u16,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Parser, Debug)]
pub enum Command {
    Open,
    /// Join a chat room from a ticket.
    Join {
        /// The ticket, as base32 string
        /// Just a node , assumes that it has gossip and is listening for "liminal::"
        ticket: String,
    }
}

// Base ticket join ( just node address for now)
#[derive(Debug, Serialize, Deserialize)]
pub struct Ticket {
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
