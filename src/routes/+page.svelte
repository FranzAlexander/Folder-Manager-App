<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { ArrowBigLeft, ArrowBigRight } from "@lucide/svelte";
  import type { Status, Tag } from "$lib/types";
  import ItemSelector from "$lib/components/ItemSelector.svelte";
  import FileTable from "$lib/components/FileTable.svelte";
  import { FileExplorerState } from "$lib/stores/FileExplorerState.svelte";
  import { Button } from "bits-ui";

  let tags: Tag[] = $state([]);
  let statusList: Status[] = $state([]);

  const fileExplorer = new FileExplorerState();

  onMount(async () => {
    const rootDir: string | null = await invoke("get_root_directory");
    if (!rootDir) {
      fileExplorer.showSetup = true;
      return;
    }

    await fileExplorer.setRootDir(rootDir);
    tags = await invoke("get_tags");
    statusList = await invoke("get_status");
  });

  async function createTag(search: string) {
    const tag: Tag = await invoke("create_tag", { tag: search });
    tags = [...tags, tag];
  }

  async function assignTag(id: number) {
    if (!fileExplorer.selectedEntry) return;

    const result = await invoke("assign_tag", {
      path: fileExplorer.selectedEntryPath,
      tagId: id,
    });

    fileExplorer.entries = fileExplorer.entries.map((entry) =>
      entry.path === fileExplorer.selectedEntryPath
        ? { ...entry, tagIds: [...entry.tagIds, id] }
        : entry,
    );

    fileExplorer.selectedEntry =
      fileExplorer.entries.find(
        (e) => e.path === fileExplorer.selectedEntryPath,
      ) || null;
  }

  async function createStatus(search: string) {
    const status: Status = await invoke("create_status", { status: search });
    statusList = [...statusList, status];
  }

  async function assignStatus(id: number) {
    if (!fileExplorer.selectedEntry) return;

    const result = await invoke("assign_status", {
      path: fileExplorer.selectedEntryPath,
      statusId: id,
    });

    fileExplorer.entries = fileExplorer.entries.map((entry) =>
      entry.path === fileExplorer.selectedEntryPath
        ? { ...entry, statusIds: [...entry.statusIds, id] }
        : entry,
    );

    fileExplorer.selectedEntry =
      fileExplorer.entries.find(
        (e) => e.path === fileExplorer.selectedEntryPath,
      ) || null;
  }
</script>

<svelte:window onmouseup={fileExplorer.handleMouseButton} />

<main class="bg-background w-full h-screen m-0 text-primary flex flex-col">
  <div class="border-b border-border px-4 py-2 flex items-center gap-3">
    <div class="flex items-center gap-1">
      <Button.Root
        disabled={fileExplorer.historyIndex === 0}
        class="p-1.5 rounded-md hover:bg-muted hover:cursor-pointer transition-colors disabled:opacity-50 disabled:cursor-not-allowed group"
        onclick={fileExplorer.goBack}
        title="Back"
      >
        <ArrowBigLeft />
      </Button.Root>
      <Button.Root
        disabled={fileExplorer.historyIndex >= fileExplorer.history.length - 1}
        class="p-1.5 rounded-md hover:bg-muted hover:cursor-pointer transition-colors disabled:opacity-50 disabled:cursor-not-allowed group"
        onclick={fileExplorer.goForward}
        title="Forward"
      >
        <ArrowBigRight />
      </Button.Root>
    </div>
    <div class="h-4 w-px bg-border/40 mx-1"></div>

    <input bind:value={fileExplorer.currentDir} />

    <ItemSelector
      items={tags}
      name={"Tags"}
      oncreate={createTag}
      action={assignTag}
      disabled={!fileExplorer.selectedEntry}
    />

    <ItemSelector
      items={statusList}
      name={"Status"}
      oncreate={createStatus}
      action={assignStatus}
      disabled={!fileExplorer.selectedEntry}
    />
  </div>

  {#if fileExplorer.rootDir && !fileExplorer.showSetup}
    <FileTable {fileExplorer} {tags} {statusList} />
  {:else}
    <div>
      <h2>Select Root Directory</h2>
      <button onclick={fileExplorer.selectDirectory}>Choose Directory</button>
    </div>
  {/if}
</main>
