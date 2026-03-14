import { invoke } from "@tauri-apps/api/core";

export class TrashState {
  constructor(private setEntries: (entries: FileSystemEntry[]) => void) {}

  async load() {
    const entries = await invoke<FileSystemEntry[]>("get_trash_entries");
    this.setEntries(entries);
  }

  async restore(iFilePath: string) {
    await invoke("restore_trash_entry", { iFilePath });
    await this.load();
  }

  async deletePermanently(iFilePath: string) {
    await invoke("delete_trash_entry", { iFilePath });
    await this.load();
  }
}
