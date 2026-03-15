<script lang="ts">
  import { createVirtualScroll } from "$lib/runes/virtualScroll.svelte";
  import type { ConflictingEntry, ConflictResolution } from "$lib/types";
  import { X, TriangleAlert } from "@lucide/svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { Button, Dialog, RadioGroup, ScrollArea } from "bits-ui";

  let {
    isOpen = $bindable(),
    onCancel,
    onResolve,
    conflictEntries,
  }: {
    isOpen: boolean;
    onCancel?: () => Promise<void>;
    onResolve?: () => Promise<void>;
    conflictEntries: ConflictingEntry[];
  } = $props();

  let actionTaken = $state(false);
  let conflictResolutions = $state<Map<string, ConflictResolution>>(new Map());

  const conflictCount = $derived(conflictEntries.length);
  const allResolved = $derived(
    conflictEntries.every((entry) => conflictResolutions.has(entry.src)),
  );

  const virtualScroll = createVirtualScroll<ConflictingEntry>({
    items: () => conflictEntries,
    itemHeight: 34,
    containerHeight: 624,
    overscan: 20,
  });

  function handleResolutionChange(src: string, value: ConflictResolution) {
    conflictResolutions.set(src, value);
    conflictResolutions = new Map(conflictResolutions);
  }

  function handleOpenChange(open: boolean) {
    if (!open && !actionTaken) {
      onCancel?.();
    }
  }

  function applyToAll(resolution: ConflictResolution) {
    conflictEntries.forEach((entry) => {
      conflictResolutions.set(entry.src, resolution);
    });

    conflictResolutions = new Map(conflictResolutions);
  }

  async function handleResolve() {
    await invoke("execute_operation", {
      conflictResolutions: Object.fromEntries(conflictResolutions),
    });

    actionTaken = true;
    isOpen = false;
    await onResolve?.();
  }
</script>

