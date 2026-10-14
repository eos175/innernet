use crate::protocol::{InterfaceStatus, NetworkSummary, PeerDetail};
use anyhow::{anyhow, Context, Result};
use innernet_client_core::{
    data_store::DataStore,
    interface::{fetch, interface_is_up, redeem_invite},
};
use innernet_shared::{
    interface_config::InterfaceConfig,
    wg, HostsOpts, NatOpts, NetworkOpts,
};
use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
    path::PathBuf,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::sync::RwLock;
use wireguard_control::{Backend, Device, InterfaceName};

/// Core service managing innernet interfaces and WireGuard state.
#[derive(Clone)]
pub struct DaemonService {
    config_dir: PathBuf,
    data_dir: PathBuf,
    backend: Backend,
    hosts_opts: HostsOpts,
    nat_opts: NatOpts,
    active_interfaces: Arc<RwLock<HashSet<String>>>,
}

impl DaemonService {
    pub fn new(
        config_dir: PathBuf,
        data_dir: PathBuf,
        backend: Backend,
        hosts_opts: HostsOpts,
        nat_opts: NatOpts,
    ) -> Self {
        Self {
            config_dir,
            data_dir,
            backend,
            hosts_opts,
            nat_opts,
            active_interfaces: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    fn build_network_opts(&self) -> NetworkOpts {
        NetworkOpts {
            backend: self.backend,
            mtu: None,
            no_routing: false,
        }
    }

    /// List all installed network configurations in the config directory.
    pub fn list_networks(&self) -> Result<Vec<NetworkSummary>> {
        if !self.config_dir.exists() {
            return Ok(Vec::new());
        }

        let entries = std::fs::read_dir(&self.config_dir)
            .with_context(|| format!("Failed to read config dir {}", self.config_dir.display()))?;

        let mut networks = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension() == Some(OsStr::new("conf")) {
                if let Some(stem) = path.file_stem() {
                    let iface_str = stem.to_string_lossy().to_string();
                    if let Ok(iface_name) = iface_str.parse::<InterfaceName>() {
                        let is_up = interface_is_up(self.backend, &iface_name);
                        let config = InterfaceConfig::from_interface(&self.config_dir, &iface_name).ok();
                        let address = config.as_ref().map(|c| c.interface.address.addr());
                        let listen_port = config.as_ref().and_then(|c| c.interface.listen_port);

                        networks.push(NetworkSummary {
                            interface: iface_str,
                            is_up,
                            address,
                            listen_port,
                        });
                    }
                }
            }
        }

        networks.sort_by(|a, b| a.interface.cmp(&b.interface));
        Ok(networks)
    }

    /// Get detailed status for an interface.
    pub fn get_status(&self, interface_str: &str) -> Result<InterfaceStatus> {
        let iface: InterfaceName = interface_str
            .parse()
            .map_err(|e| anyhow!("Invalid interface name '{}': {}", interface_str, e))?;

        let is_up = interface_is_up(self.backend, &iface);
        let config = InterfaceConfig::from_interface(&self.config_dir, &iface).ok();

        let mut status = InterfaceStatus {
            interface: interface_str.to_string(),
            is_up,
            address: config.as_ref().map(|c| c.interface.address.addr()),
            listen_port: config.as_ref().and_then(|c| c.interface.listen_port),
            public_key: None,
            server_endpoint: config
                .as_ref()
                .map(|c| c.server.external_endpoint.to_string()),
            peer_count: 0,
            active_peer_count: 0,
            total_rx_bytes: 0,
            total_tx_bytes: 0,
        };

        if let Ok(store) = DataStore::open_or_create(&self.data_dir, &iface) {
            status.peer_count = store.peers().len();
        }

        if is_up {
            if let Ok(device) = Device::get(&iface, self.backend) {
                status.public_key = device.public_key.map(|k| k.to_base64());
                if status.listen_port.is_none() {
                    status.listen_port = device.listen_port;
                }

                let now = SystemTime::now();
                let active_threshold = Duration::from_secs(180);

                for peer in &device.peers {
                    status.total_rx_bytes += peer.stats.rx_bytes;
                    status.total_tx_bytes += peer.stats.tx_bytes;

                    if let Some(last_handshake) = peer.stats.last_handshake_time {
                        if let Ok(elapsed) = now.duration_since(last_handshake) {
                            if elapsed <= active_threshold {
                                status.active_peer_count += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(status)
    }

    /// Bring up the interface and add it to the background refresh set.
    pub async fn up(&self, interface_str: &str) -> Result<()> {
        let iface: InterfaceName = interface_str
            .parse()
            .map_err(|e| anyhow!("Invalid interface name '{}': {}", interface_str, e))?;

        let network_opts = self.build_network_opts();

        fetch(
            &self.config_dir,
            &self.data_dir,
            &network_opts,
            &self.hosts_opts,
            &self.nat_opts,
            &iface,
            true,
        )?;

        let mut active = self.active_interfaces.write().await;
        active.insert(interface_str.to_string());
        log::info!("Interface {} brought up successfully", interface_str);
        Ok(())
    }

    /// Bring down the interface and remove it from background refresh.
    pub async fn down(&self, interface_str: &str) -> Result<()> {
        let iface: InterfaceName = interface_str
            .parse()
            .map_err(|e| anyhow!("Invalid interface name '{}': {}", interface_str, e))?;

        wg::down(&iface, self.backend)?;

        let mut active = self.active_interfaces.write().await;
        active.remove(interface_str);
        log::info!("Interface {} brought down successfully", interface_str);
        Ok(())
    }

    /// Force a peer and route synchronization from the coordination server.
    pub fn fetch_sync(&self, interface_str: &str) -> Result<()> {
        let iface: InterfaceName = interface_str
            .parse()
            .map_err(|e| anyhow!("Invalid interface name '{}': {}", interface_str, e))?;

        let network_opts = self.build_network_opts();

        fetch(
            &self.config_dir,
            &self.data_dir,
            &network_opts,
            &self.hosts_opts,
            &self.nat_opts,
            &iface,
            false,
        )?;

        log::info!("State synchronized for {}", interface_str);
        Ok(())
    }

    /// Query peers and merge with live WireGuard handshake / transfer statistics.
    pub fn get_peers(&self, interface_str: &str) -> Result<Vec<PeerDetail>> {
        let iface: InterfaceName = interface_str
            .parse()
            .map_err(|e| anyhow!("Invalid interface name '{}': {}", interface_str, e))?;

        let store = DataStore::open_or_create(&self.data_dir, &iface)
            .map_err(|e| anyhow!("Failed to open data store: {}", e))?;

        let is_up = interface_is_up(self.backend, &iface);
        let config = InterfaceConfig::from_interface(&self.config_dir, &iface).ok();
        let my_ip = config.as_ref().map(|c| c.interface.address.addr());
        let my_pubkey = config.as_ref().and_then(|c| {
            wireguard_control::Key::from_base64(&c.interface.private_key)
                .ok()
                .map(|k| k.get_public().to_base64())
        });

        let live_peers_map: HashMap<String, wireguard_control::PeerInfo> =
            if is_up {
                Device::get(&iface, self.backend)
                    .map(|d| {
                        d.peers
                            .into_iter()
                            .map(|p| (p.config.public_key.to_base64(), p))
                            .collect()
                    })
                    .unwrap_or_default()
            } else {
                HashMap::new()
            };

        let now = SystemTime::now();
        let active_threshold = Duration::from_secs(180);
        let mut result = Vec::new();

        for peer in store.peers() {
            if peer.is_disabled {
                continue;
            }

            let is_self = my_ip.map_or(false, |ip| ip == peer.ip)
                || my_pubkey.as_deref().map_or(false, |k| k == peer.public_key);

            let live = live_peers_map.get(&peer.public_key);
            let (last_handshake_time, is_connected, rx_bytes, tx_bytes) = if is_self {
                (None, is_up, 0, 0)
            } else if let Some(live) = live {
                let handshake_epoch = live.stats.last_handshake_time.and_then(|t| {
                    t.duration_since(UNIX_EPOCH).ok().map(|d| d.as_secs())
                });
                let connected = live
                    .stats
                    .last_handshake_time
                    .and_then(|t| now.duration_since(t).ok())
                    .map(|elapsed| elapsed <= active_threshold)
                    .unwrap_or(false);

                (handshake_epoch, connected, live.stats.rx_bytes, live.stats.tx_bytes)
            } else {
                (None, false, 0, 0)
            };

            result.push(PeerDetail {
                name: peer.name.to_string(),
                ip: peer.ip,
                is_admin: peer.is_admin,
                endpoint: peer.endpoint.as_ref().map(|e| e.to_string()),
                last_handshake_time,
                rx_bytes,
                tx_bytes,
                is_connected,
                is_self,
            });
        }

        result.sort_by(|a, b| b.is_self.cmp(&a.is_self).then_with(|| a.name.cmp(&b.name)));
        Ok(result)
    }

    /// Redeem and install a new invitation file from TOML string.
    pub async fn install_invite(
        &self,
        invite_toml: &str,
        listen_port: Option<u16>,
    ) -> Result<String> {
        let mut config: InterfaceConfig = toml::from_str(invite_toml)
            .map_err(|e| anyhow!("Failed to parse invitation TOML: {}", e))?;

        if let Some(port) = listen_port {
            config.interface.listen_port = Some(port);
        }

        let iface: InterfaceName = config
            .interface
            .network_name
            .parse()
            .map_err(|e| anyhow!("Invalid network name in invitation: {}", e))?;

        let network_opts = self.build_network_opts();

        redeem_invite(&self.config_dir, &network_opts, &iface, config)
            .map_err(|e| anyhow!("Failed to redeem invitation: {}", e))?;

        // Perform initial fetch
        let _ = fetch(
            &self.config_dir,
            &self.data_dir,
            &network_opts,
            &self.hosts_opts,
            &self.nat_opts,
            &iface,
            true,
        );

        let mut active = self.active_interfaces.write().await;
        active.insert(iface.to_string());

        Ok(iface.to_string())
    }

    /// Spawn background loop that periodically refreshes active interfaces.
    pub fn spawn_refresher(self: Arc<Self>, interval: Duration) {
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;

                let interfaces: Vec<String> = {
                    let active = self.active_interfaces.read().await;
                    active.iter().cloned().collect()
                };

                for iface_str in interfaces {
                    if let Ok(iface_name) = iface_str.parse::<InterfaceName>() {
                        if !interface_is_up(self.backend, &iface_name) {
                            let mut active = self.active_interfaces.write().await;
                            active.remove(&iface_str);
                            continue;
                        }

                        let network_opts = self.build_network_opts();

                        if let Err(e) = fetch(
                            &self.config_dir,
                            &self.data_dir,
                            &network_opts,
                            &self.hosts_opts,
                            &self.nat_opts,
                            &iface_name,
                            false,
                        ) {
                            log::warn!("Background fetch failed for {}: {}", iface_str, e);
                        }
                    }
                }
            }
        });
    }
}
