<script lang="ts">
  import type { FileExplorerState } from "$lib/state/FileExplorerState.svelte";
  import { Button } from "bits-ui";
  import ActionButtons from "./ActionButtons.svelte";
  import NavigationButtons from "./NavigationButtons.svelte";
  import OptionsMenu from "./OptionsMenu.svelte";
  import PathBar from "./PathBar.svelte";
  import SearchBar from "./SearchBar.svelte";
  import { Trash2 } from "@lucide/svelte";

  let { fileExplorer }: { fileExplorer: FileExplorerState } = $props();
</script>

<div class="border-border border-b px-4 py-2 backdrop-blur-sm">
  <div class=" flex items-center gap-6">
    <NavigationButtons {fileExplorer} />
    <PathBar {fileExplorer} />
    <SearchBar {fileExplorer} />
    <Button.Root
      class="hover:bg-muted cursor-pointer rounded-md p-1.5 transition-colors"
      onclick={() => fileExplorer.navigateToDirectory("trash://")}
    >
      <Trash2 class="size-6" />
    </Button.Root>
    <ActionButtons {fileExplorer} />
    <OptionsMenu {fileExplorer} />
  </div>
</div>
