<script lang="ts">
  import { Search, Activity } from "@lucide/svelte";

  interface Props {
    searchQuery: string;
    filterType: "all" | "active" | "admin";
    isPingingAll?: boolean;
    onPingAll?: () => void;
  }

  let { searchQuery = $bindable(), filterType = $bindable(), isPingingAll = false, onPingAll }: Props = $props();
</script>

<div class="p-2.5 border-b border-slate-800 flex items-center gap-2 shrink-0 bg-slate-900">
  <div class="relative flex-1 min-w-0">
    <Search class="w-3.5 h-3.5 text-slate-500 absolute left-2.5 top-2.5 pointer-events-none" />
    <input
      type="text"
      placeholder="Filter peers..."
      bind:value={searchQuery}
      class="w-full bg-slate-800 text-xs rounded pl-8 pr-2 py-1.5 text-slate-200 placeholder-slate-500 border border-slate-700 focus:outline-none focus:border-blue-500 transition"
    />
  </div>

  {#if onPingAll}
    <button
      onclick={onPingAll}
      disabled={isPingingAll}
      title="Diagnose / Ping all peers"
      class="relative p-1.5 rounded bg-slate-800 hover:bg-slate-700 border border-slate-700 text-slate-400 hover:text-white transition shrink-0 disabled:opacity-50 overflow-hidden"
    >
      {#if isPingingAll}
        <span class="animate-ping absolute inset-0 rounded bg-blue-500/30 pointer-events-none"></span>
      {/if}
      <Activity class="w-3.5 h-3.5 {isPingingAll ? 'animate-pulse text-amber-400' : ''}" />
    </button>
  {/if}

  <div class="flex rounded bg-slate-800 p-0.5 border border-slate-700 text-[11px] shrink-0 font-medium">
    <button
      onclick={() => (filterType = "all")}
      class="px-2.5 py-1 rounded transition whitespace-nowrap {filterType === 'all'
        ? 'bg-slate-700 text-white font-medium'
        : 'text-slate-400 hover:text-slate-200'}"
    >
      All
    </button>
    <button
      onclick={() => (filterType = "active")}
      class="px-2.5 py-1 rounded transition whitespace-nowrap {filterType === 'active'
        ? 'bg-slate-700 text-white font-medium'
        : 'text-slate-400 hover:text-slate-200'}"
    >
      Online
    </button>
    <button
      onclick={() => (filterType = "admin")}
      class="px-2.5 py-1 rounded transition whitespace-nowrap {filterType === 'admin'
        ? 'bg-slate-700 text-white font-medium'
        : 'text-slate-400 hover:text-slate-200'}"
    >
      Admin
    </button>
  </div>
</div>
