use std::path::PathBuf;

use clap::Parser;
use iroh::RelayUrl;
use iroh_gossip::proto::TopicId;

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
pub struct Args {
    /// secret key to derive our node id from.
    #[clap(long)]
    pub secret_key: Option<String>,
    /// Set a custom relay server. By default, the relay server hosted by n0 will be used.
    #[clap(short, long)]
    pub relay: Option<RelayUrl>,
    /// Disable relay completely.
    #[clap(long)]
    pub no_relay: bool,
    /// Activate the web server
    #[clap(short, long)]
    pub web: bool,
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
