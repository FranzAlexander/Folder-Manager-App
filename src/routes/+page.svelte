<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import FileTable from "$lib/components/FileTable.svelte";
  import { FileExplorerState } from "$lib/state/FileExplorerState.svelte";
  import Toolbar from "$lib/components/toolbar/Toolbar.svelte";

  const fileExplorer = new FileExplorerState();
  const { tags, statuses } = fileExplorer;

  onMount(async () => {
    const rootDir: string | null = await invoke("get_root_directory");
    if (!rootDir) {
      fileExplorer.showSetup = true;
      return;
    }

    await fileExplorer.setRootDir(rootDir);
    await tags.loadAllTags();
    await statuses.loadAllStatuses();
  });
</script>

<svelte:window onmouseup={fileExplorer.handleMouseButton} />

<main class="bg-background text-primary m-0 flex h-screen w-full flex-col">
  <Toolbar {fileExplorer} />

  {#if fileExplorer.rootDir && !fileExplorer.showSetup}
    <FileTable
      {fileExplorer}
      tags={tags.allTags}
      statusList={statuses.allStatuses}
    />
  {:else}
    <div>
      <h2>Select Root Directory</h2>
      <button onclick={fileExplorer.selectDirectory}>Choose Directory</button>
    </div>
  {/if}
</main>
