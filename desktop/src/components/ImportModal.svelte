<script lang="ts">
  import { X } from "@lucide/svelte";

  interface Props {
    isOpen: boolean;
    onClose: () => void;
    onImport: (inviteToml: string) => Promise<void>;
  }

  let { isOpen, onClose, onImport }: Props = $props();

  let inviteTomlInput = $state("");
  let importError = $state<string | null>(null);
  let isImporting = $state(false);

  async function handleImport() {
    if (!inviteTomlInput.trim() || isImporting) return;
    isImporting = true;
    importError = null;
    try {
      await onImport(inviteTomlInput);
      inviteTomlInput = "";
      onClose();
    } catch (e: any) {
      importError = e?.toString() || "Failed to import invitation.";
    } finally {
      isImporting = false;
    }
  }

  function handleClose() {
    inviteTomlInput = "";
    importError = null;
    onClose();
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50">
    <div class="bg-slate-900 border border-slate-700 rounded-lg p-4 w-full max-w-sm flex flex-col gap-3 shadow-xl">
      <div class="flex items-center justify-between border-b border-slate-800 pb-2">
        <h2 class="text-xs font-semibold uppercase tracking-wider text-slate-300">
          Import Network Invitation
        </h2>
        <button
          onclick={handleClose}
          class="text-slate-400 hover:text-white"
          title="Close dialog"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <p class="text-xs text-slate-400">
        Paste the contents of your <code class="text-slate-300 bg-slate-800 px-1 py-0.5 rounded">.toml</code> invitation file below:
      </p>

      <textarea
        bind:value={inviteTomlInput}
        rows="6"
        placeholder="[interface]&#10;network-name = &quot;my-mesh&quot;&#10;address = &quot;10.60.64.10/16&quot;&#10;..."
        class="w-full bg-slate-950 border border-slate-800 rounded p-2 text-xs font-mono text-slate-200 placeholder-slate-600 focus:outline-none focus:border-blue-500"
      ></textarea>

      {#if importError}
        <div class="text-[11px] text-red-400 bg-red-950/40 border border-red-900/60 p-2 rounded">
          {importError}
        </div>
      {/if}

      <div class="flex items-center justify-end gap-2 pt-1">
        <button
          onclick={handleClose}
          class="px-3 py-1.5 rounded text-xs text-slate-300 hover:bg-slate-800 transition"
        >
          Cancel
        </button>
        <button
          onclick={handleImport}
          disabled={isImporting || !inviteTomlInput.trim()}
          class="px-3 py-1.5 rounded text-xs bg-blue-600 hover:bg-blue-500 text-white font-medium transition disabled:opacity-50"
        >
          {isImporting ? "Importing..." : "Redeem & Install"}
        </button>
      </div>
    </div>
  </div>
{/if}
