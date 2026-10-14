mod protocol;
mod server;
mod service;

use anyhow::Result;
use clap::Parser;
use innernet_shared::{HostsOpts, NatOpts, NetworkOpts};
use server::{resolve_socket_path, run_server};
use service::DaemonService;
use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::sync::broadcast;

#[derive(Parser, Debug)]
#[command(
    name = "innernetd",
    author = "Tonari & Contributors",
    version,
    about = "Background daemon for innernet interfaces exposing a local IPC socket."
)]
struct Args {
    /// Custom path for the Unix domain socket.
    #[clap(short, long)]
    socket_path: Option<PathBuf>,

    /// Innernet configuration directory.
    #[clap(short, long, default_value = innernet_client_core::DEFAULT_CONFIG_DIR)]
    config_dir: PathBuf,

    /// Innernet data directory.
    #[clap(short, long, default_value = innernet_client_core::DEFAULT_DATA_DIR)]
    data_dir: PathBuf,

    /// Interval in seconds for background peer refresh.
    #[clap(long, default_value = "60")]
    interval: u64,

    #[clap(flatten)]
    network: NetworkOpts,

    #[clap(flatten)]
    hosts: HostsOpts,

    #[clap(flatten)]
    nat: NatOpts,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Args::parse();
    let socket_path = resolve_socket_path(args.socket_path.as_deref());

    log::info!("Starting innernetd version {}", env!("CARGO_PKG_VERSION"));
    log::info!("Config dir: {}", args.config_dir.display());
    log::info!("Data dir: {}", args.data_dir.display());

    let service = Arc::new(DaemonService::new(
        args.config_dir,
        args.data_dir,
        args.network.backend,
        args.hosts,
        args.nat,
    ));

    // Start background sync task
    service
        .clone()
        .spawn_refresher(Duration::from_secs(args.interval));

    let (shutdown_tx, shutdown_rx) = broadcast::channel::<()>(1);

    // Handle shutdown signal
    let shutdown_tx_clone = shutdown_tx.clone();
    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        log::info!("Received interrupt signal, initiating shutdown...");
        let _ = shutdown_tx_clone.send(());
    });

    run_server(socket_path, service, shutdown_rx).await?;

    Ok(())
}
