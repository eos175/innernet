import type { NetworkSummary, InterfaceStatus, PeerDetail } from "../types";

const isTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

/**
 * Invoke a command on the Tauri backend with mock fallback for browser preview.
 */
export async function invokeCommand<T>(
  cmd: string,
  args: Record<string, unknown> = {}
): Promise<T> {
  if (isTauri) {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<T>(cmd, args);
  }

  // Development mock data when running in a standard browser
  if (cmd === "get_networks") {
    return [
      {
        interface: "corp-mesh",
        is_up: true,
        address: "10.60.64.12",
        listen_port: 51820,
      },
      {
        interface: "homelab",
        is_up: false,
        address: "10.80.0.5",
        listen_port: 51821,
      },
    ] as T;
  }

  if (cmd === "get_status") {
    return {
      interface: "corp-mesh",
      is_up: true,
      address: "10.60.64.12",
      listen_port: 51820,
      public_key: "K7lH...abc=",
      server_endpoint: "vpn.corp.tonari.no:51820",
      peer_count: 5,
      active_peer_count: 3,
      total_rx_bytes: 48592300,
      total_tx_bytes: 12940200,
    } as T;
  }

  if (cmd === "get_peers") {
    const now = Math.floor(Date.now() / 1000);
    return [
      {
        name: "gateway-tokyo",
        ip: "10.60.0.1",
        is_admin: true,
        endpoint: "203.0.113.10:51820",
        last_handshake_time: now - 25,
        rx_bytes: 15420000,
        tx_bytes: 8420000,
        is_connected: true,
      },
      {
        name: "build-agent-01",
        ip: "10.60.64.20",
        is_admin: false,
        endpoint: "198.51.100.42:51820",
        last_handshake_time: now - 65,
        rx_bytes: 23140000,
        tx_bytes: 3120000,
        is_connected: true,
      },
      {
        name: "storage-nas",
        ip: "10.60.64.88",
        is_admin: false,
        endpoint: "198.51.100.88:51820",
        last_handshake_time: now - 110,
        rx_bytes: 9840000,
        tx_bytes: 1250000,
        is_connected: true,
      },
      {
        name: "dev-laptop",
        ip: "10.60.64.33",
        is_admin: false,
        endpoint: undefined,
        last_handshake_time: undefined,
        rx_bytes: 0,
        tx_bytes: 0,
        is_connected: false,
      },
    ] as T;
  }

  if (cmd === "toggle_connect") {
    return { success: true } as T;
  }

  return {} as T;
}
