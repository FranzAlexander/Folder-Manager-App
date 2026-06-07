<script lang="ts">
  import type { FileExplorerState } from "$lib/state/FileExplorerState.svelte";
  import { FolderPlus, RotateCcw, Trash2, Eraser } from "@lucide/svelte";

  let { fileExplorer }: { fileExplorer: FileExplorerState } = $props();

  const isTrash = $derived(fileExplorer.currentDir === "trash://");
  const hasSelection = $derived(fileExplorer.selectedEntries.length > 0);
</script>

{#if !isTrash}
  <div class="border-border flex items-center gap-1 border-b px-3 py-1">
    <button
      class="hover:bg-muted flex cursor-pointer items-center gap-1.5 rounded-md px-2.5 py-1.5 text-sm"
      onclick={() => fileExplorer.createFolder()}
    >
      <FolderPlus class="size-3.5" />
      New Folder
      <span class="text-muted-foreground ml-1 text-xs">Ctrl+Shift+N</span>
    </button>
  </div>
{:else if isTrash}
  <div class="border-border flex items-center gap-1 border-b px-3 py-1">
    <button
      class="hover:bg-muted disabled:text-muted-foreground flex cursor-pointer items-center gap-1.5 rounded-md px-2.5 py-1.5 text-sm disabled:cursor-not-allowed disabled:opacity-40"
      disabled={!hasSelection}
      onclick={() => fileExplorer.restoreSelected()}
    >
      <RotateCcw class="size-3.5" />
      Restore
    </button>

    <button
      class="hover:bg-destructive/10 text-destructive disabled:text-muted-foreground flex cursor-pointer items-center gap-1.5 rounded-md px-2.5 py-1.5 text-sm disabled:cursor-not-allowed disabled:opacity-40"
      disabled={!hasSelection}
      onclick={() => fileExplorer.deleteSelectedPermanently()}
    >
      <Trash2 class="size-3.5" />
      Delete Permanently
    </button>

    <div class="bg-border/60 mx-1 h-4 w-px"></div>

    <button
      class="hover:bg-destructive/10 text-destructive flex cursor-pointer items-center gap-1.5 rounded-md px-2.5 py-1.5 text-sm"
      onclick={() => fileExplorer.trash.deleteAllPermanently()}
    >
      <Eraser class="size-3.5" />
      Empty Trash
    </button>
  </div>
{/if}

