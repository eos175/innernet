<script lang="ts">
  import { X, FolderOpen } from "@lucide/svelte";
  import { pickInviteFile } from "../services/api";

  interface Props {
    isOpen: boolean;
    onClose: () => void;
    onImport: (inviteToml: string) => Promise<void>;
  }

  let { isOpen, onClose, onImport }: Props = $props();

  let inviteTomlInput = $state("");
  let importError = $state<string | null>(null);
  let isImporting = $state(false);
  let fileInputRef = $state<HTMLInputElement | null>(null);

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

  async function handlePickFile() {
    importError = null;
    const content = await pickInviteFile();
    if (content) {
      inviteTomlInput = content;
    } else if (fileInputRef) {
      fileInputRef.click();
    }
  }

  function handleHtmlFileInput(e: Event) {
    const target = e.target as HTMLInputElement;
    const file = target.files?.[0];
    if (file) {
      const reader = new FileReader();
      reader.onload = (event) => {
        inviteTomlInput = event.target?.result as string || "";
      };
      reader.readAsText(file);
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

      <div class="flex items-center justify-between">
        <p class="text-xs text-slate-400">
          Select or paste your <code class="text-slate-300 bg-slate-800 px-1 py-0.5 rounded">.toml</code> invitation:
        </p>

        <button
          onclick={handlePickFile}
          class="flex items-center gap-1.5 px-2 py-1 rounded bg-slate-800 hover:bg-slate-700 text-[11px] text-blue-400 border border-slate-700 transition"
        >
          <FolderOpen class="w-3.5 h-3.5" />
          <span>Select File</span>
        </button>

        <input
          type="file"
          accept=".toml"
          bind:this={fileInputRef}
          onchange={handleHtmlFileInput}
          class="hidden"
        />
      </div>

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
