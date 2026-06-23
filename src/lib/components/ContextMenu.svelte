<script lang="ts">
  import type { FileExplorerState } from "$lib/state/FileExplorerState.svelte";
  import type { FileSystemEntry } from "$lib/types";
  import { Copy, Scissors, Clipboard, FolderOpen, FolderPlus, Pencil, RotateCcw, Trash2, Tag, CircleDot, Check, ChevronRight, FileArchive } from "@lucide/svelte";

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
  const isOpenable = $derived(!isTrash && (entry.isDir || entry.isFile));
  const isZip = $derived(
    !isTrash && entry.isFile && entry.name.toLowerCase().endsWith(".zip"),
  );
  const hasClipboard = $derived(!fileExplorer.clipboard.isEmpty);
  const selectedEntries = $derived(fileExplorer.selectedEntries);
  const allTags = $derived(fileExplorer.tags.allTags);
  const allStatuses = $derived(fileExplorer.statuses.allStatuses);

  let tagsExpanded = $state(false);
  let statusExpanded = $state(false);

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

  async function handleNewFolder() {
    onClose();
    await fileExplorer.createFolder();
  }

  function handleRename() {
    fileExplorer.startRename(entry);
    onClose();
  }

  function handleExtract() {
    onClose();
    fileExplorer.startExtract(entry.path);
  }

  async function handleMoveToTrash() {
    await fileExplorer.trash.moveToTrash(
      fileExplorer.selectedEntries.map((e) => e.path),
    );
    await fileExplorer.updateEntries(fileExplorer.currentDir);
    fileExplorer.selection.clearSelection();
    onClose();
  }

  async function handleToggleTag(tagId: number) {
    if (entry.tagIds.includes(tagId)) {
      await fileExplorer.unassignTagFromEntry(entry.path, tagId);
    } else {
      await fileExplorer.assignTagToSelected(entry.path, tagId);
    }
  }

  async function handleToggleStatus(statusId: number) {
    if (entry.statusIds.includes(statusId)) {
      await fileExplorer.unassignStatusFromEntry(entry.path, statusId);
    } else {
      await fileExplorer.assignStatusToSelected(entry.path, statusId);
    }
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

    {#if isZip}
      <button
        class="hover:bg-muted flex w-full cursor-pointer items-center gap-2.5 rounded-lg px-2.5 py-1.5 text-sm"
        onclick={handleExtract}
        role="menuitem"
      >
        <FileArchive class="text-muted-foreground size-4 shrink-0" />
        Extract Here
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

    <button
      class="hover:bg-muted flex w-full cursor-pointer items-center gap-2.5 rounded-lg px-2.5 py-1.5 text-sm"
      onclick={handleNewFolder}
      role="menuitem"
    >
      <FolderPlus class="text-muted-foreground size-4 shrink-0" />
      <span class="flex-1 text-left">New Folder</span>
      <span class="text-muted-foreground text-xs">Ctrl+Shift+N</span>
    </button>

    <div class="bg-border/60 my-1 h-px" role="separator"></div>

    <button
      class="hover:bg-muted flex w-full cursor-pointer items-center gap-2.5 rounded-lg px-2.5 py-1.5 text-sm"
      onclick={handleRename}
      role="menuitem"
    >
      <Pencil class="text-muted-foreground size-4 shrink-0" />
      <span class="flex-1 text-left">Rename</span>
      <span class="text-muted-foreground text-xs">F2</span>
    </button>

    <div class="bg-border/60 my-1 h-px" role="separator"></div>

    <!-- Tags section -->
    <button
      class="hover:bg-muted flex w-full cursor-pointer items-center gap-2.5 rounded-lg px-2.5 py-1.5 text-sm"
      onclick={() => (tagsExpanded = !tagsExpanded)}
      role="menuitem"
    >
      <Tag class="text-muted-foreground size-4 shrink-0" />
      <span class="flex-1 text-left">Tags</span>
      <ChevronRight
        class="text-muted-foreground size-3.5 transition-transform {tagsExpanded ? 'rotate-90' : ''}"
      />
    </button>
    {#if tagsExpanded}
      {#each allTags as tag (tag.id)}
        <button
          class="hover:bg-muted flex w-full cursor-pointer items-center gap-2 rounded-lg py-1.5 pr-2.5 pl-8 text-sm"
          onclick={() => handleToggleTag(tag.id)}
          role="menuitem"
        >
          {#if entry.tagIds.includes(tag.id)}
            <Check class="text-accent size-3.5 shrink-0" />
          {:else}
            <span class="size-3.5 shrink-0"></span>
          {/if}
          {tag.name}
        </button>
      {:else}
        <p class="text-muted-foreground px-8 py-1.5 text-xs">No tags yet</p>
      {/each}
    {/if}

    <!-- Status section -->
    <button
      class="hover:bg-muted flex w-full cursor-pointer items-center gap-2.5 rounded-lg px-2.5 py-1.5 text-sm"
      onclick={() => (statusExpanded = !statusExpanded)}
      role="menuitem"
    >
      <CircleDot class="text-muted-foreground size-4 shrink-0" />
      <span class="flex-1 text-left">Status</span>
      <ChevronRight
        class="text-muted-foreground size-3.5 transition-transform {statusExpanded ? 'rotate-90' : ''}"
      />
    </button>
    {#if statusExpanded}
      {#each allStatuses as status (status.id)}
        <button
          class="hover:bg-muted flex w-full cursor-pointer items-center gap-2 rounded-lg py-1.5 pr-2.5 pl-8 text-sm"
          onclick={() => handleToggleStatus(status.id)}
          role="menuitem"
        >
          {#if entry.statusIds.includes(status.id)}
            <Check class="text-accent size-3.5 shrink-0" />
          {:else}
            <span class="size-3.5 shrink-0"></span>
          {/if}
          {status.name}
        </button>
      {:else}
        <p class="text-muted-foreground px-8 py-1.5 text-xs">No statuses yet</p>
      {/each}
    {/if}

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
