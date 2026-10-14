<script lang="ts">
  import { Check, Copy } from "@lucide/svelte";
  import type { PeerDetail } from "../types";
  import { formatTimeAgo } from "../utils/format";

  interface Props {
    peer: PeerDetail;
    copiedIp: string | null;
    onCopy: (ip: string) => void;
  }

  let { peer, copiedIp, onCopy }: Props = $props();
</script>

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

  <button
    onclick={() => onCopy(peer.ip)}
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
