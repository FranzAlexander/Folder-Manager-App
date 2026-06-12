<script lang="ts">
  import type { TabsState } from "$lib/state/TabsState.svelte";
  import { X, Plus, Folder, Trash2 } from "@lucide/svelte";

  let { tabsState, onnew }: { tabsState: TabsState; onnew: () => void } =
    $props();
</script>

<div
  class="border-border bg-muted/20 flex items-end gap-0.5 overflow-x-auto border-b px-2 pt-1"
>
  {#each tabsState.tabs as tab, i (tab)}
    {@const isActive = i === tabsState.activeIndex}
    {@const isTrash = tab.currentDir === "trash://"}
    <button
      class="group relative flex h-8 min-w-0 max-w-48 shrink-0 cursor-pointer items-center gap-1.5 rounded-t-md px-3 text-sm transition-colors {isActive
        ? 'bg-background border-border border-x border-t -mb-px text-primary'
        : 'text-muted-foreground hover:bg-muted/60 hover:text-primary'}"
      onclick={() => tabsState.switchTo(i)}
      onauxclick={(e) => { if (e.button === 1) tabsState.closeTab(i); }}
    >
      {#if isTrash}
        <Trash2 class="size-3.5 shrink-0" />
      {:else}
        <Folder class="size-3.5 shrink-0" />
      {/if}
      <span class="min-w-0 truncate">{tabsState.tabTitle(tab)}</span>
      {#if tabsState.tabs.length > 1}
        <span
          role="button"
          tabindex="0"
          class="text-muted-foreground hover:text-primary ml-0.5 shrink-0 rounded p-0.5 opacity-0 transition-opacity group-hover:opacity-100 {isActive
            ? 'opacity-100'
            : ''}"
          onclick={(e) => {
            e.stopPropagation();
            tabsState.closeTab(i);
          }}
          onkeydown={(e) => {
            if (e.key === "Enter" || e.key === " ") {
              e.stopPropagation();
              tabsState.closeTab(i);
            }
          }}
        >
          <X class="size-3" />
        </span>
      {/if}
    </button>
  {/each}

  <button
    class="text-muted-foreground hover:text-primary hover:bg-muted/50 ml-1 flex h-7 w-7 shrink-0 cursor-pointer items-center justify-center rounded-md transition-colors"
    onclick={onnew}
    title="New tab (Ctrl+T)"
  >
    <Plus class="size-4" />
  </button>
</div>
