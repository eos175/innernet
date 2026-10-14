<script lang="ts">
  import { onMount } from "svelte";
  import type { NetworkSummary, InterfaceStatus, PeerDetail } from "./types";
  import { invokeCommand, hideWindowToTray, listenTrayToggleConnect } from "./services/api";
  import Header from "./components/Header.svelte";
  import ConnectionCard from "./components/ConnectionCard.svelte";
  import PeerFilter from "./components/PeerFilter.svelte";
  import PeerList from "./components/PeerList.svelte";
  import ImportModal from "./components/ImportModal.svelte";

  // Reactive application state using Svelte 5 runes
  let networks = $state<NetworkSummary[]>([]);
  let selectedInterface = $state<string>("");
  let status = $state<InterfaceStatus | null>(null);
  let peers = $state<PeerDetail[]>([]);
  let searchQuery = $state("");
  let filterType = $state<"all" | "active" | "admin">("all");
  let isToggling = $state(false);
  let isRefreshing = $state(false);
  let isDaemonConnected = $state(true);
  let copiedIp = $state<string | null>(null);
  let showImportModal = $state(false);

  // Derived filtered peer list
  let filteredPeers = $derived(
    peers.filter((peer) => {
      const isSelf =
        peer.is_self ||
        Boolean(
          status?.address &&
            (peer.ip === status.address || status.address.startsWith(peer.ip))
        );

      const matchesSearch =
        peer.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        peer.ip.includes(searchQuery);

      if (!matchesSearch) return false;
      if (filterType === "active") {
        return isSelf ? (status?.is_up ?? false) : peer.is_connected;
      }
      if (filterType === "admin") return peer.is_admin;
      return true;
    })
  );

  async function loadNetworks() {
    try {
      const list = await invokeCommand<NetworkSummary[]>("get_networks");
      networks = list;
      isDaemonConnected = true;
      if (!selectedInterface && list.length > 0) {
        selectedInterface = list[0].interface;
      }
      if (selectedInterface) {
        await refreshCurrentNetwork();
      }
    } catch (e) {
      console.error("Failed to load networks:", e);
      isDaemonConnected = false;
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
      isDaemonConnected = true;
    } catch (e) {
      console.error("Failed to refresh status:", e);
      isDaemonConnected = false;
    } finally {
      isRefreshing = false;
    }
  }

  async function handleToggleConnection() {
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

  async function handleCopy(text: string) {
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

  async function handleImportInvite(inviteToml: string) {
    const res = await invokeCommand<{ interface: string }>("install_invite", {
      inviteToml,
    });
    await loadNetworks();
    selectedInterface = res.interface;
    await refreshCurrentNetwork();
  }

  onMount(() => {
    loadNetworks();
    let unlistenTray: (() => void) | undefined;
    listenTrayToggleConnect(() => {
      handleToggleConnection();
    }).then((fn) => {
      unlistenTray = fn;
    });

    const interval = setInterval(async () => {
      if (!isDaemonConnected || networks.length === 0) {
        await loadNetworks();
      } else {
        await refreshCurrentNetwork();
      }
    }, 5000);

    return () => {
      clearInterval(interval);
      if (unlistenTray) unlistenTray();
    };
  });
</script>

<main class="w-full h-screen flex flex-col bg-slate-900 text-slate-100 select-none overflow-hidden font-sans">
  <Header
    {networks}
    bind:selectedInterface
    {isRefreshing}
    onSelect={refreshCurrentNetwork}
    onOpenImport={() => (showImportModal = true)}
    onRefresh={refreshCurrentNetwork}
    onHideToTray={hideWindowToTray}
  />

  {#if !isDaemonConnected}
    <div class="bg-amber-500/15 border-b border-amber-500/30 px-3 py-1.5 flex items-center justify-between text-xs text-amber-200 shrink-0">
      <div class="flex items-center gap-2 min-w-0">
        <span class="w-2 h-2 rounded-full bg-amber-400 animate-pulse shrink-0"></span>
        <span class="truncate text-[11px]">innernet-daemon disconnected (waiting for daemon...)</span>
      </div>
      <button
        onclick={() => loadNetworks()}
        class="px-2 py-0.5 rounded bg-amber-500/20 hover:bg-amber-500/30 text-amber-200 text-[11px] font-medium transition shrink-0 ml-2"
      >
        Retry
      </button>
    </div>
  {/if}

  <ConnectionCard
    {status}
    {selectedInterface}
    {isToggling}
    {copiedIp}
    onToggle={handleToggleConnection}
    onCopy={handleCopy}
  />

  <section class="flex-1 flex flex-col min-h-0 bg-slate-900">
    <PeerFilter bind:searchQuery bind:filterType />

    <PeerList
      peers={filteredPeers}
      {searchQuery}
      isUp={status?.is_up ?? false}
      {copiedIp}
      currentAddress={status?.address}
      onCopy={handleCopy}
    />
  </section>

  <ImportModal
    isOpen={showImportModal}
    onClose={() => (showImportModal = false)}
    onImport={handleImportInvite}
  />
</main>
