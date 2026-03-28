<script lang="ts">
  import type { FileExplorerState } from "$lib/state/FileExplorerState.svelte";
  import type { FileSystemEntry } from "$lib/types";
  import { Copy, Scissors, Clipboard, FolderOpen, RotateCcw, Trash2 } from "@lucide/svelte";
  import { confirm } from "@tauri-apps/plugin-dialog";

  let {
    x,
    y,
    entry,
    fileExplorer,
    onClose,
    onPaste,
  }: {
    x: number;
    y: number;
    entry: FileSystemEntry;
    fileExplorer: FileExplorerState;
    onClose: () => void;
    onPaste: () => void;
  } = $props();

  const isTrash = $derived(fileExplorer.currentDir === "trash://");
  const isOpenable = $derived(!isTrash && (entry.isDir || entry.fileType === "EXE"));
  const hasClipboard = $derived(!fileExplorer.clipboard.isEmpty);
  const selectedEntries = $derived(fileExplorer.selectedEntries);

  function clampToViewport(node: HTMLElement) {
    const rect = node.getBoundingClientRect();
    if (rect.right > window.innerWidth) {
      node.style.left = `${window.innerWidth - rect.width - 8}px`;
    }
    if (rect.bottom > window.innerHeight) {
      node.style.top = `${window.innerHeight - rect.height - 8}px`;
    }
  }

  function handleCopy() {
    fileExplorer.clipboard.copy(selectedEntries, fileExplorer.currentDir);
    onClose();
  }

  function handleCut() {
    fileExplorer.clipboard.cut(selectedEntries, fileExplorer.currentDir);
    onClose();
  }

  function handlePaste() {
    onPaste();
    onClose();
  }

  function handleOpen() {
    fileExplorer.openEntry(entry);
    onClose();
  }

  async function handleRestore() {
    await fileExplorer.trash.restore(entry.path);
    onClose();
  }

  async function handleDeletePermanently() {
    await fileExplorer.trash.deletePermanently(entry.path);
    onClose();
  }

  async function handleMoveToTrash() {
    await fileExplorer.trash.moveToTrash(
      fileExplorer.selectedEntries.map((e) => e.path),
    );
    await fileExplorer.updateEntries(fileExplorer.currentDir);
    fileExplorer.selection.clearSelection();
    onClose();
  }
</script>

<svelte:window onclick={onClose} />

<div
  use:clampToViewport
  class="bg-popover border-border text-primary fixed z-50 min-w-48 rounded-xl border px-1 py-1.5 shadow-xl"
  style="left: {x}px; top: {y}px;"
  onclick={(e) => e.stopPropagation()}
  onkeydown={(e) => e.key === "Escape" && onClose()}
  oncontextmenu={(e) => e.preventDefault()}
  role="menu"
  tabindex="0"
>
  {#if isTrash}
    <button
      class="hover:bg-muted flex w-full cursor-pointer items-center gap-2.5 rounded-lg px-2.5 py-1.5 text-sm"
      onclick={handleRestore}
      role="menuitem"
    >
      <RotateCcw class="text-muted-foreground size-4 shrink-0" />
      Restore
    </button>

    <div class="bg-border/60 my-1 h-px" role="separator"></div>

    <button
      class="hover:bg-destructive/10 text-destructive flex w-full cursor-pointer items-center gap-2.5 rounded-lg px-2.5 py-1.5 text-sm"
      onclick={handleDeletePermanently}
      role="menuitem"
    >
      <Trash2 class="size-4 shrink-0" />
      Delete Permanently
    </button>
  {:else}
    {#if isOpenable}
      <button
        class="hover:bg-muted flex w-full cursor-pointer items-center gap-2.5 rounded-lg px-2.5 py-1.5 text-sm"
        onclick={handleOpen}
        role="menuitem"
      >
        <FolderOpen class="text-muted-foreground size-4 shrink-0" />
        Open
      </button>
      <div class="bg-border/60 my-1 h-px" role="separator"></div>
    {/if}

    <button
      class="hover:bg-muted flex w-full cursor-pointer items-center gap-2.5 rounded-lg px-2.5 py-1.5 text-sm"
      onclick={handleCopy}
      role="menuitem"
    >
      <Copy class="text-muted-foreground size-4 shrink-0" />
      <span class="flex-1 text-left">Copy</span>
      <span class="text-muted-foreground text-xs">Ctrl+C</span>
    </button>

    <button
      class="hover:bg-muted flex w-full cursor-pointer items-center gap-2.5 rounded-lg px-2.5 py-1.5 text-sm"
      onclick={handleCut}
      role="menuitem"
    >
      <Scissors class="text-muted-foreground size-4 shrink-0" />
      <span class="flex-1 text-left">Cut</span>
      <span class="text-muted-foreground text-xs">Ctrl+X</span>
    </button>

    <button
      class="hover:bg-muted flex w-full cursor-pointer items-center gap-2.5 rounded-lg px-2.5 py-1.5 text-sm disabled:cursor-not-allowed disabled:opacity-40"
      onclick={handlePaste}
      disabled={!hasClipboard}
      role="menuitem"
    >
      <Clipboard class="text-muted-foreground size-4 shrink-0" />
      <span class="flex-1 text-left">Paste</span>
      <span class="text-muted-foreground text-xs">Ctrl+V</span>
    </button>

    <div class="bg-border/60 my-1 h-px" role="separator"></div>

    <button
      class="hover:bg-destructive/10 text-destructive flex w-full cursor-pointer items-center gap-2.5 rounded-lg px-2.5 py-1.5 text-sm"
      onclick={handleMoveToTrash}
      role="menuitem"
    >
      <Trash2 class="size-4 shrink-0" />
      <span class="flex-1 text-left">Move to Trash</span>
      <span class="text-muted-foreground text-xs">Del</span>
    </button>
  {/if}
</div>
