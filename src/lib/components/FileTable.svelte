<script lang="ts">
  import type { FileSystemEntry, Status, Tag } from "$lib/types";
  import { Separator } from "bits-ui";
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

  type ColumnKey =
    | "name"
    | "dateModified"
    | "fileType"
    | "size"
    | "tags"
    | "status";

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
  class="w-full h-full flex flex-col overflow-hidden"
  tabindex="0"
  role="list"
  onkeydown={onKeydown}
  aria-label="File Explorer"
>
  <div class="flex border-b border-border bg-background sticky top-0 z-10">
    {#each columns as column (column.key)}
      <div
        class="relative px-4 py-3 font-semibold text-sm select-none flex items-center shrink-0"
        style="width: {column.width}px;"
      >
        <span>{column.label}</span>

        <Separator.Root
          orientation="vertical"
          class="absolute top-0 right-0 h-full w-0.5 bg-border cursor-col-resize select-none hover:bg-muted active:bg-muted-foreground"
          onmousedown={(e) => startResize(column.key, e)}
          role="separator"
          aria-orientation="vertical"
          aria-label="Resize {column.label} column"
        />
      </div>
    {/each}
  </div>

  <div class="flex-1 overflow-y-auto overflow-x-hidden">
    {#each fileExplorer.entries as entry, i (entry.path)}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <!-- svelte-ignore a11y_no_noninteractive_tabindex-->
      <div
        class="flex cursor-pointer hover:bg-muted p-2 data-[selected=true]:bg-muted"
        role="listitem"
        data-selected={fileExplorer.selectedEntry?.name === entry.name}
        onclick={() => fileExplorer.selectEntry(entry)}
        data-row-index={i}
        ondblclick={() => fileExplorer.openEntry(entry)}
      >
        <div
          class="flex items-center shrink-0 text-sm"
          style="width: {getColumnWidth('name')}px;"
        >
          <div class="flex items-center gap-2 overflow-hidden">
            <FileIcon file={entry} size={24} />
            <span class="truncate">{entry.name}</span>
          </div>
        </div>
        <div
          class="flex items-center shrink-0 text-sm"
          style="width: {getColumnWidth('dateModified')}px;"
        >
          {formateDate(entry.dateModified)}
        </div>
        <div
          class="flex items-center shrink-0 text-sm"
          style="width: {getColumnWidth('fileType')}px;"
        >
          {entry.fileType}
        </div>
        <div
          class="flex items-center shrink-0 text-sm"
          style="width: {getColumnWidth('size')}px;"
        >
          {formatFileSize(entry.size)}
        </div>
        <div
          class="flex items-center shrink-0 text-sm"
          style="width: {getColumnWidth('tags')}px;"
        >
          {#each entry.tagIds as tagId (tagId)}
            <span
              class="bg-muted text-primary border-border rounded-md border px-2 py-0.5 text-sm font-medium items-center inline-flex w-fit shrink-0 justify-center"
            >
              {tags.find((t) => t.id === tagId)?.name}
            </span>
          {/each}
        </div>
        <div
          class="flex items-center shrink-0 text-sm"
          style="width: {getColumnWidth('status')}px;"
        >
          {#each entry.statusIds as statusId (statusId)}
            <span
              class="bg-muted text-primary border-border rounded-md border px-2 py-0.5 text-sm font-medium items-center inline-flex w-fit shrink-0 justify-center"
            >
              {statusList.find((s) => s.id === statusId)?.name}
            </span>
          {/each}
        </div>
      </div>
    {/each}
  </div>
</div>
