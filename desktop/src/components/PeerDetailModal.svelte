<script lang="ts">
  import { X, Check, Copy, Terminal, Activity, ArrowDown, ArrowUp, Key, Globe, Clock, Shield } from "@lucide/svelte";
  import type { PeerDetail } from "../types";
  import { formatBytes, formatTimeAgo } from "../utils/format";
  import { pingPeer } from "../services/api";

  interface Props {
    peer: PeerDetail | null;
    currentAddress?: string;
    onClose: () => void;
  }

  let { peer, currentAddress, onClose }: Props = $props();

  let isSelf = $derived(
    peer?.is_self ||
      Boolean(peer && currentAddress && (peer.ip === currentAddress || currentAddress.startsWith(peer.ip)))
  );

  let copiedField = $state<string | null>(null);
  let isPinging = $state(false);
  let pingLatency = $state<number | null>(null);
  let pingFailed = $state(false);

  function copyText(text: string, field: string) {
    navigator.clipboard.writeText(text);
    copiedField = field;
    setTimeout(() => {
      if (copiedField === field) copiedField = null;
    }, 2000);
  }

  async function handlePing() {
    if (!peer || isPinging) return;
    isPinging = true;
    pingFailed = false;
    try {
      const ms = await pingPeer(peer.ip);
      pingLatency = Math.round(ms);
    } catch {
      pingFailed = true;
      pingLatency = null;
    } finally {
      isPinging = false;
    }
  }
</script>

{#if peer}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-950/80 backdrop-blur-sm animate-fade-in font-sans">
    <div class="w-full max-w-sm bg-slate-900 border border-slate-800 rounded-xl shadow-2xl overflow-hidden flex flex-col">
      <!-- Header -->
      <div class="p-3.5 border-b border-slate-800 flex items-center justify-between bg-slate-900/80">
        <div class="flex items-center gap-2 min-w-0">
          <div
            class="w-2.5 h-2.5 rounded-full shrink-0 {peer.is_connected
              ? 'bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.5)]'
              : 'bg-slate-600'}"
          ></div>
          <h2 class="text-sm font-semibold text-white truncate">{peer.name}</h2>
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
        </div>
        <button
          onclick={onClose}
          class="p-1 rounded text-slate-400 hover:text-slate-200 hover:bg-slate-800 transition"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Content -->
      <div class="p-4 space-y-3.5 text-xs">
        <!-- Subnet / CIDR -->
        {#if peer.cidr_name}
          <div class="flex items-center justify-between py-1 border-b border-slate-800/60">
            <span class="text-slate-400">Subnet</span>
            <span class="font-mono text-slate-200 font-medium">{peer.cidr_name}</span>
          </div>
        {/if}

        <!-- Internal IP -->
        <div class="flex items-center justify-between py-1 border-b border-slate-800/60">
          <span class="text-slate-400">Internal IP</span>
          <div class="flex items-center gap-1.5 font-mono">
            <span class="text-slate-200 font-medium">{peer.ip}</span>
            <button
              onclick={() => copyText(peer.ip, 'ip')}
              class="p-1 rounded hover:bg-slate-800 text-slate-400 hover:text-white transition"
              title="Copy IP"
            >
              {#if copiedField === 'ip'}
                <Check class="w-3.5 h-3.5 text-emerald-400" />
              {:else}
                <Copy class="w-3.5 h-3.5" />
              {/if}
            </button>
          </div>
        </div>

        <!-- Public Key -->
        {#if peer.public_key}
          <div class="flex flex-col gap-1 py-1 border-b border-slate-800/60">
            <div class="flex items-center justify-between">
              <span class="text-slate-400 flex items-center gap-1">
                <Key class="w-3 h-3 text-slate-500" /> Public Key
              </span>
              <button
                onclick={() => copyText(peer.public_key!, 'key')}
                class="p-1 rounded hover:bg-slate-800 text-slate-400 hover:text-white transition"
                title="Copy full public key"
              >
                {#if copiedField === 'key'}
                  <Check class="w-3.5 h-3.5 text-emerald-400" />
                {:else}
                  <Copy class="w-3.5 h-3.5" />
                {/if}
              </button>
            </div>
            <span class="font-mono text-[11px] text-slate-300 break-all bg-slate-950 p-1.5 rounded border border-slate-800/80">
              {peer.public_key}
            </span>
          </div>
        {/if}

        <!-- Public Endpoint -->
        <div class="flex items-center justify-between py-1 border-b border-slate-800/60">
          <span class="text-slate-400 flex items-center gap-1">
            <Globe class="w-3 h-3 text-slate-500" /> Endpoint
          </span>
          <span class="font-mono text-[11px] text-slate-300">
            {peer.endpoint || "Dynamic (NAT Traversal)"}
          </span>
        </div>

        <!-- Last Handshake -->
        <div class="flex items-center justify-between py-1 border-b border-slate-800/60">
          <span class="text-slate-400 flex items-center gap-1">
            <Clock class="w-3 h-3 text-slate-500" /> Last Handshake
          </span>
          <span class="text-slate-300">
            {#if isSelf}
              <span class="text-blue-400 font-medium">This device</span>
            {:else}
              {formatTimeAgo(peer.last_handshake_time)}
            {/if}
          </span>
        </div>

        <!-- Transfer Stats -->
        {#if !isSelf}
          <div class="flex items-center justify-between py-1 border-b border-slate-800/60">
            <span class="text-slate-400">Total Traffic</span>
            <div class="flex items-center gap-2.5 font-mono text-[11px]">
              <span class="text-emerald-400 flex items-center gap-0.5">
                <ArrowDown class="w-3 h-3" /> {formatBytes(peer.rx_bytes)}
              </span>
              <span class="text-blue-400 flex items-center gap-0.5">
                <ArrowUp class="w-3 h-3" /> {formatBytes(peer.tx_bytes)}
              </span>
            </div>
          </div>
        {/if}

        <!-- Quick Actions Row -->
        <div class="pt-2 flex items-center gap-2">
          <button
            onclick={handlePing}
            disabled={isPinging}
            class="flex-1 py-1.5 px-3 rounded bg-slate-800 hover:bg-slate-700 border border-slate-700 text-slate-200 text-xs font-medium flex items-center justify-center gap-1.5 transition disabled:opacity-50"
          >
            <Activity class="w-3.5 h-3.5 {isPinging ? 'animate-pulse text-amber-400' : ''}" />
            {#if isPinging}
              Pinging...
            {:else if pingLatency !== null}
              {pingLatency} ms
            {:else if pingFailed}
              Timeout
            {:else}
              Ping Peer
            {/if}
          </button>

          <button
            onclick={() => copyText(`ssh ${peer.name}@${peer.ip}`, 'ssh')}
            class="flex-1 py-1.5 px-3 rounded bg-slate-800 hover:bg-slate-700 border border-slate-700 text-slate-200 text-xs font-medium flex items-center justify-center gap-1.5 transition"
          >
            {#if copiedField === 'ssh'}
              <Check class="w-3.5 h-3.5 text-emerald-400" />
              Copied SSH
            {:else}
              <Terminal class="w-3.5 h-3.5" />
              Copy SSH
            {/if}
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
