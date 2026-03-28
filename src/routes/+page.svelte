<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import FileTable from "$lib/components/FileTable.svelte";
  import { FileExplorerState } from "$lib/state/FileExplorerState.svelte";
  import NavBar from "$lib/components/navbar/NavBar.svelte";
  import Toolbar from "$lib/components/Toolbar.svelte";

  const fileExplorer = new FileExplorerState();
  const { tags, statuses } = fileExplorer;

  onMount(async () => {
    const rootDir: string | null = await invoke("get_root_directory");
    if (!rootDir) {
      fileExplorer.showSetup = true;
      return;
    }

    await fileExplorer.setRootDir(rootDir);
    await Promise.all([tags.loadAllTags(), statuses.loadAllStatuses(), fileExplorer.trash.loadCount()]);
  });
</script>

<svelte:window onmouseup={fileExplorer.handleMouseButton} />

<main class="bg-background text-primary m-0 flex h-screen w-full flex-col">
  <NavBar {fileExplorer} />
  <Toolbar {fileExplorer} />

  {#if fileExplorer.rootDir && !fileExplorer.showSetup}
    <FileTable
      {fileExplorer}
      tags={tags.allTags}
      statusList={statuses.allStatuses}
    />
  {:else}
    <div class="flex flex-1 flex-col items-center justify-center gap-3">
      <h2 class="text-primary text-base font-semibold">
        Select a Root Directory
      </h2>
      <button
        onclick={fileExplorer.selectDirectory}
        class="bg-accent text-accent-foreground hover:bg-accent/90 cursor-pointer rounded-md px-4 py-2 text-sm font-medium transition-colors"
      >
        Choose Directory
      </button>
    </div>
  {/if}
</main>
