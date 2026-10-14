use serde::{Deserialize, Serialize};
use std::net::IpAddr;

/// Request sent from a client (e.g., Tauri desktop app or CLI) to the daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", content = "payload", rename_all = "snake_case")]
pub enum DaemonRequest {
    /// List all installed innernet networks.
    ListNetworks,
    /// Get the detailed connection status for a specific interface.
    GetStatus { interface: String },
    /// Bring up a WireGuard interface and start background sync.
    Up { interface: String },
    /// Bring down a WireGuard interface and stop background sync.
    Down { interface: String },
    /// Trigger an immediate peer and route fetch from the coordination server.
    Fetch { interface: String },
    /// Get list of peers for an interface.
    GetPeers { interface: String },
    /// Redeem and install a new network invitation from TOML content.
    InstallInvite {
        invite_toml: String,
        listen_port: Option<u16>,
    },
    /// Send an async ICMP echo request to a peer IP and return latency in ms.
    Ping { ip: IpAddr },
}

/// Response returned by the daemon for any request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", content = "data", rename_all = "snake_case")]
pub enum DaemonResponse {
    Ok(serde_json::Value),
    Err(String),
}

/// Summary of an installed network configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSummary {
    pub interface: String,
    pub is_up: bool,
    pub address: Option<IpAddr>,
    pub listen_port: Option<u16>,
}

/// Status details for a single network interface.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceStatus {
    pub interface: String,
    pub is_up: bool,
    pub address: Option<IpAddr>,
    pub listen_port: Option<u16>,
    pub public_key: Option<String>,
    pub server_endpoint: Option<String>,
    pub peer_count: usize,
    pub active_peer_count: usize,
    pub total_rx_bytes: u64,
    pub total_tx_bytes: u64,
}

/// Peer information returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerDetail {
    pub name: String,
    pub ip: IpAddr,
    #[serde(default)]
    pub public_key: Option<String>,
    #[serde(default)]
    pub cidr_name: Option<String>,
    pub is_admin: bool,
    pub endpoint: Option<String>,
    pub last_handshake_time: Option<u64>,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub is_connected: bool,
    #[serde(default)]
    pub is_self: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize_requests() {
        let req = DaemonRequest::GetStatus {
            interface: "corporate".to_string(),
        };
        let json = serde_json::to_string(&req).expect("Failed to serialize request");
        assert!(json.contains("\"action\":\"get_status\""));
        assert!(json.contains("\"interface\":\"corporate\""));

        let deserialized: DaemonRequest =
            serde_json::from_str(&json).expect("Failed to deserialize request");
        match deserialized {
            DaemonRequest::GetStatus { interface } => assert_eq!(interface, "corporate"),
            _ => panic!("Unexpected request variant"),
        }
    }

    #[test]
    fn test_serialize_deserialize_responses() {
        let resp = DaemonResponse::Ok(serde_json::json!({ "connected": true }));
        let json = serde_json::to_string(&resp).expect("Failed to serialize response");
        assert!(json.contains("\"status\":\"ok\""));

        let deserialized: DaemonResponse =
            serde_json::from_str(&json).expect("Failed to deserialize response");
        match deserialized {
            DaemonResponse::Ok(val) => {
                assert_eq!(val.get("connected").unwrap(), true);
            }
            _ => panic!("Unexpected response variant"),
        }
    }

    #[test]
    fn test_list_networks_request() {
        let req = DaemonRequest::ListNetworks;
        let json = serde_json::to_string(&req).expect("Failed to serialize");
        assert_eq!(json, "{\"action\":\"list_networks\"}");

        let deserialized: DaemonRequest =
            serde_json::from_str(&json).expect("Failed to deserialize");
        matches!(deserialized, DaemonRequest::ListNetworks);
    }
}

