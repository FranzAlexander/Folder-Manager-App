import type { FileSystemEntry } from "$lib/types";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

export class FileExplorerState {
  rootDir = $state<string>("");
  showSetup = $state(false);
  history = $state<string[]>([]);
  historyIndex = $state<number>(0);
  selectedEntry = $state<FileSystemEntry | null>(null);
  entries = $state<FileSystemEntry[]>([]);

  get currentDir() {
    return this.history[this.historyIndex] || "";
  }

  get selectedEntryPath() {
    return this.selectedEntry?.path;
  }

  updateEntries = async (path: string) => {
    this.entries = await invoke("read_directory", { path });
  };

  setRootDir = async (path: string) => {
    this.rootDir = path;
    this.history = [path];
    this.historyIndex = 0;
    this.updateEntries(path);
  };

  selectDirectory = async () => {
    const selected = await open({
      multiple: false,
      directory: true,
      title: "Select Root Directory",
    });

    if (selected) {
      await invoke("set_root_directory", { path: selected });
      this.rootDir = selected;
      this.history.push(this.rootDir);
      this.showSetup = false;
    }
  };

  openEntry = (entry: FileSystemEntry) => {
    if (entry.isDir) {
      this.navigateToDirectory(entry.path);
    }

    if (entry.isFile && entry.fileType === "EXE") {
      this.startExecutable(entry.path);
    }
  };

  navigateToDirectory = async (path: string) => {
    if (this.historyIndex < this.history.length - 1) {
      this.history = this.history.slice(0, this.historyIndex + 1);
    }

    this.history.push(path);
    this.historyIndex = this.history.length - 1;

    this.updateEntries(path);
    this.selectedEntry = null;
  };

  goBack = async () => {
    if (this.historyIndex > 0) {
      this.historyIndex--;
      this.updateEntries(this.currentDir);
    }
  };

  goForward = async () => {
    if (this.historyIndex < this.history.length - 1) {
      this.historyIndex++;
      this.updateEntries(this.currentDir);
    }
  };

  selectEntry = (entry: FileSystemEntry) => {
    this.selectedEntry = entry;
  };

  handleMouseButton = (event: MouseEvent) => {
    if (event.button === 3) this.goBack();
    if (event.button === 4) this.goForward();
  };

  startExecutable = async (path: string) => {
    await invoke("start_executable", { path });
  };
}
