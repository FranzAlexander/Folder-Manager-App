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
    action: (id: number) => void;
    disabled?: boolean;
  } = $props();

  //   type Item = {
  //     content: string;
  //     keywords: string[];
  //     disabled?: boolean;
  //     action?: () => void;
  //   };

  //   type Group = {
  //     name: string;
  //     items: items;
  //   };

  //   type View = {
  //     columns: number | undefined;
  //     empty: string;
  //     placeholder: string;
  //     groups: Group[];
  //   };

  //   const view: View;
  let search = $state("");

  async function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" && search.length > 0) {
      event.preventDefault();
      await oncreate(search);
      search = "";
    }
  }
</script>

<Popover.Root>
  <Popover.Trigger
    class="border-border border py-2 px-4 rounded-lg hover:bg-accent inline-flex hover:cursor-pointer"
  >
    {name}
  </Popover.Trigger>
  <Popover.Portal>
    <Popover.Content class="z-50 w-90" sideOffset={4}>
      <Command.Root
        columns={6}
        class="flex h-full w-full border-border border bg-foreground  rounded-lg flex-col text-primary"
      >
        <div class="flex items-center gap-2 border-b border-border pl-3 pr-8">
          <Search class="size-6" />
          <Command.Input
            bind:value={search}
            onkeydown={handleKeydown}
            class="outline-hidden inline-flex h-10 w-full rounded-md py-3 bg-transparent text-sm disabled:cursor-not-allowed disabled:opacity-50"
          />
        </div>
        <Command.List>
          <Command.Viewport>
            <Command.Empty>Press Enter to create new {name}.</Command.Empty>
            <Command.Group>
              <Command.GroupHeading>Current {name}</Command.GroupHeading>
              <Command.GroupItems class="grid grid-cols-6 gap-2">
                {#each items as item (item)}
                  <Command.Item {disabled} onSelect={() => action(item.id)}>
                    <div
                      class="bg-muted text-primary border-border rounded-md border px-2 py-0.5 text-sm font-medium items-center inline-flex w-fit shrink-0 justify-center"
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
