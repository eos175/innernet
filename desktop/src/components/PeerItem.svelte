<script lang="ts">
  import { Check, Copy, Terminal, Activity, ArrowDown, ArrowUp, Info } from "@lucide/svelte";
  import type { PeerDetail, PeerPingState } from "../types";
  import { formatBytes, formatTimeAgo } from "../utils/format";
  import { pingPeer } from "../services/api";

  interface Props {
    peer: PeerDetail;
    copiedIp: string | null;
    currentAddress?: string;
    pingState?: PeerPingState;
    onCopy: (ip: string) => void;
    onSelectPeer?: (peer: PeerDetail) => void;
    onPing?: (ip: string) => void;
  }

  let { peer, copiedIp, currentAddress, pingState, onCopy, onSelectPeer, onPing }: Props = $props();

  let isSelf = $derived(
    peer.is_self ||
      Boolean(currentAddress && (peer.ip === currentAddress || currentAddress.startsWith(peer.ip)))
  );

  let localIsPinging = $state(false);
  let localPingLatency = $state<number | null>(null);
  let localPingFailed = $state(false);
  let copiedSsh = $state(false);

  let isPinging = $derived(pingState ? pingState.isPinging : localIsPinging);
  let pingLatency = $derived(pingState ? pingState.latency : localPingLatency);
  let pingFailed = $derived(pingState ? pingState.failed : localPingFailed);

  async function handlePing() {
    if (isPinging) return;
    if (onPing) {
      onPing(peer.ip);
      return;
    }
    localIsPinging = true;
    localPingFailed = false;
    try {
      const ms = await pingPeer(peer.ip);
      localPingLatency = Math.round(ms);
    } catch {
      localPingFailed = true;
      localPingLatency = null;
    } finally {
      localIsPinging = false;
    }
  }

  async function handleCopySsh() {
    const cmd = `ssh ${peer.name}@${peer.ip}`;
    try {
      await navigator.clipboard.writeText(cmd);
      copiedSsh = true;
      setTimeout(() => {
        copiedSsh = false;
      }, 2000);
    } catch (e) {
      console.error("Failed to copy SSH:", e);
    }
  }
</script>

<div class="p-2.5 px-3 hover:bg-slate-800/40 transition flex items-center justify-between group">
  <div class="flex items-center gap-2.5 min-w-0">
    <div class="relative flex items-center justify-center w-2 h-2 shrink-0">
      {#if peer.is_connected}
        <span class="animate-ping absolute inline-flex h-2 w-2 rounded-full bg-emerald-400 opacity-75"></span>
      {/if}
      <div
        class="relative w-2 h-2 rounded-full {peer.is_connected
          ? 'bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.5)]'
          : 'bg-slate-600'}"
        title={peer.is_connected ? "Online" : "Offline"}
      ></div>
    </div>

    <div
      class="min-w-0 cursor-pointer"
      onclick={() => onSelectPeer?.(peer)}
      role="button"
      tabindex="0"
      onkeydown={(e) => e.key === 'Enter' && onSelectPeer?.(peer)}
    >
      <div class="flex items-center gap-1.5 flex-wrap">
        <span class="text-xs font-medium text-slate-200 truncate hover:text-blue-300 transition">{peer.name}</span>
        {#if isSelf}
          <span class="px-1.5 py-0.2 text-[9px] rounded font-medium bg-blue-500/20 text-blue-300 border border-blue-500/30">
            you
          </span>
        {/if}
        {#if peer.is_admin}
          <span class="px-1 py-0.2 text-[9px] rounded font-medium bg-amber-500/20 text-amber-300 border border-amber-500/30">
            admin
          </span>
        {/if}

        {#if isPinging}
          <span class="px-1.5 py-0.2 text-[9px] rounded font-mono font-medium bg-amber-500/20 text-amber-300 border border-amber-500/30 animate-pulse flex items-center gap-1">
            <span class="w-1.5 h-1.5 rounded-full bg-amber-400 animate-ping"></span>
            pinging...
          </span>
        {:else if pingLatency !== null && pingLatency !== undefined}
          <span class="px-1 py-0.2 text-[9px] rounded font-mono font-medium {pingLatency < 50 ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/30' : 'bg-amber-500/20 text-amber-300 border border-amber-500/30'}">
            {pingLatency}ms
          </span>
        {:else if pingFailed}
          <span class="px-1 py-0.2 text-[9px] rounded font-mono font-medium bg-rose-500/20 text-rose-300 border border-rose-500/30">
            timeout
          </span>
        {/if}
      </div>

      <div class="flex items-center gap-2 text-[11px] text-slate-400 font-mono mt-0.5">
        <span>{peer.ip}</span>
        <span class="text-slate-600">•</span>
        {#if isSelf}
          <span class="text-[10px] text-blue-400 font-sans font-medium">This device</span>
        {:else}
          <span class="text-[10px] text-slate-400">{formatTimeAgo(peer.last_handshake_time)}</span>
        {/if}

        {#if (peer.rx_bytes > 0 || peer.tx_bytes > 0) && !isSelf}
          <span class="text-slate-600 hidden sm:inline">•</span>
          <span class="text-[10px] text-slate-500 hidden sm:flex items-center gap-1 font-sans">
            <span class="flex items-center"><ArrowDown class="w-2.5 h-2.5 text-slate-400" />{formatBytes(peer.rx_bytes)}</span>
            <span class="flex items-center"><ArrowUp class="w-2.5 h-2.5 text-slate-400" />{formatBytes(peer.tx_bytes)}</span>
          </span>
        {/if}
      </div>
    </div>
  </div>

  <div class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition shrink-0">
    <button
      onclick={() => onSelectPeer?.(peer)}
      title="View peer technical details"
      class="p-1.5 rounded hover:bg-slate-700 text-slate-400 hover:text-white transition"
    >
      <Info class="w-3.5 h-3.5" />
    </button>

    <button
      onclick={handlePing}
      disabled={isPinging}
      title="Ping peer latency"
      class="relative p-1.5 rounded hover:bg-slate-700 text-slate-400 hover:text-white transition disabled:opacity-50"
    >
      {#if isPinging}
        <span class="animate-ping absolute inset-0 rounded bg-amber-400/30 pointer-events-none"></span>
      {/if}
      <Activity class="w-3.5 h-3.5 {isPinging ? 'animate-pulse text-amber-400' : ''}" />
    </button>

    <button
      onclick={handleCopySsh}
      title="Copy SSH command: ssh {peer.name}@{peer.ip}"
      class="p-1.5 rounded hover:bg-slate-700 text-slate-400 hover:text-white transition"
    >
      {#if copiedSsh}
        <Check class="w-3.5 h-3.5 text-emerald-400" />
      {:else}
        <Terminal class="w-3.5 h-3.5" />
      {/if}
    </button>

    <button
      onclick={() => onCopy(peer.ip)}
      title="Copy peer IP"
      class="p-1.5 rounded hover:bg-slate-700 text-slate-400 hover:text-white transition"
    >
      {#if copiedIp === peer.ip}
        <Check class="w-3.5 h-3.5 text-emerald-400" />
      {:else}
        <Copy class="w-3.5 h-3.5" />
      {/if}
    </button>
  </div>
</div>
