import type { FileSystemEntry } from "$lib/types";
import { SvelteSet } from "svelte/reactivity";

export class ClipboardState {
  entries = $state<FileSystemEntry[]>([]);
  operation = $state<"copy" | "cut" | null>(null);
  sourcePath = $state<string | null>(null);

  private clipboardPaths = $derived(
    new SvelteSet(this.entries.map((entry) => entry.path)),
  );

  get isEmpty(): boolean {
    return this.entries.length === 0;
  }

  get isCopy(): boolean {
    return this.operation === "copy";
  }

  get isCut(): boolean {
    return this.operation === "cut";
  }

  copy(entries: FileSystemEntry[], sourcePath: string) {
    this.entries = [...entries];
    this.operation = "copy";
    this.sourcePath = sourcePath;
  }

  cut(entries: FileSystemEntry[], sourcePath: string) {
    this.entries = [...entries];
    this.operation = "cut";
    this.sourcePath = sourcePath;
  }

  clear() {
    this.entries = [];
    this.operation = null;
    this.sourcePath = null;
  }

  isInClipboard(path: string): boolean {
    return this.clipboardPaths.has(path);
  }
}
