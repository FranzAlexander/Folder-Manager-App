<script lang="ts">
  import { DropdownMenu } from "bits-ui";
  import { EllipsisVertical, Folder } from "@lucide/svelte";
  import type { FileExplorerState } from "$lib/state/FileExplorerState.svelte";

  let { fileExplorer }: { fileExplorer: FileExplorerState } = $props();
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger
    class="hover:bg-foreground inline-flex cursor-pointer items-center rounded-full p-1.5 select-none"
  >
    <EllipsisVertical />
  </DropdownMenu.Trigger>
  <DropdownMenu.Portal>
    <DropdownMenu.Content
      class="bg-popover border-border text-primary z-50 w-60 rounded-xl border px-1 py-1.5 shadow-xl outline-hidden focus-visible:outline-none"
      sideOffset={6}
    >
      <DropdownMenu.Item
        onSelect={fileExplorer.selectDirectory}
        class="hover:bg-muted flex cursor-pointer items-center gap-2 rounded-xl px-1.5 py-1"
      >
        <Folder />
        <p>Change Root</p>
      </DropdownMenu.Item>
    </DropdownMenu.Content>
  </DropdownMenu.Portal>
</DropdownMenu.Root>
