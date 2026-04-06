import type { FileExplorerState } from "$lib/state/FileExplorerState.svelte";
import { confirm } from "@tauri-apps/plugin-dialog";

export function createKeyboardShortcuts(
  getFileExplorer: () => FileExplorerState,
  onPaste: () => void,
) {
  async function handleKeydown(event: KeyboardEvent) {
    const fileExplorer = getFileExplorer();
    const ctrl = event.ctrlKey || event.metaKey;
    const isTrash = fileExplorer.currentDir === "trash://";

    if (ctrl && event.key === "a") {
      event.preventDefault();
      fileExplorer.selection.selectAll(fileExplorer.entries);
      return;
    }

    if (ctrl && event.key === "c") {
      event.preventDefault();
      const selectedEntries = fileExplorer.selectedEntries;
      if (selectedEntries.length === 0) return;
      fileExplorer.clipboard.copy(selectedEntries, fileExplorer.currentDir);
      return;
    }

    if (ctrl && event.key === "x") {
      event.preventDefault();
      const selectedEntries = fileExplorer.selectedEntries;
      if (selectedEntries.length === 0) return;
      fileExplorer.clipboard.cut(selectedEntries, fileExplorer.currentDir);
      return;
    }

    if (ctrl && event.key === "v") {
      event.preventDefault();
      onPaste();
      return;
    }

    if (event.key === "Delete") {
      event.preventDefault();
      if (fileExplorer.selectedEntries.length === 0) return;

      if (isTrash) {
        // In trash: Del = permanently delete selected
        const ok = await confirm(
          `Permanently delete ${fileExplorer.selectedEntries.length} item(s)? This cannot be undone.`,
          { title: "Delete Permanently", kind: "warning" },
        );
        if (ok) await fileExplorer.deleteSelectedPermanently();
      } else if (event.shiftKey) {
        // Normal dir + Shift+Delete = permanently delete (bypass trash)
        const ok = await confirm(
          `Permanently delete ${fileExplorer.selectedEntries.length} item(s)? This cannot be undone.`,
          { title: "Delete Permanently", kind: "warning" },
        );
        if (ok) await fileExplorer.deletePermanentlySelected();
      } else {
        // Normal dir + Del = move to trash
        await fileExplorer.moveSelectedToTrash();
      }
      return;
    }

    if (event.key === "Enter") {
      event.preventDefault();
      const selected = fileExplorer.selectedEntry;
      if (selected && !isTrash) fileExplorer.openEntry(selected);
      return;
    }

    if (event.key === "Escape") {
      fileExplorer.clipboard.clear();
      return;
    }
  }

  return { handleKeydown };
}
