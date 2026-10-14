<script lang="ts">
  import { onMount } from "svelte";
  import type { NetworkSummary, InterfaceStatus, PeerDetail } from "./types";
  import { invokeCommand, hideWindowToTray } from "./services/api";
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
  let copiedIp = $state<string | null>(null);
  let showImportModal = $state(false);

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
    const interval = setInterval(refreshCurrentNetwork, 10000);
    return () => clearInterval(interval);
  });
</script>

<main class="w-[380px] h-[580px] flex flex-col bg-slate-900 text-slate-100 border border-slate-800 rounded-lg shadow-2xl overflow-hidden font-sans">
  <Header
    {networks}
    bind:selectedInterface
    {isRefreshing}
    onSelect={refreshCurrentNetwork}
    onOpenImport={() => (showImportModal = true)}
    onRefresh={refreshCurrentNetwork}
    onHideToTray={hideWindowToTray}
  />

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
      onCopy={handleCopy}
    />
  </section>

  <ImportModal
    isOpen={showImportModal}
    onClose={() => (showImportModal = false)}
    onImport={handleImportInvite}
  />
</main>
