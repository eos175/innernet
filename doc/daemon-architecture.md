# innernetd Architecture & Non-Root IPC Specification

## Overview

Historically, `innernet` required `root` (or `sudo`) for almost every client operation because configuring WireGuard interfaces (`wg set`, creating network devices, modifying routing tables) requires Linux kernel network capabilities (`CAP_NET_ADMIN`).

`innernetd` introduces a decoupled client-server architecture inspired by production Linux tools like `dockerd` (Docker) and `tailscaled` (Tailscale):

1. **`innernetd` (Background Daemon)**:
   - Runs with elevated privileges as a systemd service (`innernetd.service`).
   - Maintains WireGuard network interfaces, manages state synchronization with the `innernet-server`, executes pure-Rust ICMP ping probes, and monitors routing.
   - Binds to a local Unix domain socket at `/var/run/innernet/innernetd.sock` (mode `0666`).
2. **`innernet` (Non-Root CLI)**:
   - Runs in unprivileged user space.
   - Detects `/var/run/innernet/innernetd.sock`. When available, everyday commands (`list`, `up`, `down`, `fetch`) are delegated via IPC, requiring **no `sudo` or root password**.
   - If the daemon is not running and the user is root, it gracefully falls back to legacy direct standalone execution (100% backward compatible).
3. **`innernet-desktop` (GUI Client)**:
   - An unprivileged desktop application (Tauri v2 + Svelte 5) that communicates with `innernetd` over the same Unix socket.

---

## IPC Protocol

The daemon exposes a line-delimited JSON-RPC interface over `/var/run/innernet/innernetd.sock` (or fallback `/tmp/innernetd.sock`).

### Request Format

Each request is a single-line JSON object with an `action` tag and an optional `payload`:

```json
{"action": "list_networks"}
{"action": "get_status", "payload": {"interface": "corporate"}}
{"action": "up", "payload": {"interface": "corporate"}}
{"action": "down", "payload": {"interface": "corporate"}}
{"action": "fetch", "payload": {"interface": "corporate"}}
{"action": "get_peers", "payload": {"interface": "corporate"}}
{"action": "ping", "payload": {"ip": "10.42.0.1"}}
{"action": "install_invite", "payload": {"invite_toml": "...", "listen_port": 51820}}
```

### Response Format

Each response is a single-line JSON object with a `status` field:

```json
{"status": "ok", "data": { ... }}
{"status": "err", "data": "Error message description"}
```

---

## Systemd Service Configuration

The unified package installs `innernetd.service` to `/usr/lib/systemd/system/innernetd.service`:

```ini
[Unit]
Description=innernet background management daemon
After=network-online.target nss-lookup.target
Wants=network-online.target nss-lookup.target
Conflicts=innernet@*.service

[Service]
Type=simple
ExecStart=/usr/bin/innernetd --socket-path /var/run/innernet/innernetd.sock
Restart=always
RestartSec=5
RuntimeDirectory=innernet
RuntimeDirectoryMode=0755

[Install]
WantedBy=multi-user.target
```

### Mutual Conflict Prevention
To avoid dual synchronization loops on the same WireGuard interface:
- `innernetd.service` declares `Conflicts=innernet@*.service`.
- `innernet@.service` declares `Conflicts=innernetd.service`.
Starting `innernetd` automatically halts any running per-interface units, and vice versa.

---

## Unified Packaging

The client package (`innernet.rpm` and `innernet.deb`) bundles both binaries and services:
- `/usr/bin/innernet` (CLI frontend)
- `/usr/bin/innernetd` (Background daemon)
- `/usr/lib/systemd/system/innernetd.service`
- `/usr/lib/systemd/system/innernet@.service`
- Shell completions and man pages.

Building packages via Docker remains identical to upstream:
```sh
./rpm/build-package.sh fedora:40
```
This produces both `innernet-*.rpm` (client + daemon) and `innernet-server-*.rpm`.
