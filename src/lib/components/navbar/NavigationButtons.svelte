<script lang="ts">
  import type { FileExplorerState } from "$lib/state/FileExplorerState.svelte";
  import { ArrowBigLeft, ArrowBigRight } from "@lucide/svelte";
  import { Button } from "bits-ui";

  let { fileExplorer }: { fileExplorer: FileExplorerState } = $props();
</script>

<div class="flex items-center gap-0.5">
  <Button.Root
    disabled={!fileExplorer.history.canGoBack}
    class="hover:bg-muted group rounded-md p-1.5 transition-colors hover:cursor-pointer disabled:cursor-not-allowed disabled:opacity-50"
    onclick={fileExplorer.goBack}
    title="Back"
  >
    <ArrowBigLeft class="size-4" />
  </Button.Root>
  <Button.Root
    disabled={!fileExplorer.history.canGoForward}
    class="hover:bg-muted group rounded-md p-1.5 transition-colors hover:cursor-pointer disabled:cursor-not-allowed disabled:opacity-50"
    onclick={fileExplorer.goForward}
    title="Forward"
  >
    <ArrowBigRight class="size-4" />
  </Button.Root>
</div>
