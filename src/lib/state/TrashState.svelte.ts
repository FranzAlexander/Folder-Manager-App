import type { FileSystemEntry } from "$lib/types";
import { invoke } from "@tauri-apps/api/core";

export class TrashState {
  entries = $state<FileSystemEntry[]>([]);
  count = $state(0);

  constructor(private setEntries: (entries: FileSystemEntry[]) => void) {}

  async loadCount() {
    this.count = await invoke<number>("get_trash_count");
  }

  async load() {
    const entries = await invoke<FileSystemEntry[]>("get_trash_entries");
    this.entries = entries;
    this.count = entries.length;
    this.setEntries(entries);
  }

  async restore(iFilePath: string) {
    await invoke("restore_trash_entry", { iFilePath });
    await this.load();
  }

  async restoreAll() {
    await Promise.all(
      this.entries.map((e) => invoke("restore_trash_entry", { iFilePath: e.path })),
    );
    await this.load();
  }

  async moveToTrash(paths: string[]) {
    await invoke("move_to_trash", { paths });
  }

  async deletePermanently(iFilePath: string) {
    await invoke("delete_trash_entry", { iFilePath });
    await this.load();
  }

  async deleteAllPermanently() {
    await Promise.all(
      this.entries.map((e) => invoke("delete_trash_entry", { iFilePath: e.path })),
    );
    await this.load();
  }
}
