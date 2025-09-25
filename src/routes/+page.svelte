<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { ArrowBigLeft, ArrowBigRight } from "@lucide/svelte";
  import FileIcon from "./FileIcon.svelte";

  interface DirectoryEntry {
    name: string;
    is_dir: boolean;
    is_file: boolean;
    size: number | null;
  }

  let rootDir = $state<string | null>(null);
  let currentDir = $state("");
  let selectedEntry = $state<DirectoryEntry | null>(null);
  let showSetup = $state(false);
  let entries: DirectoryEntry[] = $state([]);

  onMount(async () => {
    rootDir = await invoke("get_root_directory");
    if (!rootDir) {
      showSetup = true;
    }

    currentDir = rootDir || "";

    entries = await invoke("read_directory", { path: rootDir });
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
        currentDir = rootDir;
        showSetup = false;
      }
    } catch (error) {
      console.error("Failed to select directory:", error);
    }
  }

  async function navigateToDirectory(name: string) {
    currentDir = rootDir + name;

    entries = await invoke("read_directory", { path: currentDir });
  }

  async function selectEntry(entry: DirectoryEntry) {
    selectedEntry = entry;
  }
</script>

<main
  class="m-0 h-svh font-family-primary bg-linear-to-br from-bg-primary to-bg-secondary text-text-primary w-full"
>
  <div class="border-b border-border-primary p-2 flex items-center gap-4">
    <div class="flex gap-2">
      <button><ArrowBigLeft class="size-4" /> </button>
      <button><ArrowBigRight class="size-4" /></button>
    </div>
    <h3 class="ml-4">{currentDir}</h3>
    <button disabled={!selectedEntry}>Tags</button>
    <button disabled={!selectedEntry}>Filter</button>
  </div>
  {#if rootDir && !showSetup}
    <ul>
      {#each entries as entry}
        <li
          data-selected={selectedEntry?.name === entry.name}
          class="flex gap-2 data-[selected=true]:bg-bg-secondary/80"
        >
          <FileIcon file={entry} size={24} />
          {#if entry.is_dir}
            <button
              onclick={() => selectEntry(entry)}
              ondblclick={() => navigateToDirectory(entry.name)}
              >{entry.name}</button
            >
          {:else}
            <button onclick={() => selectEntry(entry)}>{entry.name}</button>
          {/if}
        </li>
      {/each}
    </ul>
  {:else}
    <div>
      <h2>Select Root Directory</h2>
      <button onclick={selectDirectory}>Choose Directory</button>
    </div>
  {/if}
</main>
