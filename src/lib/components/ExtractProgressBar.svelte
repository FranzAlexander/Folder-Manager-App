<script lang="ts">
  import type { ExtractProgress } from "$lib/types";
  import { formatFileSize } from "$lib/utils/formatters";

  let { progress }: { progress: ExtractProgress | null } = $props();

  // total is unknown (0) until the first progress event arrives.
  const percent = $derived(
    progress && progress.total > 0
      ? Math.min(100, Math.round((progress.current / progress.total) * 100))
      : 0,
  );
</script>

<div class="px-6 pb-2">
  <div class="bg-background border-border h-2 w-full overflow-hidden rounded-full border">
    <div
      class="bg-accent h-full rounded-full transition-[width] duration-150"
      style="width: {percent}%"
    ></div>
  </div>
  <p class="text-muted-foreground mt-2 text-xs">
    {#if progress && progress.total > 0}
      {formatFileSize(progress.current)} of {formatFileSize(progress.total)}
      ({percent}%)
    {:else}
      Preparing…
    {/if}
  </p>
</div>
