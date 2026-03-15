<script lang="ts">
  import type { FileExplorerState } from "$lib/state/FileExplorerState.svelte";

  let { fileExplorer }: { fileExplorer: FileExplorerState } = $props();

  let draft = $state<string | null>(null);

  const displayValue = $derived(draft ?? fileExplorer.currentDir);

  function handleInput(e: Event) {
    draft = (e.currentTarget as HTMLInputElement).value;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && draft !== null) {
      fileExplorer.navigateToDirectory(draft);
      draft = null;
      (e.currentTarget as HTMLInputElement).blur();
    } else if (e.key === "Escape") {
      draft = null;
      (e.currentTarget as HTMLInputElement).blur();
    }
  }

  function handleBlur() {
    draft = null;
  }
</script>

<div class="flex flex-1 items-center">
  <input
    value={displayValue}
    oninput={handleInput}
    onkeydown={handleKeydown}
    onblur={handleBlur}
    class="border-border bg-muted/40 text-muted-foreground focus:text-primary focus:border-accent/60 w-full rounded-md border px-3 py-1.5 text-sm outline-none transition-colors focus:bg-transparent"
  />
</div>