<Dialog.Root bind:open={isOpen} onOpenChange={handleOpenChange}>
  <Dialog.Portal>
    <Dialog.Overlay
      class="data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 fixed inset-0 z-50 bg-black/70 backdrop-blur-sm"
    />
    <Dialog.Content
      class="border-border bg-foreground text-primary data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 fixed top-1/2 left-1/2 z-50 w-full max-w-4xl -translate-1/2 rounded-xl border p-1.5 shadow-2xl"
    >
      <div class="flex items-start gap-4 px-6 py-5">
        <div
          class="bg-accent/10 border-accent/20 flex size-10 shrink-0 items-center justify-center rounded-lg border"
        >
          <TriangleAlert class="text-accent size-5" />
        </div>
        <div class="flex-1">
          <Dialog.Title class="text-xl font-semibold"
            >File Conflicts Detected</Dialog.Title
          >
          <p class="text-muted-foreground mt-1.5 text-sm">
            {conflictCount} File{conflictCount > 1 ? "s" : ""} Already Exist
          </p>
        </div>
        <Dialog.Close
          class="hover:bg-destructive hover:text-destructive-foreground focus-visible:ring-destructive -mt-1 -mr-1 cursor-pointer rounded-lg p-2 transition-colors focus-visible:ring-2 focus-visible:outline-none"
          aria-label="Close"
        >
          <X class="size-5" />
        </Dialog.Close>
      </div>
      <div
        class="bg-background border-border mb-3 flex flex-col rounded-md border"
      >
        <div class="border-border/60 border-b">
          <div
            class="grid grid-cols-[20%_30%_30%_20%] px-3 py-2 text-[11px] font-semibold tracking-wider text-muted-foreground uppercase"
          >
            <div>Name</div>
            <div>Source</div>
            <div>Destination</div>
            <div class="grid grid-cols-3 place-items-center gap-1">
              <span>Skip</span>
              <span>Keep</span>
              <span>Replace</span>
            </div>
          </div>
        </div>

        <ScrollArea.Root class="h-80">
          <ScrollArea.Viewport
            class="h-full w-full"
            onscroll={(e) =>
              (virtualScroll.scrollTop = e.currentTarget.scrollTop)}
          >
            <div class="relative py-1" style="height: {virtualScroll.totalHeight}px">
              <div
                class="flex flex-col"
                style="transform: translateY({virtualScroll.offsetY}px)"
              >
                {#each virtualScroll.visibleItems as item (item.src)}
                  <div
                    class="hover:bg-muted/50 mx-1 grid grid-cols-[20%_30%_30%_20%] rounded-md px-2 py-2 text-sm transition-colors"
                  >
                    <div class="min-w-0 pr-2">
                      <div class="truncate font-medium" title={item.name}>
                        {item.name}
                      </div>
                    </div>
                    <div class="min-w-0 pr-2">
                      <div class="truncate text-muted-foreground" title={item.src}>
                        {item.src}
                      </div>
                    </div>
                    <div class="min-w-0 pr-2">
                      <div class="truncate text-muted-foreground" title={item.dest}>
                        {item.dest}
                      </div>
                    </div>
                    <div>
                      <RadioGroup.Root
                        value={conflictResolutions.get(item.src) ?? ""}
                        onValueChange={(value) =>
                          handleResolutionChange(
                            item.src,
                            value as ConflictResolution,
                          )}
                        orientation="horizontal"
                        class="grid grid-cols-3 place-items-center gap-1"
                      >
                        <RadioGroup.Item
                          value="skip"
                          class="bg-background border-border hover:border-primary data-[state=checked]:border-accent size-4 shrink-0 cursor-default rounded-full border-2 transition-all data-[state=checked]:border-[5px]"
                        />
                        <RadioGroup.Item
                          value="keep"
                          class="bg-background border-border hover:border-primary data-[state=checked]:border-accent size-4 shrink-0 cursor-default rounded-full border-2 transition-all data-[state=checked]:border-[5px]"
                        />
                        <RadioGroup.Item
                          value="replace"
                          class="bg-background border-border hover:border-primary data-[state=checked]:border-accent size-4 shrink-0 cursor-default rounded-full border-2 transition-all data-[state=checked]:border-[5px]"
                        />
                      </RadioGroup.Root>
                    </div>
                  </div>
                {/each}
              </div>
            </div>
          </ScrollArea.Viewport>
          <ScrollArea.Scrollbar
            orientation="vertical"
            class="flex w-1.5 touch-none p-px transition-opacity"
          >
            <ScrollArea.Thumb
              class="bg-border/50 hover:bg-border relative flex-1 rounded-full transition-colors"
            />
          </ScrollArea.Scrollbar>
        </ScrollArea.Root>
      </div>

      <div
        class="border-border flex items-center justify-between gap-3 border-t px-5 py-3"
      >
        <div class="flex items-center gap-2">
          <span class="text-muted-foreground text-xs font-medium"
            >Apply to all:</span
          >
          <div class="flex gap-1.5">
            <Button.Root
              onclick={() => applyToAll("skip")}
              class="border-border hover:bg-muted cursor-pointer rounded-md border bg-transparent px-2.5 py-1 text-xs font-medium transition-colors"
              >Skip</Button.Root
            >
            <Button.Root
              onclick={() => applyToAll("keep")}
              class="border-border hover:bg-muted cursor-pointer rounded-md border bg-transparent px-2.5 py-1 text-xs font-medium transition-colors"
              >Keep</Button.Root
            >
            <Button.Root
              onclick={() => applyToAll("replace")}
              class="border-border hover:bg-muted cursor-pointer rounded-md border bg-transparent px-2.5 py-1 text-xs font-medium transition-colors"
              >Replace</Button.Root
            >
          </div>
        </div>
        <Button.Root
          disabled={!allResolved}
          onclick={handleResolve}
          class="bg-accent text-accent-foreground hover:bg-accent/90 rounded px-4 py-1.5 text-xs font-semibold transition-all disabled:pointer-events-none disabled:opacity-50"
          >Apply</Button.Root
        >
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
