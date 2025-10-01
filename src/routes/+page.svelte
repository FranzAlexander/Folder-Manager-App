<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { ArrowBigLeft, ArrowBigRight } from "@lucide/svelte";
  import FileIcon from "./FileIcon.svelte";
  import type { DirectoryEntry, Tag } from "$lib/types";
  import ItemSelector from "$lib/components/ItemSelector.svelte";
  import { formateDate, formatFileSize } from "$lib/utils/formatters";

  let rootDir = $state<string | null>(null);
  let history = $state<string[]>([]);
  let historyIndex = $state(0);
  let currentDir = $derived(history[historyIndex] || "");
  let selectedEntry = $state<DirectoryEntry | null>(null);
  let showSetup = $state(false);
  let entries: DirectoryEntry[] = $state([]);

  let tags: Tag[] = $state([]);

  onMount(async () => {
    rootDir = await invoke("get_root_directory");
    if (!rootDir) {
      showSetup = true;
      return;
    }

    history = [rootDir];
    entries = await invoke("read_directory", { path: rootDir });
    tags = await invoke("get_tags");
  });

  async function selectDirectory() {
    try {
      const selected = await open({
        multiple: false,
        directory: true,
        title: "Select Root Directory",
      });

      if (selected) {
        await invoke("set_root_directory", { path: selected });
        rootDir = selected;
        history.push(currentDir);
        showSetup = false;
      }
    } catch (error) {
      console.error("Failed to select directory:", error);
    }
  }

  async function navigateToDirectory(path: string) {
    if (historyIndex < history.length - 1) {
      history = history.slice(0, historyIndex + 1);
    }

    history.push(path);
    historyIndex = history.length - 1;

    entries = await invoke("read_directory", { path: currentDir });
  }

  async function goBack() {
    if (historyIndex > 0) {
      historyIndex--;
      entries = await invoke("read_directory", { path: currentDir });
    }
  }

  async function goForward() {
    if (historyIndex < history.length - 1) {
      historyIndex++;
      entries = await invoke("read_directory", { path: currentDir });
    }
  }

  function selectEntry(entry: DirectoryEntry) {
    selectedEntry = entry;
  }

  function handleMouseButton(event: MouseEvent) {
    if (event.button === 3) goBack();
    if (event.button === 4) goForward();
  }

  async function createTag(search: string) {
    const tag: Tag = await invoke("create_tag", { tag: search });
    tags = [...tags, tag];
  }

  async function assignTag(id: number) {
    if (!selectedEntry) return;

    const result = await invoke("assign_tag", {
      path: selectedEntry.path,
      tag_id: id,
    });

    entries = entries.map((entry) =>
      entry.path === selectedEntry?.path
        ? { ...entry, tagIds: [...entry.tagIds, id] }
        : entry,
    );

    selectedEntry = entries.find((e) => e.path === selectedEntry?.path) || null;
  }
</script>

<svelte:window onmouseup={handleMouseButton} />

<main class="bg-background w-full h-screen m-0 text-primary">
  <div class="border-b border-border px-4 py-2 flex items-center gap-3">
    <div class="flex items-center gap-1">
      <button
        disabled={historyIndex === 0}
        class="p-1.5 rounded-md hover:bg-muted hover:cursor-pointer transition-colors disabled:opacity-50 disabled:cursor-not-allowed group"
        onclick={goBack}
        title="Back"><ArrowBigLeft /></button
      >
      <button
        disabled={historyIndex >= history.length - 1}
        class="p-1.5 rounded-md hover:bg-muted hover:cursor-pointer transition-colors disabled:opacity-50 disabled:cursor-not-allowed group"
        onclick={goForward}
        title="Forward"><ArrowBigRight /></button
      >
    </div>
    <div class="h-4 w-px bg-border/40 mx-1"></div>

    <input bind:value={currentDir} />

    <ItemSelector
      items={tags}
      name={"Tags"}
      oncreate={createTag}
      action={assignTag}
      disabled={selectedEntry === null}
    />
    <!-- <button
      disabled={!selectedEntry}
      class="bg-foreground py-2 px-4 border border-border-primary rounded-lg hover:bg-hover hover:border-foreground disabled:opacity-50 disabled:cursor-not-allowed transition-colors duration-150 cursor-pointer"
      >Tags</button
    > -->

    <!-- <Popover.Root>
      <Popover.Trigger
        class="border-border border py-2 px-4 rounded-lg hover:bg-accent inline-flex hover:cursor-pointer"
        >Tags</Popover.Trigger
      >
      <Popover.Portal>
        <Popover.Content class="z-50 w-90" sideOffset={8}>
          <Command.Root
            class="flex h-full w-full border-border border bg-foreground  rounded-lg flex-col text-primary "
          >
            <div class="flex items-center gap-2 border-b pl-3 pr-8">
              <Command.Input
                bind:value={search}
                onkeydown={createTag}
                class="outline-hidden inline-flex h-10 w-full rounded-md py-3 bg-transparent text-sm disabled:cursor-not-allowed disabled:opacity-50"
              />
            </div>
            <Command.List>
              <Command.Viewport>
                <Command.Empty>Press Enter to create new tag.</Command.Empty>
              </Command.Viewport>
              {#each tags as tag}
                <Command.Item>
                  {tag.name}
                </Command.Item>
              {/each}
            </Command.List>
          </Command.Root>
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root> -->
    <button
      disabled={!selectedEntry}
      class="bg-foreground py-2 px-4 border border-primary rounded-lg hover:bg-hover hover:border-foreground disabled:opacity-50 disabled:cursor-not-allowed transition-colors duration-150 cursor-pointer"
      >Filter</button
    >
  </div>
  {#if rootDir && !showSetup}
    <div class="relative w-full overflow-x-auto">
      <table class="w-full table-auto">
        <thead>
          <tr>
            <th>Name</th>
            <th>Date Modified</th>
            <th>File Type</th>
            <th>Size</th>
            <th>Tags</th>
            <th>Status</th>
          </tr>
        </thead>
        <tbody>
          {#each entries as entry}
            <tr
              data-selected={selectedEntry?.name === entry.name}
              onclick={() => selectEntry(entry)}
              ondblclick={() => {
                if (entry.isDir) navigateToDirectory(entry.path);
              }}
              class="data-[selected=true]:bg-foreground/80 flex"
            >
              <td class=" flex"
                ><FileIcon file={entry} size={24} /> {entry.name}</td
              >
              <td>{formateDate(entry.dateModified)}</td>
              <td>{entry.type}</td>
              <td>{formatFileSize(entry.size)}</td>
              <td>
                {#each entry.tagIds as tagId (tagId)}
                  {tags.find((t) => t.id === tagId)?.name}
                {/each}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
    <!-- <ul>
      {#each entries as entry}
        <li
          data-selected={selectedEntry?.name === entry.name}
          class="flex gap-2 data-[selected=true]:bg-bg-secondary/80"
        >
          <FileIcon file={entry} size={24} />
          {#if entry.is_dir}
            <button
              onclick={() => selectEntry(entry)}
              ondblclick={() => navigateToDirectory(entry.path)}
              >{entry.name}</button
            >
          {:else}
            <button onclick={() => selectEntry(entry)}>{entry.name}</button>
          {/if}
        </li>
      {/each}
    </ul> -->
  {:else}
    <div>
      <h2>Select Root Directory</h2>
      <button onclick={selectDirectory}>Choose Directory</button>
    </div>
  {/if}
</main>
