import type { FileSystemEntry, SearchEvent } from "$lib/types";
import { Channel, invoke } from "@tauri-apps/api/core";

/**
 * Owns the in-directory search: the query text, debounce, and the streaming
 * `search_files` channel. Results and the surrounding directory are owned by
 * the explorer, so they're reached through injected callbacks.
 */
export class SearchState {
  query = $state("");
  isSearching = $state(false);

  private currentSearchId = 0;
  private searchTimeout: ReturnType<typeof setTimeout> | null = null;

  constructor(
    private getDir: () => string,
    private setEntries: (entries: FileSystemEntry[]) => void,
    private reloadDir: () => void,
  ) {}

  run = (query: string) => {
    this.query = query;

    if (this.searchTimeout !== null) {
      clearTimeout(this.searchTimeout);
    }

    if (query === "") {
      this.currentSearchId++;
      this.isSearching = false;
      this.reloadDir();
      return;
    }

    this.searchTimeout = setTimeout(() => {
      this.execute(query);
      this.searchTimeout = null;
    }, 500);
  };

  private async execute(name: string) {
    const searchId = ++this.currentSearchId;

    this.isSearching = true;
    const onEvent = new Channel<SearchEvent>();
    const newEntries: FileSystemEntry[] = [];

    onEvent.onmessage = (searchEvent) => {
      if (searchId !== this.currentSearchId) return;

      switch (searchEvent.event) {
        case "searching":
          newEntries.push(...searchEvent.data.entries);
          this.setEntries([...newEntries]);
          break;
        case "done":
          this.isSearching = false;
          break;
        case "notFound":
          this.setEntries([]);
          this.isSearching = false;
          break;
      }
    };

    await invoke("search_files", { path: this.getDir(), name, onEvent });
  }

  /** Cancels any in-flight/pending search without touching the query text. */
  cancel() {
    this.currentSearchId++;
    if (this.searchTimeout !== null) {
      clearTimeout(this.searchTimeout);
      this.searchTimeout = null;
    }
    this.isSearching = false;
  }

  /** Clears the query and cancels — used when navigating away. */
  reset() {
    this.query = "";
    this.cancel();
  }
}
