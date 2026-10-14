<script lang="ts">
  import type { PeerDetail } from "../types";
  import PeerItem from "./PeerItem.svelte";

  interface Props {
    peers: PeerDetail[];
    searchQuery: string;
    isUp: boolean;
    copiedIp: string | null;
    currentAddress?: string;
    onCopy: (ip: string) => void;
  }

  let { peers, searchQuery, isUp, copiedIp, currentAddress, onCopy }: Props = $props();

  interface CidrGroup {
    name: string;
    peers: PeerDetail[];
  }

  let groupedPeers = $derived.by(() => {
    const groups: CidrGroup[] = [];
    const groupMap = new Map<string, PeerDetail[]>();

    for (const peer of peers) {
      const cidr = peer.cidr_name || "other";
      if (!groupMap.has(cidr)) {
        const list: PeerDetail[] = [];
        groupMap.set(cidr, list);
        groups.push({ name: cidr, peers: list });
      }
      groupMap.get(cidr)!.push(peer);
    }

    return groups;
  });
</script>

<div class="flex-1 overflow-y-auto min-h-0 bg-slate-900">
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
    {#each groupedPeers as group (group.name)}
      <div class="sticky top-0 z-10 bg-slate-950/90 backdrop-blur-sm px-3 py-1 border-y border-slate-800/80 flex items-center justify-between">
        <span class="text-[11px] font-semibold text-slate-300 font-mono tracking-wide">
          {group.name}
        </span>
        <span class="text-[10px] text-slate-500 font-mono">
          {group.peers.length} {group.peers.length === 1 ? 'peer' : 'peers'}
        </span>
      </div>

      <div class="divide-y divide-slate-800/40">
        {#each group.peers as peer (peer.ip)}
          <PeerItem {peer} {copiedIp} {currentAddress} {onCopy} />
        {/each}
      </div>
    {/each}
  {/if}
</div>
