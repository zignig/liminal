//! A wasm cloud attached to an Iroh node.
mod trace;
// mod wash_host;
mod auth; 

use auth::AuthService;
use anyhow::Result;
use clap::Parser;
use tracing::warn;
use std::path::PathBuf;

// Faster Compile times ( remove wask host for now)
// use wash_host::WasmHost;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to wasm
    input: Option<PathBuf>,
    #[arg(short, long, action = clap::ArgAction::SetTrue, default_value_t = false)]
    service: bool,
    #[arg(short, long, action = clap::ArgAction::SetTrue, default_value_t = false)]
    run: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup and start some logging.
    trace::start_logging();

    
    // Get the args and print them
    let args = Args::parse();
    println!("{:#?}", &args);

    let _auth = AuthService::new().await?;

    // Create the WASM engine ( needs config to go into it )

    // CHOP
    // let wh = WasmHost::new().await?;
    
    // if let Some(path) = args.input {
    //     let _ = wh.upload(path, args.service).await;
    // }

    // wh.startup().await?;
    // UNCHOP

    tokio::signal::ctrl_c().await?;

    warn!("Exit Program");
    Ok(())
}
