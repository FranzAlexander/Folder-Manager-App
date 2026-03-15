<script lang="ts">
  import type { FileExplorerState } from "$lib/state/FileExplorerState.svelte";
  import ItemSelector from "../ItemSelector.svelte";

  let { fileExplorer }: { fileExplorer: FileExplorerState } = $props();
  const tags = $derived(fileExplorer.tags);
  const statuses = $derived(fileExplorer.statuses);

  async function handleCreateTag(tagName: string) {
    await tags.createTag(tagName);
  }

  async function handleAssignTag(tagId: number) {
    const tag = tags.resolveTag(tagId);
    if (tag) {
      await fileExplorer.assignTagToSelected(
        fileExplorer.selectedEntryPath,
        tag.name,
      );
    }
  }

  async function handleCreateStatus(statusName: string) {
    await statuses.createStatus(statusName);
  }

  async function handleAssignStatus(statusId: number) {
    await fileExplorer.assignStatusToSelected(
      fileExplorer.selectedEntryPath,
      statusId,
    );
  }
</script>

<div class="flex items-center gap-3">
  <ItemSelector
    items={tags.allTags}
    name={"Tags"}
    oncreate={handleCreateTag}
    action={handleAssignTag}
    disabled={!fileExplorer.selectedEntry}
  />

  <ItemSelector
    items={statuses.allStatuses}
    name={"Status"}
    oncreate={handleCreateStatus}
    action={handleAssignStatus}
    disabled={!fileExplorer.selectedEntry}
  />
</div>
