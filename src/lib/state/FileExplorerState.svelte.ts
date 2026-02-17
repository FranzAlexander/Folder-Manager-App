import type {
  ColumnKey,
  ConflictingEntry,
  FileSystemEntry,
  SearchEvent,
} from "$lib/types";
import { Channel, invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { SelectionState } from "./SelectionState.svelte";
import { statusManager } from "$lib/state/StatusManager.svelte";
import { ClipboardState } from "./ClipboardState.svelte";
import { tagManager } from "./TagManager.svelte";
import type { newMenu } from "@tauri-apps/api/menu/base";

export class FileExplorerState {
  rootDir = $state<string>("");
  showSetup = $state(false);
  history = $state<string[]>([]);
  historyIndex = $state<number>(0);
  entries = $state<FileSystemEntry[]>([]);
  sortedColumn = $state<ColumnKey>("name");
  sortedDirection = $state<"asc" | "desc">("desc");

  selection = new SelectionState();
  clipboard = new ClipboardState();

  isSearching = $state(false);
  searchQuery = $state("");

  readonly tags = tagManager;
  readonly statuses = statusManager;

  private currentSearchId = 0;
  private searchTimeout: ReturnType<typeof setTimeout> | null = null;

  get currentDir() {
    return this.history[this.historyIndex] || "";
  }

  get selectedEntry(): FileSystemEntry | null {
    if (this.selection.lastSelectedIndex === null) {
      return null;
    }

    if (this.selection.lastSelectedIndex >= this.entries.length) {
      return null;
    }

    return this.entries[this.selection.lastSelectedIndex];
  }

  get selectedEntryPath(): string | undefined {
    return this.selectedEntry?.path;
  }

  get selectedEntries(): FileSystemEntry[] {
    return this.entries.filter((entry) =>
      this.selection.isSelected(entry.path),
    );
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
    this.selection.clearSelection();

    if (path == "trash://") {
      await this.loadTrash();
    } else {
      await this.updateEntries(path);
    }
  };

  goBack = async () => {
    if (this.historyIndex > 0) {
      this.historyIndex--;

      this.searchQuery = "";
      this.cancelSearch();

      this.selection.clearSelection();

      this.updateEntries(this.currentDir);
    }
  };

  goForward = async () => {
    if (this.historyIndex < this.history.length - 1) {
      this.historyIndex++;

      this.searchQuery = "";
      this.cancelSearch();

      this.selection.clearSelection();

      this.updateEntries(this.currentDir);
    }
  };

  handleEntryClick(entry: FileSystemEntry, index: number, event: MouseEvent) {
    if (event.shiftKey) {
      this.selection.selectRange(index, this.entries);
      return;
    }

    if (event.ctrlKey || event.metaKey) {
      this.selection.toggleSelect(entry, index);
      return;
    }

    this.selection.selectSingle(entry, index);
  }

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

  async paste(): Promise<ConflictingEntry[] | void> {
    if (this.clipboard.isEmpty) return;

    const sourceEntries = this.clipboard.entries.map((entry) => ({
      path: entry.path,
      isDir: entry.isDir,
    }));

    const destPath = this.currentDir;
    const operationType = this.clipboard.isCopy ? "copy" : "move";

    const conflicts: ConflictingEntry[] = await invoke("prepare_operation", {
      srcEntries: sourceEntries,
      dest: destPath,
      operationType,
    });

    if (conflicts.length !== 0) {
      return conflicts;
    }

    await invoke("execute_operation", {
      conflictResolutions: {},
    });

    if (this.clipboard.isCut) {
      this.clipboard.clear();
    }

    await this.updateEntries(destPath);
  }

  async assignTagToSelected(entryPath: string | undefined, tagName: string) {
    if (!entryPath) return;
    const tag = await this.tags.assignTag(entryPath, tagName);

    const entry = this.entries.find((e) => e.path === entryPath);
    if (entry && !entry.tagIds.includes(tag.id)) {
      entry.tagIds = [...entry.tagIds, tag.id];
    }
  }

  async assignStatusToSelected(
    entryPath: string | undefined,
    statusId: number,
  ) {
    if (!entryPath) return;
    const status = await this.statuses.setStatus(entryPath, statusId);

    const entry = this.entries.find((e) => e.path === entryPath);
    if (entry && !entry.statusIds.includes(status.id)) {
      entry.statusIds = [...entry.statusIds, status.id];
    }
  }

  async loadTrash() {
    this.entries = await invoke<FileSystemEntry[]>("get_trash_entries");
  }
}
