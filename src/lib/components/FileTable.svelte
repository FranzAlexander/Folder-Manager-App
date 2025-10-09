<script lang="ts">
  import type { ColumnKey, FileSystemEntry, Status, Tag } from "$lib/types";
  import { Button, ScrollArea, Separator } from "bits-ui";
  import FileIcon from "./FileIcon.svelte";
  import { formateDate, formatFileSize } from "$lib/utils/formatters";
  import type { FileExplorerState } from "$lib/stores/FileExplorerState.svelte";

  let {
    fileExplorer,
    tags,
    statusList,
  }: {
    fileExplorer: FileExplorerState;
    tags: Tag[];
    statusList: Status[];
  } = $props();

  let columns = $state([
    { key: "name" as ColumnKey, label: "Name", width: 300 },
    { key: "dateModified" as ColumnKey, label: "Date Modified", width: 180 },
    { key: "fileType" as ColumnKey, label: "Type", width: 120 },
    { key: "size" as ColumnKey, label: "Size", width: 100 },
    { key: "tags" as ColumnKey, label: "Tags", width: 200 },
    { key: "status" as ColumnKey, label: "Status", width: 150 },
  ]);

  let resizingColumn = $state<ColumnKey | null>(null);
  let resizeStartX = $state(0);
  let resizeStartWidth = $state(0);

  let scrollTop = $state(0);

  const itemHeight = 34;
  const containerHeight = 624;
  const overscan = 5;

  const visableStart = $derived(
    Math.max(0, Math.floor(scrollTop / itemHeight) - overscan),
  );

  const visableEnd = $derived(
    Math.min(
      fileExplorer.entries.length,
      Math.ceil((scrollTop + containerHeight) / itemHeight) + overscan,
    ),
  );

  const visableItems = $derived(
    fileExplorer.entries.slice(visableStart, visableEnd),
  );

  const totalHeight = $derived(fileExplorer.entries.length * itemHeight);

  const offsetY = $derived(visableStart * itemHeight);

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

  function onKeydown(e: KeyboardEvent) {
    const idx = fileExplorer.entries.findIndex(
      (x) => x.path === fileExplorer.selectedEntryPath,
    );
    if (idx === -1) return;

    e.preventDefault();

    if (e.key === "ArrowUp") {
      const prev = fileExplorer.entries[idx - 1];
      if (prev) fileExplorer.selectEntry(prev);
    }

    if (e.key === "ArrowDown") {
      const next = fileExplorer.entries[idx + 1];
      if (next) fileExplorer.selectEntry(next);
    }
  }
</script>

<svelte:window onmousemove={handleMouseMove} onmouseup={stopResize} />

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div
  class="flex h-full w-full flex-col overflow-hidden"
  tabindex="0"
  role="list"
  onkeydown={onKeydown}
  aria-label="File Explorer"
>
  <div class="border-border bg-background sticky top-0 z-10 flex border-b">
    {#each columns as column (column.key)}
      <Button.Root
        data-column-sorted={fileExplorer.sortedColumn === column.key}
        class="data-[column-sorted=true]:bg-muted/50 hover:bg-muted relative flex shrink-0 cursor-pointer items-center px-4 py-3 text-sm font-semibold select-none"
        onclick={() => {
          fileExplorer.sortColumns(column.key);
        }}
        style="width: {column.width}px;"
      >
        <span>{column.label}</span>
        <Separator.Root
          orientation="vertical"
          class="bg-border hover:bg-muted active:bg-muted-foreground absolute top-0 right-0 h-full w-0.5 cursor-col-resize select-none"
          onmousedown={(e) => startResize(column.key, e)}
          role="separator"
          aria-orientation="vertical"
          aria-label="Resize {column.label} column"
        />
      </Button.Root>
    {/each}
  </div>
  <ScrollArea.Root class="flex-1 overflow-hidden">
    <ScrollArea.Viewport
      class="h-full w-full scroll-smooth"
      onscroll={(e) => (scrollTop = e.currentTarget.scrollTop)}
    >
      <div class="relative" style="height: {totalHeight}px">
        <div style="transform: translateY({offsetY}px)">
          {#each visableItems as entry, i (entry.path)}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <!-- svelte-ignore a11y_no_noninteractive_tabindex-->
            <div
              class="hover:bg-muted data-[selected=true]:bg-muted flex cursor-pointer px-2 py-2.5 transition-colors"
              role="listitem"
              data-selected={fileExplorer.selectedEntryPath === entry.path}
              onclick={() => fileExplorer.selectEntry(entry)}
              data-row-index={i}
              ondblclick={() => fileExplorer.openEntry(entry)}
            >
              <div
                class="flex shrink-0 items-center text-sm"
                style="width: {getColumnWidth('name')}px;"
              >
                <div class="flex items-center gap-2 overflow-hidden">
                  <FileIcon file={entry} size={24} />
                  <span class="truncate">{entry.name}</span>
                </div>
              </div>
              <div
                class="flex shrink-0 items-center text-sm"
                style="width: {getColumnWidth('dateModified')}px;"
              >
                {formateDate(entry.dateModified)}
              </div>
              <div
                class="flex shrink-0 items-center text-sm"
                style="width: {getColumnWidth('fileType')}px;"
              >
                {entry.fileType}
              </div>
              <div
                class="flex shrink-0 items-center text-sm"
                style="width: {getColumnWidth('size')}px;"
              >
                {formatFileSize(entry.size)}
              </div>
              <div
                class="flex shrink-0 items-center gap-1 text-sm"
                style="width: {getColumnWidth('tags')}px;"
              >
                {#each entry.tagIds as tagId (tagId)}
                  <span
                    class="bg-muted text-primary border-border inline-flex w-fit shrink-0 items-center justify-center rounded-md border px-2 py-0.5 text-sm font-medium"
                  >
                    {tags.find((t) => t.id === tagId)?.name}
                  </span>
                {/each}
              </div>
              <div
                class="flex shrink-0 items-center text-sm"
                style="width: {getColumnWidth('status')}px;"
              >
                {#each entry.statusIds as statusId (statusId)}
                  <span
                    class="bg-muted text-primary border-border inline-flex w-fit shrink-0 items-center justify-center rounded-md border px-2 py-0.5 text-sm font-medium"
                  >
                    {statusList.find((s) => s.id === statusId)?.name}
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
      class="hover:bg-muted flex h-full w-2.5 touch-none border-l border-l-transparent p-px transition-colors select-none"
    >
      <ScrollArea.Thumb
        class="bg-border/60 hover:bg-border relative flex-1 rounded-full transition-colors"
      />
    </ScrollArea.Scrollbar>
  </ScrollArea.Root>
</div>
