//! A wasm cloud attached to an Iroh node.
mod trace;
mod wash_host;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use wash_host::WasmHost;

// Local tracing
// use tracing::info;

// use crate::workloads::Dispatcher;

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

    let _wh = if args.run {
        let wh = WasmHost::new().await?;
        if let Some(path) = args.input {
            let _ = wh.upload(path, args.service).await;
        }
        wh.startup().await?;
        Some(wh)
    } else {
        None
    };
    tokio::signal::ctrl_c().await?;
    Ok(())
}
