import type { FileSystemEntry } from "$lib/types";
import { SvelteSet } from "svelte/reactivity";

export class SelectionState {
  selectedPaths = $state(new SvelteSet<string>());
  anchorIndex = $state<number | null>(null);
  lastSelectedIndex = $state<number | null>(null);

  selectSingle(entry: FileSystemEntry, index: number) {
    this.selectedPaths.clear();
    this.selectedPaths.add(entry.path);

    this.anchorIndex = index;
    this.lastSelectedIndex = index;
  }

  toggleSelect(entry: FileSystemEntry, index: number) {
    if (this.selectedPaths.has(entry.path)) {
      this.selectedPaths.delete(entry.path);
    } else {
      this.selectedPaths.add(entry.path);
    }

    this.lastSelectedIndex = index;
  }

  selectRange(targetIndex: number, entries: FileSystemEntry[]) {
    const anchor = this.anchorIndex ?? 0;

    const startIndex = Math.min(anchor, targetIndex);
    const endIndex = Math.max(anchor, targetIndex);

    for (let i = startIndex; i <= endIndex; i++) {
      this.selectedPaths.add(entries[i].path);
    }

    this.lastSelectedIndex = targetIndex;
  }

  selectAll(entries: FileSystemEntry[]) {
    this.selectedPaths = new SvelteSet(entries.map((e) => e.path));
    this.anchorIndex = 0;
    this.lastSelectedIndex = entries.length - 1;
  }

  isSelected(path: string): boolean {
    return this.selectedPaths.has(path);
  }

  clearSelection() {
    this.selectedPaths.clear();
    this.anchorIndex = null;
    this.lastSelectedIndex = null;
  }
}
