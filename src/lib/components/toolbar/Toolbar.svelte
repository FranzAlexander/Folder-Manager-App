<script lang="ts">
  import type { FileExplorerState } from "$lib/state/FileExplorerState.svelte";
  import ActionButtons from "./ActionButtons.svelte";
  import NavigationButtons from "./NavigationButtons.svelte";
  import PathBar from "./PathBar.svelte";
  import SearchBar from "./SearchBar.svelte";

  let { fileExplorer }: { fileExplorer: FileExplorerState } = $props();
</script>

<div class="border-border border-b px-4 py-2 backdrop-blur-sm">
  <div class=" flex items-center gap-6">
    <NavigationButtons {fileExplorer} />
    <PathBar {fileExplorer} />
    <SearchBar {fileExplorer} />
    <ActionButtons {fileExplorer} />
  </div>
</div>
