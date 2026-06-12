import type {
  ColumnKey,
  ConflictingEntry,
  FileSystemEntry,
  SearchEvent,
} from "$lib/types";
import { Channel, invoke } from "@tauri-apps/api/core";
import { message, open } from "@tauri-apps/plugin-dialog";
import { SelectionState } from "./SelectionState.svelte";
import { statusManager } from "$lib/state/StatusManager.svelte";
import { ClipboardState } from "./ClipboardState.svelte";
import { tagManager } from "./TagManager.svelte";
import { TrashState } from "./TrashState.svelte";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

function sortEntries(
  entries: FileSystemEntry[],
  column: ColumnKey,
  direction: "asc" | "desc",
): FileSystemEntry[] {
  const dir = direction === "asc" ? 1 : -1;
  return [...entries].sort((a, b) => {
    const aValue = a[column as keyof FileSystemEntry];
    const bValue = b[column as keyof FileSystemEntry];
    if (aValue == null && bValue == null) return 0;
    if (aValue == null) return 1;
    if (bValue == null) return -1;
    if (typeof aValue === "string" && typeof bValue === "string")
      return aValue.localeCompare(bValue) * dir;
    if (typeof aValue === "number" && typeof bValue === "number")
      return (aValue - bValue) * dir;
    return 0;
  });
}

export class FileExplorerState {
  rootDir = $state<string>("");
  showSetup = $state(false);
  history = $state<string[]>([]);
  historyIndex = $state<number>(0);
  private _dirEntries = $state<FileSystemEntry[]>([]);
  sortedColumn = $state<ColumnKey>("name");
  sortedDirection = $state<"asc" | "desc">("desc");

  renamingPath = $state<string | null>(null);
  renameValue = $state("");
  pendingScrollToIndex = $state<number | null>(null);

  private dirChangeUnlisten: UnlistenFn | null = null;
  private refreshTimeout: ReturnType<typeof setTimeout> | null = null;

  selection = new SelectionState();
  clipboard = new ClipboardState();
  trash = new TrashState();
  readonly entries = $derived.by(() => {
    const base =
      this.currentDir === "trash://" ? this.trash.entries : this._dirEntries;
    return sortEntries(base, this.sortedColumn, this.sortedDirection);
  });

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

  createFolder = async () => {
    const newPath: string = await invoke("create_folder", {
      parent: this.currentDir,
      name: "New Folder",
    });
    await this.updateEntries(this.currentDir);
    const entry = this.entries.find((e) => e.path === newPath);
    if (entry) {
      const index = this.entries.indexOf(entry);
      this.selection.selectSingle(entry, index);
      this.pendingScrollToIndex = index;
      this.startRename(entry);
    }
  };

  startRename = (entry: FileSystemEntry) => {
    this.renamingPath = entry.path;
    this.renameValue = entry.name;
  };

  commitRename = async () => {
    if (!this.renamingPath) return;
    const oldPath = this.renamingPath;
    const newName = this.renameValue.trim();
    this.renamingPath = null;
    this.renameValue = "";
    if (!newName) return;
    const entry = this.entries.find((e) => e.path === oldPath);
    if (entry?.name === newName) return;
    try {
      await invoke("rename_entry", { oldPath, newName });
      await this.updateEntries(this.currentDir);
    } catch (e) {
      const msg =
        typeof e === "object" && e !== null && "message" in e
          ? String((e as { message: unknown }).message)
          : String(e);
      await message(msg, { title: "Rename failed", kind: "error" });
    }
  };

  cancelRename = () => {
    this.renamingPath = null;
    this.renameValue = "";
  };

  updateEntries = async (path: string) => {
    this._dirEntries = await invoke("read_directory", { path });
  };

  setRootDir = async (path: string) => {
    this.rootDir = path;
    this.history = [path];
    this.historyIndex = 0;
    await invoke("watch_directory", { path });
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
      return;
    }

