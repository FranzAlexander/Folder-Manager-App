<script lang="ts">
  import type { FileExplorerState } from "$lib/state/FileExplorerState.svelte";

  let { fileExplorer }: { fileExplorer: FileExplorerState } = $props();
</script>

<div class="flex flex-1 items-center rounded-lg">
  <input
    bind:value={fileExplorer.currentDir}
    class="text-primary/80 bg-foreground flex-1 rounded-md p-1 text-sm outline-none"
  />
</div>
