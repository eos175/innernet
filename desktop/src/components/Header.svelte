<script lang="ts">
  import { Shield, ChevronDown, Plus, RefreshCw, X } from "@lucide/svelte";
  import type { NetworkSummary } from "../types";
  import { dragWindow } from "../services/api";

  interface Props {
    networks: NetworkSummary[];
    selectedInterface: string;
    isRefreshing: boolean;
    onSelect: (iface: string) => void;
    onOpenImport: () => void;
    onRefresh: () => void;
    onHideToTray?: () => void;
  }

  let {
    networks,
    selectedInterface = $bindable(),
    isRefreshing,
    onSelect,
    onOpenImport,
    onRefresh,
    onHideToTray,
  }: Props = $props();

  function handleSelect(e: Event) {
    const target = e.target as HTMLSelectElement;
    selectedInterface = target.value;
    onSelect(target.value);
  }

  function handleMouseDown(e: MouseEvent) {
    if (e.button === 0) {
      const target = e.target as HTMLElement;
      if (target.closest("button, select, input, a, option")) {
        return;
      }
      dragWindow();
    }
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<header
  onmousedown={handleMouseDown}
  class="p-3 bg-slate-950/80 backdrop-blur border-b border-slate-800/80 flex items-center justify-between shrink-0 select-none cursor-grab active:cursor-grabbing"
>
  <div class="flex items-center gap-2 pointer-events-none">
    <div class="p-1.5 rounded-md bg-blue-600/20 text-blue-400 border border-blue-500/30">
      <Shield class="w-4 h-4" />
    </div>
    <div>
      <h1 class="text-sm font-semibold tracking-tight text-white leading-tight">innernet</h1>
      <p class="text-[10px] text-slate-400">Mesh VPN</p>
    </div>
  </div>

  <div
    class="flex items-center gap-1.5 cursor-default"
    onmousedown={(e) => e.stopPropagation()}
  >
    {#if networks.length > 1}
      <div class="relative">
        <select
          value={selectedInterface}
          onchange={handleSelect}
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
      type="button"
      onclick={onOpenImport}
      title="Add Network Invitation"
      class="p-1.5 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white border border-slate-700 cursor-pointer transition"
    >
      <Plus class="w-3.5 h-3.5" />
    </button>

    <button
      type="button"
      onclick={onRefresh}
      disabled={isRefreshing}
      title={isRefreshing ? "Refreshing peers..." : "Refresh peers"}
      class="p-1.5 rounded bg-slate-800 hover:bg-slate-700 disabled:opacity-50 disabled:cursor-not-allowed text-slate-300 hover:text-white border border-slate-700 cursor-pointer transition"
    >
      <RefreshCw class="w-3.5 h-3.5 {isRefreshing ? 'animate-spin text-blue-400' : ''}" />
    </button>

    <div class="h-3.5 w-px bg-slate-800 mx-0.5"></div>

    {#if onHideToTray}
      <button
        type="button"
        onclick={onHideToTray}
        title="Hide to tray"
        class="p-1.5 rounded bg-slate-800 hover:bg-red-500/20 text-slate-300 hover:text-red-400 border border-slate-700 hover:border-red-500/30 cursor-pointer transition"
      >
        <X class="w-3.5 h-3.5" />
      </button>
    {/if}
  </div>
</header>