    if (entry.isFile) {
      this.openFile(entry.path);
    }
  };

  async startWatching() {
    this.dirChangeUnlisten = await listen<string>("dir-changed", (event) => {
      if (event.payload !== this.currentDir) return;
      if (this.isSearching) return;
      if (this.refreshTimeout !== null) clearTimeout(this.refreshTimeout);
      this.refreshTimeout = setTimeout(async () => {
        await this.updateEntries(this.currentDir);
        this.refreshTimeout = null;
      }, 300);
    });
  }

  stopWatching() {
    this.dirChangeUnlisten?.();
    this.dirChangeUnlisten = null;
    invoke("unwatch_directory");
  }

  navigateToDirectory = async (path: string) => {
    if (this.historyIndex < this.history.length - 1) {
      this.history = this.history.slice(0, this.historyIndex + 1);
    }

    this.history.push(path);
    this.historyIndex = this.history.length - 1;

    this.searchQuery = "";
    this.cancelSearch();
    this.selection.clearSelection();

    if (path === "trash://") {
      await invoke("unwatch_directory");
      await this.trash.load();
    } else {
      await invoke("watch_directory", { path });
      await this.updateEntries(path);
    }
  };

  goBack = async () => {
    if (this.historyIndex > 0) {
      this.historyIndex--;

      this.searchQuery = "";
      this.cancelSearch();
      this.selection.clearSelection();

      if (this.currentDir === "trash://") {
        await invoke("unwatch_directory");
        await this.trash.load();
      } else {
        await invoke("watch_directory", { path: this.currentDir });
        await this.updateEntries(this.currentDir);
      }
    }
  };

  goForward = async () => {
    if (this.historyIndex < this.history.length - 1) {
      this.historyIndex++;

      this.searchQuery = "";
      this.cancelSearch();
      this.selection.clearSelection();

      if (this.currentDir === "trash://") {
        await invoke("unwatch_directory");
        await this.trash.load();
      } else {
        await invoke("watch_directory", { path: this.currentDir });
        await this.updateEntries(this.currentDir);
      }
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

  openFile = async (path: string) => {
    try {
      await invoke("open_file", { path });
    } catch (e) {
      await message(String(e), { title: "Failed to open file", kind: "error" });
    }
  };

  startExecutable = async (path: string) => {
    try {
      await invoke("start_executable", { path });
    } catch (e) {
      await message(String(e), {
        title: "Failed to launch executable",
        kind: "error",
      });
    }
  };

  sortColumns = (columnKey: ColumnKey) => {
    if (this.sortedColumn === columnKey) {
      this.sortedDirection = this.sortedDirection === "asc" ? "desc" : "asc";
    } else {
      this.sortedColumn = columnKey;
      this.sortedDirection = "asc";
    }
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
      if (searchId !== this.currentSearchId) return;

      switch (searchEvent.event) {
        case "searching":
          newEntries.push(...searchEvent.data.entries);
          this._dirEntries = [...newEntries];
          break;
        case "done":
          this.isSearching = false;
          break;
        case "notFound":
          this._dirEntries = [];
          this.isSearching = false;
          break;
      }
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
        this._dirEntries = [];
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

  async moveSelectedToTrash() {
    const paths = this.selectedEntries.map((e) => e.path);
    if (paths.length === 0) return;
    await this.trash.moveToTrash(paths);
    this.selection.clearSelection();
    await this.updateEntries(this.currentDir);
  }

  async deletePermanentlySelected() {
    const paths = this.selectedEntries.map((e) => e.path);
    if (paths.length === 0) return;
    await invoke("delete_permanently", { paths });
    this.selection.clearSelection();
    await this.updateEntries(this.currentDir);
  }

  async restoreSelected() {
    await Promise.all(
      this.selectedEntries.map((e) =>
        invoke("restore_trash_entry", { iFilePath: e.path }),
      ),
    );
    this.selection.clearSelection();
    await this.trash.load();
  }

  async deleteSelectedPermanently() {
    await Promise.all(
      this.selectedEntries.map((e) =>
        invoke("delete_trash_entry", { iFilePath: e.path }),
      ),
    );
    this.selection.clearSelection();
    await this.trash.load();
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

  async assignTagToSelected(entryPath: string | undefined, tagId: number) {
    if (!entryPath) return;
    await this.tags.assignTag(entryPath, tagId);

    const entry = this.entries.find((e) => e.path === entryPath);
    if (entry && !entry.tagIds.includes(tagId)) {
      entry.tagIds = [...entry.tagIds, tagId];
    }
  }

  async unassignTagFromEntry(entryPath: string | undefined, tagId: number) {
    if (!entryPath) return;
    await this.tags.unassignTag(entryPath, tagId);

    const entry = this.entries.find((e) => e.path === entryPath);
    if (entry) {
      entry.tagIds = entry.tagIds.filter((id) => id !== tagId);
    }
  }

  async assignStatusToSelected(
    entryPath: string | undefined,
    statusId: number,
  ) {
    if (!entryPath) return;
    await this.statuses.setStatus(entryPath, statusId);

    const entry = this.entries.find((e) => e.path === entryPath);
    if (entry && !entry.statusIds.includes(statusId)) {
      entry.statusIds = [...entry.statusIds, statusId];
    }
  }

  async unassignStatusFromEntry(entryPath: string | undefined, statusId: number) {
    if (!entryPath) return;
    await this.statuses.unassignStatus(entryPath, statusId);

    const entry = this.entries.find((e) => e.path === entryPath);
    if (entry) {
      entry.statusIds = entry.statusIds.filter((id) => id !== statusId);
    }
  }
}
