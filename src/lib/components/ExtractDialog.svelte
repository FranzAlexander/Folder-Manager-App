<script lang="ts">
  import type { ExtractProgress } from "$lib/types";
  import { untrack } from "svelte";
  import ExtractProgressBar from "./ExtractProgressBar.svelte";
  import { X, FileArchive, Folder } from "@lucide/svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { Button, Dialog } from "bits-ui";

  let {
    archivePath,
    isExtracting,
    progress,
    onCancel,
    onCancelExtract,
    onConfirm,
  }: {
    archivePath: string;
    isExtracting: boolean;
    progress: ExtractProgress | null;
    onCancel: () => void;
    onCancelExtract: () => void;
    onConfirm: (dest: string) => void;
  } = $props();

  // Split a Windows/Unix path into [parent, fileName] on the last separator.
  function splitPath(p: string): [string, string] {
    const idx = Math.max(p.lastIndexOf("\\"), p.lastIndexOf("/"));
    return idx === -1 ? ["", p] : [p.slice(0, idx), p.slice(idx + 1)];
  }

  const sep = $derived(archivePath.includes("\\") ? "\\" : "/");
  const [parentDir, fileName] = $derived.by(() => splitPath(archivePath));
  // Drop the extension for the default folder name (photos.zip -> photos).
  const stem = $derived(fileName.replace(/\.[^.]+$/, ""));

  // Editable destination, defaulted Windows-Explorer style: a subfolder named
  // after the archive, next to the archive. Snapshot the default once (untrack)
  // so later renders never clobber what the user types.
  let dest = $state(untrack(() => `${parentDir}${sep}${stem}`));

  let isOpen = $state(true);

  async function handleBrowse() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Select destination folder",
    });
    if (typeof selected === "string") {
      dest = selected;
    }
  }

  function handleExtract() {
    const trimmed = dest.trim();
    if (!trimmed) return;
    onConfirm(trimmed);
  }

  function handleOpenChange(open: boolean) {
    // Parent unmounts us when extraction finishes; only a real user dismiss
    // (Esc/overlay/X, blocked while extracting) should cancel.
    if (!open && !isExtracting) onCancel();
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
          <FileArchive class="text-accent size-5" />
        </div>
        <div class="flex-1">
          <Dialog.Title class="text-xl font-semibold">
            {isExtracting ? "Extracting…" : "Extract Archive"}
          </Dialog.Title>
          <p class="text-muted-foreground mt-1.5 truncate text-sm" title={fileName}>
            {fileName}
          </p>
        </div>
        {#if !isExtracting}
          <Dialog.Close
            class="hover:bg-destructive hover:text-destructive-foreground focus-visible:ring-destructive -mt-1 -mr-1 cursor-pointer rounded-lg p-2 transition-colors focus-visible:ring-2 focus-visible:outline-none"
            aria-label="Close"
          >
            <X class="size-5" />
          </Dialog.Close>
        {/if}
      </div>

      {#if isExtracting}
        <ExtractProgressBar {progress} />
      {:else}
        <div class="px-6 pb-2">
          <span class="text-muted-foreground text-xs font-medium">
            Files will be extracted to this folder:
          </span>
          <div class="mt-2 flex gap-2">
            <input
              type="text"
              bind:value={dest}
              spellcheck="false"
              onkeydown={(e) => {
                if (e.key === "Enter") handleExtract();
              }}
              class="bg-background border-border focus:border-accent min-w-0 flex-1 rounded-md border px-3 py-2 text-sm outline-none"
            />
            <Button.Root
              onclick={handleBrowse}
              class="border-border hover:bg-muted flex shrink-0 cursor-pointer items-center gap-1.5 rounded-md border bg-transparent px-3 py-2 text-sm font-medium transition-colors"
            >
              <Folder class="size-3.5" />
              Browse
            </Button.Root>
          </div>
        </div>
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
            onclick={onCancel}
            class="border-border hover:bg-muted cursor-pointer rounded-md border bg-transparent px-4 py-1.5 text-xs font-medium transition-colors"
          >
            Cancel
          </Button.Root>
          <Button.Root
            disabled={!dest.trim()}
            onclick={handleExtract}
            class="bg-accent text-accent-foreground hover:bg-accent/90 cursor-pointer rounded px-4 py-1.5 text-xs font-semibold transition-all disabled:pointer-events-none disabled:opacity-50"
          >
            Extract
          </Button.Root>
        {/if}
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
