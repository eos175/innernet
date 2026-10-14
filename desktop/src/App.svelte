<script lang="ts">
  import { onMount } from "svelte";
  import type { NetworkSummary, InterfaceStatus, PeerDetail, PeerPingState } from "./types";
  import {
    invokeCommand,
    hideWindowToTray,
    minimizeWindow,
    listenTrayToggleConnect,
    pingPeer,
    updateTrayStatus,
    sendNotification,
  } from "./services/api";
  import Header from "./components/Header.svelte";
  import ConnectionCard from "./components/ConnectionCard.svelte";
  import PeerFilter from "./components/PeerFilter.svelte";
  import PeerList from "./components/PeerList.svelte";
  import ImportModal from "./components/ImportModal.svelte";
  import PeerDetailModal from "./components/PeerDetailModal.svelte";

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
  let isPingingAll = $state(false);
  let pingStates = $state<Record<string, PeerPingState>>({});
  let activeModalPeer = $state<PeerDetail | null>(null);
  let copiedIp = $state<string | null>(null);
  let showImportModal = $state(false);
  let rxSpeed = $state(0);
  let txSpeed = $state(0);
  let prevRx: number | null = null;
  let prevTx: number | null = null;
  let prevTimestamp: number | null = null;
  let prevIsUp: boolean | null = null;
  let prevDaemonConnected: boolean | null = null;

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

  function resetBandwidthCounters() {
    prevRx = null;
    prevTx = null;
    prevTimestamp = null;
    prevIsUp = null;
    rxSpeed = 0;
    txSpeed = 0;
  }

  async function handleSelectInterface() {
    resetBandwidthCounters();
    await refreshCurrentNetwork();
  }

  async function handleManualRefresh() {
    if (!selectedInterface || isRefreshing) return;
    isRefreshing = true;
    try {
      await invokeCommand("fetch_sync", { interface: selectedInterface });
    } catch (e) {
      console.warn("Manual fetch_sync error:", e);
    } finally {
      isRefreshing = false;
      await refreshCurrentNetwork();
    }
  }

  async function refreshCurrentNetwork() {
    if (!selectedInterface || isRefreshing) return;
    isRefreshing = true;
    try {
      const [st, pList] = await Promise.all([
        invokeCommand<InterfaceStatus>("get_status", { interface: selectedInterface }),
        invokeCommand<PeerDetail[]>("get_peers", { interface: selectedInterface }),
      ]);
      status = st;
      peers = pList;

      if (prevDaemonConnected === false) {
        sendNotification("innernet Daemon Restored", "Connection to innernet daemon restored");
      }
      isDaemonConnected = true;
      prevDaemonConnected = true;

      if (st) {
        const now = Date.now();
        if (st.is_up && prevTimestamp !== null && prevRx !== null && prevTx !== null) {
          const deltaSec = (now - prevTimestamp) / 1000;
          if (deltaSec >= 1) {
            const rxDelta = Math.max(0, st.total_rx_bytes - prevRx);
            const txDelta = Math.max(0, st.total_tx_bytes - prevTx);
            rxSpeed = Math.round(rxDelta / deltaSec);
            txSpeed = Math.round(txDelta / deltaSec);
            prevRx = st.total_rx_bytes;
            prevTx = st.total_tx_bytes;
            prevTimestamp = now;
          }
        } else if (st.is_up) {
          prevRx = st.total_rx_bytes;
          prevTx = st.total_tx_bytes;
          prevTimestamp = now;
          rxSpeed = 0;
          txSpeed = 0;
        } else {
          prevRx = null;
          prevTx = null;
          prevTimestamp = null;
          rxSpeed = 0;
          txSpeed = 0;
        }

        // Dynamic tray icon and tooltip
        updateTrayStatus(st.is_up, selectedInterface, st.address);

        // Connection state notifications
        if (prevIsUp !== null && prevIsUp !== st.is_up) {
          if (st.is_up) {
            sendNotification("innernet Connected", `Connected to ${selectedInterface} (${st.address})`);
          } else {
            sendNotification("innernet Disconnected", `Disconnected from ${selectedInterface}`);
          }
        }
        prevIsUp = st.is_up;
      }
    } catch (e) {
      console.error("Failed to refresh status:", e);
      if (prevDaemonConnected === true) {
        sendNotification("innernet Daemon Disconnected", "Lost connection to innernet daemon");
      }
      isDaemonConnected = false;
      prevDaemonConnected = false;
      rxSpeed = 0;
      txSpeed = 0;
      updateTrayStatus(false, selectedInterface);
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

  function isSelfPeer(peer: PeerDetail): boolean {
    return Boolean(
      peer.is_self ||
        (status?.address && (peer.ip === status.address || status.address.startsWith(peer.ip)))
    );
  }

  async function handlePingPeer(ip: string) {
    pingStates[ip] = {
      latency: pingStates[ip]?.latency ?? null,
      isPinging: true,
      failed: false,
    };
    try {
      const ms = await pingPeer(ip);
      pingStates[ip] = { latency: Math.round(ms), isPinging: false, failed: false };
    } catch {
      pingStates[ip] = { latency: null, isPinging: false, failed: true };
    }
  }

  async function handlePingAll() {
    if (isPingingAll || peers.length === 0) return;
    isPingingAll = true;
    try {
      const targetPeers = peers.filter((p) => !isSelfPeer(p));
      await Promise.allSettled(targetPeers.map((p) => handlePingPeer(p.ip)));
    } catch (e) {
      console.error("Failed to ping all:", e);
    } finally {
      isPingingAll = false;
    }
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
    }, 2500);

    return () => {
      clearInterval(interval);
      if (unlistenTray) unlistenTray();
    };
  });
</script>

<main class="w-full h-screen flex flex-col bg-slate-900 text-slate-100 select-none overflow-hidden font-sans border border-slate-800/80">
  <Header
    {networks}
    bind:selectedInterface
    {isRefreshing}
    onSelect={handleSelectInterface}
    onOpenImport={() => (showImportModal = true)}
    onRefresh={handleManualRefresh}
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
    {rxSpeed}
    {txSpeed}
    onToggle={handleToggleConnection}
    onCopy={handleCopy}
  />

  <section class="flex-1 flex flex-col min-h-0 bg-slate-900">
    <PeerFilter
      bind:searchQuery
      bind:filterType
      isPingingAll={isPingingAll}
      onPingAll={handlePingAll}
    />

    <PeerList
      peers={filteredPeers}
      {searchQuery}
      isUp={status?.is_up ?? false}
      {copiedIp}
      currentAddress={status?.address}
      {pingStates}
      onCopy={handleCopy}
      onSelectPeer={(p) => (activeModalPeer = p)}
      onPing={handlePingPeer}
    />
  </section>

  <ImportModal
    isOpen={showImportModal}
    onClose={() => (showImportModal = false)}
    onImport={handleImportInvite}
  />

  <PeerDetailModal
    peer={activeModalPeer}
    currentAddress={status?.address}
    pingState={activeModalPeer ? pingStates[activeModalPeer.ip] : undefined}
    onClose={() => (activeModalPeer = null)}
    onPing={handlePingPeer}
  />
</main>
