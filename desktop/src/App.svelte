<script lang="ts">
  import { onMount } from "svelte";
  import {
    Shield,
    Wifi,
    WifiOff,
    Copy,
    Check,
    Search,
    Plus,
    RefreshCw,
    X,
    ChevronDown,
    ArrowDown,
    ArrowUp,
  } from "@lucide/svelte";
  import type { NetworkSummary, InterfaceStatus, PeerDetail } from "./types";

  // Mock / Tauri invoke bridge
  const isTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

  async function invokeCommand<T>(cmd: string, args: Record<string, unknown> = {}): Promise<T> {
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke<T>(cmd, args);
    } else {
      // Mock data for development and web preview
      if (cmd === "get_networks") {
        return [
          { interface: "corp-mesh", is_up: true, address: "10.60.64.12", listen_port: 51820 },
          { interface: "homelab", is_up: false, address: "10.80.0.5", listen_port: 51821 },
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
        return [
          {
            name: "gateway-tokyo",
            ip: "10.60.0.1",
            is_admin: true,
            endpoint: "203.0.113.10:51820",
            last_handshake_time: Math.floor(Date.now() / 1000) - 25,
            rx_bytes: 15420000,
            tx_bytes: 8420000,
            is_connected: true,
          },
          {
            name: "build-agent-01",
            ip: "10.60.64.20",
            is_admin: false,
            endpoint: "198.51.100.42:51820",
            last_handshake_time: Math.floor(Date.now() / 1000) - 65,
            rx_bytes: 23140000,
            tx_bytes: 3120000,
            is_connected: true,
          },
          {
            name: "storage-nas",
            ip: "10.60.64.88",
            is_admin: false,
            endpoint: "198.51.100.88:51820",
            last_handshake_time: Math.floor(Date.now() / 1000) - 110,
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
  }

  // Reactive state using Svelte 5 runes
  let networks = $state<NetworkSummary[]>([]);
  let selectedInterface = $state<string>("");
  let status = $state<InterfaceStatus | null>(null);
  let peers = $state<PeerDetail[]>([]);
  let searchQuery = $state("");
  let filterType = $state<"all" | "active" | "admin">("all");
  let isToggling = $state(false);
  let isRefreshing = $state(false);
  let copiedIp = $state<string | null>(null);

  // Modal state
  let showImportModal = $state(false);
  let inviteTomlInput = $state("");
  let importError = $state<string | null>(null);
  let isImporting = $state(false);

  // Derived filtered peer list
  let filteredPeers = $derived(
    peers.filter((peer) => {
      const matchesSearch =
        peer.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        peer.ip.includes(searchQuery);

      if (!matchesSearch) return false;
      if (filterType === "active") return peer.is_connected;
      if (filterType === "admin") return peer.is_admin;
      return true;
    })
  );

  async function loadNetworks() {
    try {
      const list = await invokeCommand<NetworkSummary[]>("get_networks");
      networks = list;
      if (!selectedInterface && list.length > 0) {
        selectedInterface = list[0].interface;
      }
      if (selectedInterface) {
        await refreshCurrentNetwork();
      }
    } catch (e) {
      console.error("Failed to load networks:", e);
    }
  }

  async function refreshCurrentNetwork() {
    if (!selectedInterface) return;
    isRefreshing = true;
    try {
      const [st, pList] = await Promise.all([
        invokeCommand<InterfaceStatus>("get_status", { interface: selectedInterface }),
        invokeCommand<PeerDetail[]>("get_peers", { interface: selectedInterface }),
      ]);
      status = st;
      peers = pList;
    } catch (e) {
      console.error("Failed to refresh status:", e);
    } finally {
      isRefreshing = false;
    }
  }

  async function toggleConnection() {
    if (!status || isToggling) return;
    isToggling = true;
    const targetState = !status.is_up;
    try {
      await invokeCommand("toggle_connect", {
        interface: selectedInterface,
        enable: targetState,
      });
      await refreshCurrentNetwork();
    } catch (e) {
      console.error("Failed to toggle connection:", e);
    } finally {
      isToggling = false;
    }
  }

  async function copyToClipboard(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      copiedIp = text;
      setTimeout(() => {
        copiedIp = null;
      }, 2000);
    } catch (e) {
      console.error("Clipboard copy failed:", e);
    }
  }

  async function handleImportInvite() {
    if (!inviteTomlInput.trim()) return;
    isImporting = true;
    importError = null;
    try {
      const res = await invokeCommand<{ interface: string }>("install_invite", {
        inviteToml: inviteTomlInput,
      });
      showImportModal = false;
      inviteTomlInput = "";
      await loadNetworks();
      selectedInterface = res.interface;
      await refreshCurrentNetwork();
    } catch (e: any) {
      importError = e?.toString() || "Failed to import invitation.";
    } finally {
      isImporting = false;
    }
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
  }

  function formatTimeAgo(epochSeconds?: number): string {
    if (!epochSeconds) return "Never";
    const now = Math.floor(Date.now() / 1000);
    const diff = now - epochSeconds;
    if (diff < 30) return "Just now";
    if (diff < 60) return `${diff}s ago`;
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    return `${Math.floor(diff / 86400)}d ago`;
  }

  onMount(() => {
    loadNetworks();
    const interval = setInterval(refreshCurrentNetwork, 10000);
    return () => clearInterval(interval);
  });
</script>

<main class="w-[380px] h-[580px] flex flex-col bg-slate-900 text-slate-100 border border-slate-800 rounded-lg shadow-2xl overflow-hidden font-sans">
  <!-- Top Bar -->
  <header class="p-3 bg-slate-950/80 backdrop-blur border-b border-slate-800/80 flex items-center justify-between">
    <div class="flex items-center gap-2">
      <div class="p-1.5 rounded-md bg-blue-600/20 text-blue-400 border border-blue-500/30">
        <Shield class="w-4 h-4" />
      </div>
      <div>
        <h1 class="text-sm font-semibold tracking-tight text-white leading-tight">innernet</h1>
        <p class="text-[10px] text-slate-400">Mesh VPN</p>
      </div>
    </div>

    <!-- Network Switcher or Add Button -->
    <div class="flex items-center gap-1.5">
      {#if networks.length > 1}
        <div class="relative">
          <select
            bind:value={selectedInterface}
            onchange={refreshCurrentNetwork}
            class="appearance-none bg-slate-800 hover:bg-slate-700/80 text-xs text-slate-200 px-2 py-1 pr-6 rounded border border-slate-700 cursor-pointer focus:outline-none"
          >
            {#each networks as net}
              <option value={net.interface}>{net.interface}</option>
            {/each}
          </select>
          <ChevronDown class="w-3 h-3 text-slate-400 absolute right-1.5 top-2 pointer-events-none" />
        </div>
      {/if}

      <button
        onclick={() => (showImportModal = true)}
        title="Add Network Invitation"
        class="p-1.5 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white border border-slate-700 transition"
      >
        <Plus class="w-3.5 h-3.5" />
      </button>

      <button
        onclick={refreshCurrentNetwork}
        title="Refresh peers"
        class="p-1.5 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white border border-slate-700 transition"
      >
        <RefreshCw class="w-3.5 h-3.5 {isRefreshing ? 'animate-spin text-blue-400' : ''}" />
      </button>
    </div>
  </header>

  <!-- Connection Card (Tailscale style) -->
  <section class="p-3.5 bg-slate-950/40 border-b border-slate-800 flex flex-col gap-3">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2.5">
        <div class="relative">
          <div
            class="w-3 h-3 rounded-full {status?.is_up
              ? 'bg-emerald-500 ring-4 ring-emerald-500/20'
              : 'bg-slate-600'}"
          ></div>
        </div>
        <div>
          <span class="text-sm font-medium {status?.is_up ? 'text-white' : 'text-slate-400'}">
            {status?.is_up ? "Connected" : "Disconnected"}
          </span>
          {#if selectedInterface}
            <span class="text-xs text-slate-400 block font-mono">{selectedInterface}</span>
          {/if}
        </div>
      </div>

      <!-- Animated Toggle Switch -->
      <button
        onclick={toggleConnection}
        disabled={isToggling}
        aria-label="Toggle network connection"
        class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none {status?.is_up
          ? 'bg-blue-600'
          : 'bg-slate-700'} {isToggling ? 'opacity-50 cursor-wait' : 'cursor-pointer'}"
      >
        <span
          class="inline-block h-4 w-4 transform rounded-full bg-white shadow-md transition-transform {status?.is_up
            ? 'translate-x-6'
            : 'translate-x-1'}"
        ></span>
      </button>
    </div>

    <!-- Machine IP & Stats Row -->
    {#if status?.is_up && status?.address}
      <div class="flex items-center justify-between bg-slate-800/60 rounded-md p-2 px-2.5 border border-slate-700/60">
        <div class="flex items-center gap-1.5">
          <span class="text-[11px] text-slate-400 font-medium">This Machine:</span>
          <span class="text-xs font-mono font-medium text-blue-300">{status.address}</span>
        </div>
        <button
          onclick={() => copyToClipboard(status?.address || "")}
          class="flex items-center gap-1 text-[11px] text-slate-400 hover:text-white transition px-1.5 py-0.5 rounded hover:bg-slate-700"
        >
          {#if copiedIp === status.address}
            <Check class="w-3 h-3 text-emerald-400" />
            <span class="text-emerald-400">Copied</span>
          {:else}
            <Copy class="w-3 h-3" />
            <span>Copy</span>
          {/if}
        </button>
      </div>

      <!-- Traffic Stats Pill -->
      <div class="flex items-center justify-between text-[11px] text-slate-400 px-1">
        <div class="flex items-center gap-3">
          <span class="flex items-center gap-1">
            <ArrowDown class="w-3 h-3 text-emerald-400" />
            {formatBytes(status.total_rx_bytes)}
          </span>
          <span class="flex items-center gap-1">
            <ArrowUp class="w-3 h-3 text-blue-400" />
            {formatBytes(status.total_tx_bytes)}
          </span>
        </div>
        <span class="text-slate-400 font-medium">
          {status.active_peer_count} / {status.peer_count} active
        </span>
      </div>
    {/if}
  </section>

  <!-- Peers Section -->
  <section class="flex-1 flex flex-col min-h-0 bg-slate-900">
    <!-- Filter bar -->
    <div class="p-2.5 border-b border-slate-800 flex items-center gap-2">
      <div class="relative flex-1">
        <Search class="w-3.5 h-3.5 text-slate-500 absolute left-2.5 top-2.5 pointer-events-none" />
        <input
          type="text"
          placeholder="Filter peers..."
          bind:value={searchQuery}
          class="w-full bg-slate-800 text-xs rounded pl-8 pr-2 py-1.5 text-slate-200 placeholder-slate-500 border border-slate-700 focus:outline-none focus:border-blue-500 transition"
        />
      </div>

      <div class="flex rounded bg-slate-800 p-0.5 border border-slate-700 text-[11px]">
        <button
          onclick={() => (filterType = "all")}
          class="px-2 py-1 rounded transition {filterType === 'all'
            ? 'bg-slate-700 text-white font-medium'
            : 'text-slate-400 hover:text-slate-200'}"
        >
          All
        </button>
        <button
          onclick={() => (filterType = "active")}
          class="px-2 py-1 rounded transition {filterType === 'active'
            ? 'bg-slate-700 text-white font-medium'
            : 'text-slate-400 hover:text-slate-200'}"
        >
          Online
        </button>
        <button
          onclick={() => (filterType = "admin")}
          class="px-2 py-1 rounded transition {filterType === 'admin'
            ? 'bg-slate-700 text-white font-medium'
            : 'text-slate-400 hover:text-slate-200'}"
        >
          Admin
        </button>
      </div>
    </div>

    <!-- Peer Scroll List -->
    <div class="flex-1 overflow-y-auto divide-y divide-slate-800/60">
      {#if filteredPeers.length === 0}
        <div class="p-8 text-center text-slate-500 text-xs">
          {#if searchQuery}
            No peers matching "{searchQuery}"
          {:else if !status?.is_up}
            Connect to view network peers
          {:else}
            No peers discovered yet
          {/if}
        </div>
      {:else}
        {#each filteredPeers as peer}
          <div class="p-2.5 px-3 hover:bg-slate-800/40 transition flex items-center justify-between group">
            <div class="flex items-center gap-2.5 min-w-0">
              <div
                class="w-2 h-2 rounded-full shrink-0 {peer.is_connected
                  ? 'bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.5)]'
                  : 'bg-slate-600'}"
                title={peer.is_connected ? "Online" : "Offline"}
              ></div>

              <div class="min-w-0">
                <div class="flex items-center gap-1.5">
                  <span class="text-xs font-medium text-slate-200 truncate">{peer.name}</span>
                  {#if peer.is_admin}
                    <span class="px-1 py-0.2 text-[9px] rounded font-medium bg-amber-500/20 text-amber-300 border border-amber-500/30">
                      admin
                    </span>
                  {/if}
                </div>
                <div class="flex items-center gap-2 text-[11px] text-slate-400 font-mono">
                  <span>{peer.ip}</span>
                  <span class="text-slate-600">•</span>
                  <span class="text-[10px] text-slate-400">{formatTimeAgo(peer.last_handshake_time)}</span>
                </div>
              </div>
            </div>

            <!-- Copy button on hover -->
            <button
              onclick={() => copyToClipboard(peer.ip)}
              title="Copy peer IP"
              class="opacity-0 group-hover:opacity-100 p-1.5 rounded hover:bg-slate-700 text-slate-400 hover:text-white transition shrink-0"
            >
              {#if copiedIp === peer.ip}
                <Check class="w-3.5 h-3.5 text-emerald-400" />
              {:else}
                <Copy class="w-3.5 h-3.5" />
              {/if}
            </button>
          </div>
        {/each}
      {/if}
    </div>
  </section>

  <!-- Import Modal Dialog -->
  {#if showImportModal}
    <div class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50">
      <div class="bg-slate-900 border border-slate-700 rounded-lg p-4 w-full max-w-sm flex flex-col gap-3 shadow-xl">
        <div class="flex items-center justify-between border-b border-slate-800 pb-2">
          <h2 class="text-xs font-semibold uppercase tracking-wider text-slate-300">
            Import Network Invitation
          </h2>
          <button
            onclick={() => (showImportModal = false)}
            class="text-slate-400 hover:text-white"
          >
            <X class="w-4 h-4" />
          </button>
        </div>

        <p class="text-xs text-slate-400">
          Paste the contents of your <code class="text-slate-300 bg-slate-800 px-1 py-0.5 rounded">.toml</code> invitation file below:
        </p>

        <textarea
          bind:value={inviteTomlInput}
          rows="6"
          placeholder="[interface]&#10;network-name = &quot;my-mesh&quot;&#10;address = &quot;10.60.64.10/16&quot;&#10;..."
          class="w-full bg-slate-950 border border-slate-800 rounded p-2 text-xs font-mono text-slate-200 placeholder-slate-600 focus:outline-none focus:border-blue-500"
        ></textarea>

        {#if importError}
          <div class="text-[11px] text-red-400 bg-red-950/40 border border-red-900/60 p-2 rounded">
            {importError}
          </div>
        {/if}

        <div class="flex items-center justify-end gap-2 pt-1">
          <button
            onclick={() => (showImportModal = false)}
            class="px-3 py-1.5 rounded text-xs text-slate-300 hover:bg-slate-800 transition"
          >
            Cancel
          </button>
          <button
            onclick={handleImportInvite}
            disabled={isImporting || !inviteTomlInput.trim()}
            class="px-3 py-1.5 rounded text-xs bg-blue-600 hover:bg-blue-500 text-white font-medium transition disabled:opacity-50"
          >
            {isImporting ? "Importing..." : "Redeem & Install"}
          </button>
        </div>
      </div>
    </div>
  {/if}
</main>
