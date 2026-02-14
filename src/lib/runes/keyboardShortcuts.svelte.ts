import type { FileExplorerState } from "$lib/state/FileExplorerState.svelte";

export function createKeyboardShortcuts(
  getFileExplorer: () => FileExplorerState,
) {
  function handleKeydown(event: KeyboardEvent) {
    const fileExplorer = getFileExplorer();
    const ctrl = event.ctrlKey || event.metaKey;

    if (ctrl && event.key === "a") {
      event.preventDefault();
      fileExplorer.selection.selectAll(fileExplorer.entries);
      return;
    }

    if (ctrl && event.key === "c") {
      event.preventDefault();

      const selectedEntries = fileExplorer.selectedEntries;

      if (selectedEntries.length === 0) {
        return;
      }

      fileExplorer.clipboard.copy(selectedEntries, fileExplorer.currentDir);
      return;
    }

    if (ctrl && event.key === "x") {
      event.preventDefault();

      const selectedEntries = fileExplorer.selectedEntries;

      if (selectedEntries.length === 0) {
        return;
      }

      fileExplorer.clipboard.cut(selectedEntries, fileExplorer.currentDir);
      return;
    }

    if (ctrl && event.key === "v") {
      event.preventDefault();
      fileExplorer.paste();
      return;
    }

    if (event.key === "Escape") {
      fileExplorer.clipboard.clear();
      return;
    }
  }
  return { handleKeydown };
}
