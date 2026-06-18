<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import FileTable from "$lib/components/FileTable.svelte";
  import { TabsState } from "$lib/state/TabsState.svelte";
  import NavBar from "$lib/components/navbar/NavBar.svelte";
  import Toolbar from "$lib/components/Toolbar.svelte";
  import TabBar from "$lib/components/TabBar.svelte";

  const tabsState = new TabsState();

  const fileExplorer = $derived(tabsState.activeTab);
  const tags = $derived(fileExplorer.tags);
  const statuses = $derived(fileExplorer.statuses);

  onMount(async () => {
    const firstTab = tabsState.activeTab;
    const rootDir: string | null = await invoke("get_root_directory");
    if (!rootDir) {
      firstTab.showSetup = true;
      return;
    }

    await firstTab.setRootDir(rootDir);
    await Promise.all([
      firstTab.tags.loadAllTags(),
      firstTab.statuses.loadAllStatuses(),
      firstTab.trash.loadCount(),
    ]);
  });

  // The backend has a single OS watcher; keep it pointed at the active tab's
  // directory. Re-runs whenever the active tab or its current dir changes.
  $effect(() => {
    const dir = fileExplorer.currentDir;
    if (dir && dir !== "trash://") {
      invoke("watch_directory", { path: dir });
    } else {
      invoke("unwatch_directory");
    }
  });

  // Single "dir-changed" listener for the whole app: debounced-refresh the
  // active tab when its directory changes on disk.
  onMount(() => {
    let refreshTimeout: ReturnType<typeof setTimeout> | null = null;
    const unlisten = listen<string>("dir-changed", (event) => {
      const tab = tabsState.activeTab;
      if (event.payload !== tab.currentDir) return;
      if (tab.search.isSearching) return;
      if (refreshTimeout !== null) clearTimeout(refreshTimeout);
      refreshTimeout = setTimeout(() => {
        tab.updateEntries(tab.currentDir);
        refreshTimeout = null;
      }, 300);
    });
    return () => {
      if (refreshTimeout !== null) clearTimeout(refreshTimeout);
      unlisten.then((fn) => fn());
    };
  });

  async function newTab() {
    await tabsState.newTab();
  }

  function handleWindowKeydown(e: KeyboardEvent) {
    const ctrl = e.ctrlKey || e.metaKey;
    if (ctrl && e.key === "t") {
      e.preventDefault();
      newTab();
      return;
    }
    if (ctrl && e.key === "w") {
      e.preventDefault();
      tabsState.closeTab(tabsState.activeIndex);
      return;
    }
    if (ctrl && e.key === "Tab") {
      e.preventDefault();
      e.shiftKey ? tabsState.prevTab() : tabsState.nextTab();
    }
  }
</script>

<svelte:window
  onmouseup={(e) => fileExplorer.handleMouseButton(e)}
  onkeydown={handleWindowKeydown}
/>

<main class="bg-background text-primary m-0 flex h-screen w-full flex-col">
  <TabBar {tabsState} onnew={newTab} />
  <NavBar {fileExplorer} />
  <Toolbar {fileExplorer} />

  {#if fileExplorer.rootDir && !fileExplorer.showSetup}
    <FileTable
      {fileExplorer}
      tags={tags.allTags}
      statusList={statuses.allStatuses}
      onopennewtab={(path) => tabsState.openInNewTab(path, false)}
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
