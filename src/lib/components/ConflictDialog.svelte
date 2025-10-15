<script lang="ts">
  import type { ConflictingEntries } from "$lib/types";
  import { X } from "@lucide/svelte";
  import { Button, Checkbox, Dialog, RadioGroup, ScrollArea } from "bits-ui";

  let {
    isOpen = $bindable(),
    onCancel,
    conflictEntries,
  }: {
    isOpen: boolean;
    onCancel?: () => Promise<void>;
    conflictEntries: ConflictingEntries[];
  } = $props();

  let applyToAll = $state(false);
  let actionTaken = $state(false);

  function handleOpenChange(open: boolean) {
    if (!open && !actionTaken) {
      onCancel?.();
    }
  }

  const conflictCount = $derived(conflictEntries.length);
</script>

<Dialog.Root bind:open={isOpen} onOpenChange={handleOpenChange}>
  <Dialog.Portal>
    <Dialog.Overlay class="fixed inset-0 z-50 bg-black/50" />
    <Dialog.Content
      class="border-border bg-foreground text-primary fixed top-1/2 left-1/2 z-50 w-full max-w-2xl -translate-x-1/2 -translate-y-1/2 rounded-lg border p-6 shadow-lg"
    >
      <Dialog.Title class="mb-2 text-xl font-semibold">
        {conflictCount} File{conflictCount > 1 ? "s" : ""} Already Exist
      </Dialog.Title>
      <ScrollArea.Root class="border-border bg-background mb-4 h-96 rounded-md">
        <ScrollArea.Viewport class="h-full p-1">
          <div class="bg-foreground border-border sticky top-0 z-10 border-b">
            <div
              class="flex items-center gap-4 px-6 py-3 text-sm font-semibold"
            >
              <div class="w-24 flex-shrink-0">Name</div>
              <div class="w-24 min-w-0">Source</div>
              <div class="w-24 min-w-0">Destination</div>
              <div class="flex">
                <div class="w-16 flex-shrink-0 text-center">Skip</div>
                <div class="w-16 flex-shrink-0 text-center">Keep</div>
                <div class="w-20 flex-shrink-0 text-center">Replace</div>
              </div>
            </div>
          </div>
          <div class="space-y-3">
            {#each conflictEntries as conflict}
              <div
                class="border-border bg-foreground hover:bg-muted flex items-center gap-4 border-b py-3 transition-colors"
              >
                <div class="w-24 flex-shrink-0 truncate text-sm font-medium">
                  {conflict.name}
                </div>
                <div class="w-24 min-w-0 truncate text-xs" title={conflict.src}>
                  {conflict.src}
                </div>
                <div
                  class="w-24 min-w-0 truncate text-xs"
                  title={conflict.dest}
                >
                  {conflict.dest}
                </div>
                <div>
                  <RadioGroup.Root class="flex">
                    <div>
                      <RadioGroup.Item
                        id="skip"
                        value="skip"
                        class="border-border-input bg-background hover:border-dark-40 data-[state=checked]:border-accent size-5 shrink-0 cursor-default rounded-full border transition-all duration-100 ease-in-out data-[state=checked]:border-6"
                      />
                    </div>
                    <div>
                      <RadioGroup.Item
                        id="keep"
                        value="keep"
                        class="border-border-input bg-background hover:border-dark-40 data-[state=checked]:border-foreground size-5 shrink-0 cursor-default rounded-full border transition-all duration-100 ease-in-out data-[state=checked]:border-6"
                      />
                    </div>
                    <div>
                      <RadioGroup.Item
                        id="replace"
                        value="replace"
                        class="border-border-input bg-background hover:border-dark-40 data-[state=checked]:border-foreground size-5 shrink-0 cursor-default rounded-full border transition-all duration-100 ease-in-out data-[state=checked]:border-6"
                      />
                    </div>
                  </RadioGroup.Root>
                </div>
              </div>
            {/each}
          </div>
        </ScrollArea.Viewport>
      </ScrollArea.Root>

      <div class="flex gap-3">
        <Button.Root
          class="border-border hover:bg-accent cursor-pointer rounded-md border px-4 py-2 text-sm font-medium transition-colors"
          >Skip</Button.Root
        >
        <Button.Root
          class="border-border hover:bg-accent cursor-pointer rounded-md border px-4 py-2 text-sm font-medium transition-colors"
          >Keep Both</Button.Root
        >
        <Button.Root
          class="border-border hover:bg-accent cursor-pointer rounded-md border px-4 py-2 text-sm font-medium transition-colors"
          >Replace</Button.Root
        >
      </div>
      <Dialog.Close
        class="absolute top-5 right-5 cursor-pointer rounded-lg p-2 hover:bg-red-500"
      >
        <div>
          <X class="size-5" />
          <span class="sr-only">Close</span>
        </div>
      </Dialog.Close>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
