<script lang="ts">
  import type { ExtractProgress } from "$lib/types";
  import ExtractProgressBar from "./ExtractProgressBar.svelte";
  import { FolderSync } from "@lucide/svelte";
  import { Button, Dialog } from "bits-ui";

  let {
    zipName,
    folderName,
    isExtracting,
    progress,
    onUpdate,
    onIgnore,
    onCancelExtract,
  }: {
    zipName: string;
    folderName: string;
    isExtracting: boolean;
    progress: ExtractProgress | null;
    onUpdate: () => void;
    onIgnore: () => void;
    onCancelExtract: () => void;
  } = $props();

  let isOpen = $state(true);

  function handleOpenChange(open: boolean) {
    // Dismissing without acting = ignore; blocked while extracting.
    if (!open && !isExtracting) onIgnore();
  }
</script>

<Dialog.Root bind:open={isOpen} onOpenChange={handleOpenChange}>
  <Dialog.Portal>
    <Dialog.Overlay
      class="data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 fixed inset-0 z-50 bg-black/70 backdrop-blur-sm"
    />
    <Dialog.Content
      onEscapeKeydown={(e) => isExtracting && e.preventDefault()}
      onInteractOutside={(e) => isExtracting && e.preventDefault()}
      class="border-border bg-foreground text-primary data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 fixed top-1/2 left-1/2 z-50 w-full max-w-lg -translate-1/2 rounded-xl border p-1.5 shadow-2xl"
    >
      <div class="flex items-start gap-4 px-6 py-5">
        <div
          class="bg-accent/10 border-accent/20 flex size-10 shrink-0 items-center justify-center rounded-lg border"
        >
          <FolderSync class="text-accent size-5" />
        </div>
        <div class="min-w-0 flex-1">
          <Dialog.Title class="text-xl font-semibold">
            {isExtracting ? "Updating folder…" : "Update folder from archive?"}
          </Dialog.Title>
          {#if isExtracting}
            <p class="text-muted-foreground mt-1.5 truncate text-sm" title={folderName}>
              {folderName}
            </p>
          {:else}
            <p class="text-muted-foreground mt-1.5 text-sm">
              <span class="text-primary font-medium">{zipName}</span> looks like it
              belongs to the folder
              <span class="text-primary font-medium">{folderName}</span>. Extract its
              contents into that folder?
            </p>
          {/if}
        </div>
      </div>

      {#if isExtracting}
        <ExtractProgressBar {progress} />
      {/if}

      <div class="border-border mt-3 flex items-center justify-end gap-2 border-t px-5 py-3">
        {#if isExtracting}
          <Button.Root
            onclick={onCancelExtract}
            class="border-border hover:bg-muted cursor-pointer rounded-md border bg-transparent px-4 py-1.5 text-xs font-medium transition-colors"
          >
            Cancel
          </Button.Root>
        {:else}
          <Button.Root
            onclick={onIgnore}
            class="border-border hover:bg-muted cursor-pointer rounded-md border bg-transparent px-4 py-1.5 text-xs font-medium transition-colors"
          >
            Ignore
          </Button.Root>
          <Button.Root
            onclick={onUpdate}
            class="bg-accent text-accent-foreground hover:bg-accent/90 cursor-pointer rounded px-4 py-1.5 text-xs font-semibold transition-all"
          >
            Update
          </Button.Root>
        {/if}
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
