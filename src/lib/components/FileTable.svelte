<script lang="ts">
  import type {
    ColumnKey,
    ConflictingEntry,
    FileSystemEntry,
    Status,
    Tag,
  } from "$lib/types";
  import { Button, ScrollArea, Separator } from "bits-ui";
  import FileIcon from "./FileIcon.svelte";
  import { formatDate, formatFileSize } from "$lib/utils/formatters";
  import type { FileExplorerState } from "$lib/state/FileExplorerState.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import ConflictDialog from "./ConflictDialog.svelte";
  import ContextMenu from "./ContextMenu.svelte";
  import { createVirtualScroll } from "$lib/runes/virtualScroll.svelte";
  import { createKeyboardShortcuts } from "$lib/runes/keyboardShortcuts.svelte";

  let {
    fileExplorer,
    tags,
    statusList,
    onopennewtab,
  }: {
    fileExplorer: FileExplorerState;
    tags: Tag[];
    statusList: Status[];
    onopennewtab?: (path: string) => void;
  } = $props();

  let columns = $state([
    { key: "name" as ColumnKey, label: "Name", width: 300 },
    { key: "dateModified" as ColumnKey, label: "Date Modified", width: 180 },
    { key: "lastOpened" as ColumnKey, label: "Last Opened", width: 180 },
    { key: "fileType" as ColumnKey, label: "Type", width: 120 },
    { key: "size" as ColumnKey, label: "Size", width: 100 },
    { key: "tags" as ColumnKey, label: "Tags", width: 200 },
    { key: "status" as ColumnKey, label: "Status", width: 150 },
  ]);

  const tagMap = $derived(new Map(tags.map((t) => [t.id, t])));
  const statusMap = $derived(new Map(statusList.map((s) => [s.id, s])));

  function focusOnMount(node: HTMLInputElement) {
    node.focus();
    node.select();
  }

  let viewportEl = $state<HTMLElement | null>(null);

  $effect(() => {
    const idx = fileExplorer.pendingScrollToIndex;
    if (idx === null || !viewportEl) return;
    const { itemHeight, containerHeight } = virtualScroll;
    const targetScrollTop = Math.max(0, idx * itemHeight - containerHeight / 2 + itemHeight / 2);
    virtualScroll.scrollTop = targetScrollTop;
    viewportEl.scrollTop = targetScrollTop;
    fileExplorer.pendingScrollToIndex = null;
  });

  let moveAlertOpen = $state(false);
  let conflictEntries = $state<ConflictingEntry[]>([]);

  let contextMenuOpen = $state(false);
  let contextMenuX = $state(0);
  let contextMenuY = $state(0);
  let contextMenuEntry = $state<FileSystemEntry | null>(null);

  let resizingColumn = $state<ColumnKey | null>(null);
  let resizeStartX = $state(0);
  let resizeStartWidth = $state(0);

  async function handlePaste() {
    const conflicts = await fileExplorer.paste();
    if (conflicts && conflicts.length > 0) {
      conflictEntries = conflicts;
      moveAlertOpen = true;
    }
  }

  const keyboardShortcuts = createKeyboardShortcuts(() => fileExplorer, handlePaste);

  function handleContextMenu(e: MouseEvent, entry: FileSystemEntry, index: number) {
    e.preventDefault();
    if (!fileExplorer.selection.isSelected(entry.path)) {
      fileExplorer.selection.selectSingle(entry, index);
    }
    contextMenuX = e.clientX;
    contextMenuY = e.clientY;
    contextMenuEntry = entry;
    contextMenuOpen = true;
  }

  const virtualScroll = createVirtualScroll<FileSystemEntry>({
    items: () => fileExplorer.entries,
    itemHeight: 34,
    containerHeight: 624,
    overscan: 20,
  });

  function startResize(columnKey: ColumnKey, event: MouseEvent) {
    resizingColumn = columnKey;
    resizeStartX = event.clientX;
    const col = columns.find((c) => c.key === columnKey);
    resizeStartWidth = col ? col.width : 0;
    event.preventDefault();
  }

  function handleMouseMove(event: MouseEvent) {
    if (!resizingColumn) return;

    const delta = event.clientX - resizeStartX;
    const newWidth = Math.max(50, resizeStartWidth + delta);

    columns = columns.map((col) =>
      col.key === resizingColumn ? { ...col, width: newWidth } : col,
    );
  }

  function stopResize() {
    resizingColumn = null;
  }

  function getColumnWidth(key: ColumnKey): number {
    return columns.find((c) => c.key === key)?.width || 100;
  }

  function dragStart(e: DragEvent, path: string, isDir: boolean) {
    if (!e.dataTransfer) {
      console.error("NO DATATRANSFER!");
      return;
    }
    e.dataTransfer.clearData();

    const isDraggedItemSelected = fileExplorer.selection.isSelected(path);

    let sourceEntries = isDraggedItemSelected
      ? fileExplorer.entries
          .filter((entry) => fileExplorer.selection.isSelected(entry.path))
          .map((entry) => ({
            path: entry.path,
            isDir: entry.isDir,
          }))
      : [
          {
            path: path,
            isDir: isDir,
          },
        ];

    if (sourceEntries.length === 0) return;

    e.dataTransfer.effectAllowed = "move";
    e.dataTransfer.setData("text/plain", JSON.stringify(sourceEntries));
  }

  function dragOver(e: DragEvent) {
    e.preventDefault();
  }

  async function dragDrop(e: DragEvent, path: string) {
    const data = e.dataTransfer?.getData("text/plain");
    if (!data) return;

    const sourceEntries = JSON.parse(data);

    let conflicts: ConflictingEntry[] = await invoke("prepare_operation", {
      srcEntries: sourceEntries,
      dest: path,
      operationType: "move",
    });

    if (conflicts.length !== 0) {
      conflictEntries = conflicts;
      moveAlertOpen = true;
    } else {
      await invoke("execute_operation", {
        conflictResolutions: {},
      });
      await fileExplorer.updateEntries(fileExplorer.currentDir);
    }
  }

  async function cancelOperation() {
    await invoke("cancel_operation");
    moveAlertOpen = false;
  }
