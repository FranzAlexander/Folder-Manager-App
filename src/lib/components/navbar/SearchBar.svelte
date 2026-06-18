<script lang="ts">
  import type { FileExplorerState } from "$lib/state/FileExplorerState.svelte";

  let { fileExplorer }: { fileExplorer: FileExplorerState } = $props();
</script>

<div class="flex min-w-72 items-center rounded-lg">
  <input
    type="text"
    class="text-primary/80 bg-foreground flex-1 rounded-md p-1 text-sm outline-none"
    placeholder="Search here"
    bind:value={fileExplorer.search.query}
    oninput={() => fileExplorer.search.run(fileExplorer.search.query)}
  />
</div>
