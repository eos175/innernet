<script lang="ts">
  import type { PeerDetail } from "../types";
  import PeerItem from "./PeerItem.svelte";

  interface Props {
    peers: PeerDetail[];
    searchQuery: string;
    isUp: boolean;
    copiedIp: string | null;
    onCopy: (ip: string) => void;
  }

  let { peers, searchQuery, isUp, copiedIp, onCopy }: Props = $props();
</script>

<div class="flex-1 overflow-y-auto divide-y divide-slate-800/60 min-h-0 bg-slate-900">
  {#if peers.length === 0}
    <div class="p-8 text-center text-slate-500 text-xs">
      {#if searchQuery}
        No peers matching "{searchQuery}"
      {:else if !isUp}
        Connect to view network peers
      {:else}
        No peers discovered yet
      {/if}
    </div>
  {:else}
    {#each peers as peer (peer.ip)}
      <PeerItem {peer} {copiedIp} {onCopy} />
    {/each}
  {/if}
</div>
