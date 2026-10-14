<script lang="ts">
  import { ArrowDown, ArrowUp, Copy, Check } from "@lucide/svelte";
  import type { InterfaceStatus } from "../types";
  import { formatBytes } from "../utils/format";

  interface Props {
    status: InterfaceStatus | null;
    selectedInterface: string;
    isToggling: boolean;
    copiedIp: string | null;
    onToggle: () => void;
    onCopy: (text: string) => void;
  }

  let {
    status,
    selectedInterface,
    isToggling,
    copiedIp,
    onToggle,
    onCopy,
  }: Props = $props();
</script>

<section class="p-3.5 bg-slate-950/40 border-b border-slate-800 flex flex-col gap-3 shrink-0">
  <div class="flex items-center justify-between">
    <div class="flex items-center gap-2.5">
      <div class="relative flex items-center justify-center">
        {#if status?.is_up}
          <span class="animate-ping absolute inline-flex h-3 w-3 rounded-full bg-emerald-400 opacity-75"></span>
        {/if}
        <div
          class="relative w-3 h-3 rounded-full {status?.is_up
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
      onclick={onToggle}
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
        onclick={() => onCopy(status?.address || "")}
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
