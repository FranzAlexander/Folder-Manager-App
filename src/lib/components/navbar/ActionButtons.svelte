<script lang="ts">
  import type { FileExplorerState } from "$lib/state/FileExplorerState.svelte";
  import ItemSelector from "../ItemSelector.svelte";

  let { fileExplorer }: { fileExplorer: FileExplorerState } = $props();
  const tags = $derived(fileExplorer.tags);
  const statuses = $derived(fileExplorer.statuses);
  const selectedEntry = $derived(fileExplorer.selectedEntry);
  const assignedTagIds = $derived(selectedEntry?.tagIds ?? []);
  const assignedStatusIds = $derived(selectedEntry?.statusIds ?? []);

  async function handleCreateTag(tagName: string) {
    await tags.createTag(tagName);
  }

  async function handleToggleTag(tagId: number) {
    const path = fileExplorer.selectedEntryPath;
    if (assignedTagIds.includes(tagId)) {
      await fileExplorer.unassignTagFromEntry(path, tagId);
    } else {
      await fileExplorer.assignTagToSelected(path, tagId);
    }
  }

  async function handleCreateStatus(statusName: string) {
    await statuses.createStatus(statusName);
  }

  async function handleToggleStatus(statusId: number) {
    const path = fileExplorer.selectedEntryPath;
    if (assignedStatusIds.includes(statusId)) {
      await fileExplorer.unassignStatusFromEntry(path, statusId);
    } else {
      await fileExplorer.assignStatusToSelected(path, statusId);
    }
  }
</script>

<div class="flex items-center gap-3">
  <ItemSelector
    items={tags.allTags}
    name={"Tags"}
    assignedIds={assignedTagIds}
    oncreate={handleCreateTag}
    action={handleToggleTag}
    disabled={!selectedEntry}
  />

  <ItemSelector
    items={statuses.allStatuses}
    name={"Status"}
    assignedIds={assignedStatusIds}
    oncreate={handleCreateStatus}
    action={handleToggleStatus}
    disabled={!selectedEntry}
  />
</div>
