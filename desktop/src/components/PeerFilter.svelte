<script lang="ts">
  import { Search } from "@lucide/svelte";

  interface Props {
    searchQuery: string;
    filterType: "all" | "active" | "admin";
  }

  let { searchQuery = $bindable(), filterType = $bindable() }: Props = $props();
</script>

<div class="p-2.5 border-b border-slate-800 flex items-center gap-2 shrink-0 bg-slate-900">
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