</script>

<svelte:window
  onmousemove={handleMouseMove}
  onmouseup={stopResize}
  onkeydown={keyboardShortcuts.handleKeydown}
/>

<ConflictDialog
  bind:isOpen={moveAlertOpen}
  onCancel={cancelOperation}
  onResolve={async () => fileExplorer.updateEntries(fileExplorer.currentDir)}
  {conflictEntries}
/>

{#if contextMenuOpen && contextMenuEntry}
  <ContextMenu
    x={contextMenuX}
    y={contextMenuY}
    entry={contextMenuEntry}
    {fileExplorer}
    onClose={() => (contextMenuOpen = false)}
    onPaste={handlePaste}
  />
{/if}

<!-- onkeydown={fileExplorer.handleKeydown} -->

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div
  class="flex h-full w-full flex-col overflow-hidden focus:outline-none"
  tabindex="0"
  role="list"
  aria-label="File Explorer"
>
  <!-- Header -->
  <div class="border-border/60 bg-background sticky top-0 z-10 flex border-b px-2">
    {#each columns as column (column.key)}
      <Button.Root
        data-column-sorted={fileExplorer.sortedColumn === column.key}
        class="data-[column-sorted=true]:text-primary relative flex shrink-0 cursor-pointer items-center gap-1 px-3 py-2.5 text-[11px] font-semibold tracking-wider text-muted-foreground uppercase transition-colors select-none hover:text-primary"
        onclick={() => fileExplorer.sortColumns(column.key)}
        style="width: {column.width}px;"
      >
        <span>{column.label}</span>
        {#if fileExplorer.sortedColumn === column.key}
          <svg
            class="size-3 shrink-0"
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            {#if fileExplorer.sortedDirection === "asc"}
              <path d="m18 15-6-6-6 6" />
            {:else}
              <path d="m6 9 6 6 6-6" />
            {/if}
          </svg>
        {/if}
        <Separator.Root
          orientation="vertical"
          class="bg-border/60 hover:bg-accent active:bg-accent absolute top-2 right-0 h-[calc(100%-16px)] w-px cursor-col-resize rounded-full opacity-0 transition-all hover:opacity-100 select-none"
          onmousedown={(e) => startResize(column.key, e)}
          role="separator"
          aria-orientation="vertical"
          aria-label="Resize {column.label} column"
        />
      </Button.Root>
    {/each}
  </div>

  <!-- Rows -->
  <ScrollArea.Root class="flex-1 overflow-hidden" type="hover">
    <ScrollArea.Viewport
      class="h-full w-full"
      bind:ref={viewportEl}
      onscroll={(e) => (virtualScroll.scrollTop = e.currentTarget.scrollTop)}
    >
      <div class="relative py-1" style="height: {virtualScroll.totalHeight}px">
        <div style="transform: translateY({virtualScroll.offsetY}px)">
          {#each virtualScroll.visibleItems as entry, i (entry.path)}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <!-- svelte-ignore a11y_no_noninteractive_tabindex-->
            <div
              class="hover:bg-muted/50 data-[selected=true]:bg-accent/20 mx-2 flex cursor-pointer rounded-md transition-colors duration-100 data-[clipboard-cut=true]:opacity-40"
              role="listitem"
              onclick={(e) =>
                fileExplorer.handleEntryClick(
                  entry,
                  i + virtualScroll.visibleStart,
                  e,
                )}
              data-selected={fileExplorer.selection.isSelected(entry.path)}
              data-row-index={i + virtualScroll.visibleStart}
              data-clipboard-cut={fileExplorer.clipboard.isInClipboard(
                entry.path,
              ) && fileExplorer.clipboard.isCut}
              ondblclick={() => fileExplorer.openEntry(entry)}
              onmousedown={(e) => { if (e.button === 1) e.preventDefault(); }}
              onauxclick={(e) => { if (e.button === 1 && entry.isDir) onopennewtab?.(entry.path); }}
              oncontextmenu={(e) => handleContextMenu(e, entry, i + virtualScroll.visibleStart)}
              draggable="true"
              ondragstart={(e) => dragStart(e, entry.path, entry.isDir)}
              ondrop={(e) => dragDrop(e, entry.path)}
              ondragover={(e) => dragOver(e)}
            >
              <!-- Name -->
              <div
                class="flex shrink-0 items-center px-3 py-1.5"
                style="width: {getColumnWidth('name')}px;"
              >
                <div class="flex min-w-0 items-center gap-2">
                  <FileIcon file={entry} size={18} />
                  {#if fileExplorer.renamingPath === entry.path}
                    <input
                      type="text"
                      bind:value={fileExplorer.renameValue}
                      class="text-primary min-w-0 flex-1 rounded border border-border bg-background px-1 text-sm font-medium outline-none focus:ring-1 focus:ring-accent"
                      use:focusOnMount
                      onkeydown={(e) => {
                        if (e.key === "Enter") { e.preventDefault(); fileExplorer.commitRename(); }
                        if (e.key === "Escape") { e.preventDefault(); fileExplorer.cancelRename(); }
                        e.stopPropagation();
                      }}
                      onblur={() => fileExplorer.commitRename()}
                      onclick={(e) => e.stopPropagation()}
                      ondblclick={(e) => e.stopPropagation()}
                    />
                  {:else}
                    <span class="text-primary truncate text-sm font-medium">{entry.name}</span>
                  {/if}
                </div>
              </div>

              <!-- Date Modified -->
              <div
                class="flex shrink-0 items-center px-3 py-1.5 text-sm text-muted-foreground"
                style="width: {getColumnWidth('dateModified')}px;"
              >
                {formatDate(entry.dateModified)}
              </div>

              <!-- Last Opened -->
              <div
                class="flex shrink-0 items-center px-3 py-1.5 text-sm text-muted-foreground"
                style="width: {getColumnWidth('lastOpened')}px;"
              >
                {entry.lastOpened ? formatDate(entry.lastOpened) : "—"}
              </div>

              <!-- Type -->
              <div
                class="flex shrink-0 items-center px-3 py-1.5 text-sm text-muted-foreground"
                style="width: {getColumnWidth('fileType')}px;"
              >
                {entry.fileType}
              </div>

              <!-- Size -->
              <div
                class="flex shrink-0 items-center px-3 py-1.5 text-sm tabular-nums text-muted-foreground"
                style="width: {getColumnWidth('size')}px;"
              >
                {formatFileSize(entry.size)}
              </div>

              <!-- Tags -->
              <div
                class="flex shrink-0 items-center gap-1 px-3 py-1.5"
                style="width: {getColumnWidth('tags')}px;"
              >
                {#each entry.tagIds as tagId (tagId)}
                  <span
                    class="bg-accent/15 text-accent inline-flex shrink-0 items-center rounded-full px-2 py-0.5 text-xs font-medium"
                  >
                    {tagMap.get(tagId)?.name}
                  </span>
                {/each}
              </div>

              <!-- Status -->
              <div
                class="flex shrink-0 items-center gap-1 px-3 py-1.5"
                style="width: {getColumnWidth('status')}px;"
              >
                {#each entry.statusIds as statusId (statusId)}
                  <span
                    class="bg-muted/80 text-muted-foreground inline-flex shrink-0 items-center rounded-full px-2 py-0.5 text-xs font-medium"
                  >
                    {statusMap.get(statusId)?.name}
                  </span>
                {/each}
              </div>
            </div>
          {/each}
        </div>
      </div>
    </ScrollArea.Viewport>
    <ScrollArea.Scrollbar
      orientation="vertical"
      class="flex h-full w-1.5 touch-none p-px transition-opacity select-none"
    >
      <ScrollArea.Thumb
        class="bg-border/50 hover:bg-border relative flex-1 rounded-full transition-colors"
      />
    </ScrollArea.Scrollbar>
  </ScrollArea.Root>
</div>
