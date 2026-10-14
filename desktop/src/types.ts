export interface NetworkSummary {
  interface: string;
  is_up: boolean;
  address?: string;
  listen_port?: number;
}

export interface InterfaceStatus {
  interface: string;
  is_up: boolean;
  address?: string;
  listen_port?: number;
  public_key?: string;
  server_endpoint?: string;
  peer_count: number;
  active_peer_count: number;
  total_rx_bytes: number;
  total_tx_bytes: number;
}

export interface PeerDetail {
  name: string;
  ip: string;
  public_key?: string;
  cidr_name?: string;
  is_admin: boolean;
  endpoint?: string;
  last_handshake_time?: number;
  rx_bytes: number;
  tx_bytes: number;
  is_connected: boolean;
  is_self?: boolean;
}

export interface PeerPingState {
  latency: number | null;
  isPinging: boolean;
  failed: boolean;
}
