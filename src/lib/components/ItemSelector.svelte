<script lang="ts">
  import type { Tag } from "$lib/types";
  import { Search } from "@lucide/svelte";
  import { Command, Popover } from "bits-ui";

  let {
    items,
    name,
    oncreate,
    action,
    disabled,
  }: {
    items: Tag[];
    name: string;
    oncreate: (search: string) => Promise<void>;
    action: (id: number) => Promise<void>;
    disabled?: boolean;
  } = $props();

  let search = $state("");

  async function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" && search.length > 0) {
      event.preventDefault();
      await oncreate(search);
      search = "";
    }
  }

  async function onaction(itemId: number) {
    await action(itemId);
  }
</script>

<Popover.Root>
  <Popover.Trigger
    class="border-border hover:bg-muted/60 text-muted-foreground hover:text-primary cursor-pointer rounded-md border px-3 py-1.5 text-sm font-medium transition-colors"
  >
    {name}
  </Popover.Trigger>
  <Popover.Portal>
    <Popover.Content class="z-50 w-72" sideOffset={6}>
      <Command.Root
        class="border-border bg-popover text-primary overflow-hidden rounded-xl border shadow-2xl"
      >
        <div class="border-border/60 flex items-center gap-2.5 border-b px-3">
          <Search class="text-muted-foreground size-4 shrink-0" />
          <Command.Input
            bind:value={search}
            onkeydown={handleKeydown}
            placeholder="Search or create..."
            class="placeholder:text-muted-foreground w-full bg-transparent py-3 text-sm outline-none disabled:cursor-not-allowed disabled:opacity-50"
          />
        </div>

        <Command.List class="overflow-y-auto">
          <Command.Viewport class="p-2">
            <Command.Empty class="text-muted-foreground py-8 text-center text-sm">
              No {name.toLowerCase()} found. Press Enter to create.
            </Command.Empty>

            <Command.Group>
              <Command.GroupHeading
                class="text-muted-foreground mb-1.5 px-1 text-[11px] font-semibold tracking-wider uppercase"
              >
                Current {name}
              </Command.GroupHeading>
              <Command.GroupItems class="flex flex-wrap gap-1.5 px-1">
                {#each items as item (item)}
                  <Command.Item {disabled} onSelect={() => onaction(item.id)}>
                    <div
                      class="bg-accent/10 text-accent hover:bg-accent hover:text-accent-foreground cursor-pointer rounded-full px-3 py-1 text-xs font-medium transition-colors"
                      title={item.name}
                    >
                      {item.name}
                    </div>
                  </Command.Item>
                {/each}
              </Command.GroupItems>
            </Command.Group>
          </Command.Viewport>
        </Command.List>
      </Command.Root>
    </Popover.Content>
  </Popover.Portal>
</Popover.Root>
