use anyhow::{Context, Result};
use innernet_shared::daemon_protocol::{
    DaemonRequest, DaemonResponse, InterfaceStatus, NetworkSummary, PeerDetail,
};
use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    path::Path,
    time::Duration,
};

pub const DEFAULT_SOCKET_PATH: &str = "/var/run/innernet/innernetd.sock";
pub const FALLBACK_SOCKET_PATH: &str = "/tmp/innernetd.sock";

pub struct DaemonClient {
    stream: UnixStream,
}

impl DaemonClient {
    /// Check if the daemon Unix domain socket is available and responding.
    #[allow(dead_code)]
    pub fn is_available() -> bool {
        Self::connect().is_ok()
    }

    /// Check if current process has effective root privileges.
    pub fn is_root() -> bool {
        unsafe { libc::geteuid() == 0 }
    }

    /// Connect to the running innernet-daemon over Unix domain socket.
    pub fn connect() -> Result<Self> {
        let socket_path = if Path::new(DEFAULT_SOCKET_PATH).exists() {
            DEFAULT_SOCKET_PATH
        } else if Path::new(FALLBACK_SOCKET_PATH).exists() {
            FALLBACK_SOCKET_PATH
        } else {
            anyhow::bail!("innernet daemon socket not found");
        };

        let stream = UnixStream::connect(socket_path)
            .with_context(|| format!("Failed to connect to daemon at {}", socket_path))?;

        let timeout = Duration::from_secs(60);
        let _ = stream.set_read_timeout(Some(timeout));
        let _ = stream.set_write_timeout(Some(timeout));

        Ok(Self { stream })
    }

    /// Send a request to the daemon and wait for a response.
    pub fn request(&mut self, req: &DaemonRequest) -> Result<serde_json::Value> {
        let mut line = serde_json::to_vec(req)?;
        line.push(b'\n');
        self.stream.write_all(&line)?;
        self.stream.flush()?;

        let mut reader = BufReader::new(&mut self.stream);
        let mut response_str = String::new();
        reader.read_line(&mut response_str)?;

        if response_str.is_empty() {
            anyhow::bail!("Daemon connection closed unexpectedly");
        }

        let response: DaemonResponse = serde_json::from_str(&response_str)
            .with_context(|| format!("Invalid response from daemon: {}", response_str.trim()))?;

        match response {
            DaemonResponse::Ok(val) => Ok(val),
            DaemonResponse::Err(err) => anyhow::bail!("{}", err),
        }
    }

    pub fn up(&mut self, interface: &str) -> Result<()> {
        self.request(&DaemonRequest::Up {
            interface: interface.to_string(),
        })?;
        Ok(())
    }

    pub fn down(&mut self, interface: &str) -> Result<()> {
        self.request(&DaemonRequest::Down {
            interface: interface.to_string(),
        })?;
        Ok(())
    }

    pub fn fetch(&mut self, interface: &str) -> Result<()> {
        self.request(&DaemonRequest::Fetch {
            interface: interface.to_string(),
        })?;
        Ok(())
    }

    pub fn list_networks(&mut self) -> Result<Vec<NetworkSummary>> {
        let val = self.request(&DaemonRequest::ListNetworks)?;
        let list: Vec<NetworkSummary> = serde_json::from_value(val)?;
        Ok(list)
    }

    pub fn get_status(&mut self, interface: &str) -> Result<InterfaceStatus> {
        let val = self.request(&DaemonRequest::GetStatus {
            interface: interface.to_string(),
        })?;
        let status: InterfaceStatus = serde_json::from_value(val)?;
        Ok(status)
    }

    pub fn get_peers(&mut self, interface: &str) -> Result<Vec<PeerDetail>> {
        let val = self.request(&DaemonRequest::GetPeers {
            interface: interface.to_string(),
        })?;
        let peers: Vec<PeerDetail> = serde_json::from_value(val)?;
        Ok(peers)
    }
}
