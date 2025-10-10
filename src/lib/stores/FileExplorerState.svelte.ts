import type { ColumnKey, FileSystemEntry, SearchEvent } from "$lib/types";
import { Channel, invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

export class FileExplorerState {
  rootDir = $state<string>("");
  showSetup = $state(false);
  history = $state<string[]>([]);
  historyIndex = $state<number>(0);
  selectedEntry = $state<FileSystemEntry | null>(null);
  entries = $state<FileSystemEntry[]>([]);
  sortedColumn = $state<ColumnKey>("name");
  sortedDirection = $state<"asc" | "desc">("desc");

  isSearching = $state(false);
  searchQuery = $state("");

  private currentSearchId = 0;
  private searchTimeout: ReturnType<typeof setTimeout> | null = null;

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
    await this.updateEntries(path);
  };

  selectDirectory = async () => {
    const selected = await open({
      multiple: false,
      directory: true,
      title: "Select Root Directory",
    });

    if (selected) {
      await invoke("set_root_directory", { path: selected });
      await this.setRootDir(selected);
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

    this.searchQuery = "";
    this.cancelSearch();

    this.updateEntries(path);
    this.selectedEntry = null;
  };

  goBack = async () => {
    if (this.historyIndex > 0) {
      this.historyIndex--;

      this.searchQuery = "";
      this.cancelSearch();

      this.updateEntries(this.currentDir);
    }
  };

  goForward = async () => {
    if (this.historyIndex < this.history.length - 1) {
      this.historyIndex++;

      this.searchQuery = "";
      this.cancelSearch();

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

  sortColumns = (columnKey: ColumnKey) => {
    if (this.sortedColumn === columnKey) {
      this.sortedDirection = this.sortedDirection === "asc" ? "desc" : "asc";
    } else {
      this.sortedColumn = columnKey;
      this.sortedDirection = "asc";
    }

    const direction = this.sortedDirection === "asc" ? 1 : -1;

    this.entries.sort((a, b) => {
      const aValue = a[columnKey as keyof FileSystemEntry];
      const bValue = b[columnKey as keyof FileSystemEntry];

      if (typeof aValue === "string" && typeof bValue === "string") {
        return aValue.localeCompare(bValue) * direction;
      }

      if (typeof aValue === "number" && typeof bValue === "number") {
        return (aValue - bValue) * direction;
      }

      return 0;
    });
  };

  search = (query: string) => {
    this.searchQuery = query;

    if (this.searchTimeout !== null) {
      clearTimeout(this.searchTimeout);
    }

    if (query === "") {
      this.currentSearchId++;
      this.isSearching = false;
      this.updateEntries(this.currentDir);
      return;
    }

    this.searchTimeout = setTimeout(() => {
      this.executeSearch(query);
      this.searchTimeout = null;
    }, 500);
  };

  private async executeSearch(name: string) {
    const searchId = ++this.currentSearchId;

    this.isSearching = true;
    const onEvent = new Channel<SearchEvent>();
    const newEntries: FileSystemEntry[] = [];

    onEvent.onmessage = (searchEvent) => {
      if (searchId !== this.currentSearchId) {
        return searchEvent.event;
      }

      switch (searchEvent.event) {
        case "searching":
          newEntries.push(...searchEvent.data.entries);
          this.entries = [...newEntries];
          break;
        case "done":
          this.isSearching = false;
          break;
        case "notFound":
          this.entries = [];
          this.isSearching = false;
          break;
      }

      return searchEvent.event;
    };

    if (name !== "") {
      await invoke("search_files", {
        path: this.currentDir,
        name,
        onEvent,
      });
    } else {
      if (searchId === this.currentSearchId) {
        this.isSearching = false;
        this.entries = [];
      }
    }
  }

  cancelSearch() {
    this.currentSearchId++;
    if (this.searchTimeout !== null) {
      clearTimeout(this.searchTimeout);
      this.searchTimeout = null;
    }
    this.isSearching = false;
  }
}
