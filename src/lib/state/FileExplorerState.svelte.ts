import type {
  ColumnKey,
  ConflictingEntry,
  FileSystemEntry,
} from "$lib/types";
import { invoke } from "@tauri-apps/api/core";
import { message, open } from "@tauri-apps/plugin-dialog";
import { SelectionState } from "./SelectionState.svelte";
import { SearchState } from "./SearchState.svelte";
import { NavigationHistory } from "./NavigationHistory.svelte";
import { statusManager } from "$lib/state/StatusManager.svelte";
import { ClipboardState } from "./ClipboardState.svelte";
import { tagManager } from "./TagManager.svelte";
import { TrashState } from "./TrashState.svelte";

// Cached once — constructing a Collator per comparison is what makes
// localeCompare slow. `numeric` gives natural sort (file2 before file10),
// `base` sensitivity makes it case-insensitive like Windows Explorer.
const collator = new Intl.Collator(undefined, {
  numeric: true,
  sensitivity: "base",
});

function sortEntries(
  entries: FileSystemEntry[],
  column: ColumnKey,
  direction: "asc" | "desc",
): FileSystemEntry[] {
  const dir = direction === "asc" ? 1 : -1;
  return [...entries].sort((a, b) => {
    // Folders always sort above files, regardless of sort direction.
    if (a.isDir !== b.isDir) return a.isDir ? -1 : 1;

    const aValue = a[column as keyof FileSystemEntry];
    const bValue = b[column as keyof FileSystemEntry];
    if (aValue == null && bValue == null) return 0;
    if (aValue == null) return 1;
    if (bValue == null) return -1;
    if (typeof aValue === "string" && typeof bValue === "string")
      return collator.compare(aValue, bValue) * dir;
    if (typeof aValue === "number" && typeof bValue === "number")
      return (aValue - bValue) * dir;
    return 0;
  });
}

export class FileExplorerState {
  rootDir = $state<string>("");
  showSetup = $state(false);
  history = new NavigationHistory();
  private _dirEntries = $state<FileSystemEntry[]>([]);
  sortedColumn = $state<ColumnKey>("name");
  sortedDirection = $state<"asc" | "desc">("desc");

  renamingPath = $state<string | null>(null);
  renameValue = $state("");
  pendingScrollToIndex = $state<number | null>(null);

  selection = new SelectionState();
  // clipboard and trash are shared across all tabs (injected by TabsState)
  clipboard: ClipboardState;
  trash: TrashState;
  readonly entries = $derived.by(() => {
    const base =
      this.currentDir === "trash://" ? this.trash.entries : this._dirEntries;
    return sortEntries(base, this.sortedColumn, this.sortedDirection);
  });

  search = new SearchState(
    () => this.currentDir,
    (entries) => (this._dirEntries = entries),
    () => this.updateEntries(this.currentDir),
  );

  readonly tags = tagManager;
  readonly statuses = statusManager;

  constructor(clipboard: ClipboardState, trash: TrashState) {
    this.clipboard = clipboard;
    this.trash = trash;
  }

  get currentDir() {
    return this.history.current;
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
    this.history.reset(path);
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

  // Loads whatever currentDir now points at: routes trash:// to the trash
  // state, everything else to a directory read. Callers mutate the history
  // index first, then call this. The OS file-watcher is repointed separately
  // by the app-level controller in +page.svelte, which tracks the active tab.
  private loadCurrentDir = async () => {
    this.search.reset();
    this.selection.clearSelection();

    if (this.currentDir === "trash://") {
      await this.trash.load();
    } else {
      await this.updateEntries(this.currentDir);
    }
  };

  navigateToDirectory = async (path: string) => {
    this.history.push(path);
    await this.loadCurrentDir();
  };

  goBack = async () => {
    if (this.history.back()) await this.loadCurrentDir();
  };

  goForward = async () => {
    if (this.history.forward()) await this.loadCurrentDir();
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

  // Optimistically applies a tag/status id change to the local entry so the UI
  // updates without a directory re-read. The backend call is the source of truth.
  private mutateEntryIds(
    entryPath: string,
    field: "tagIds" | "statusIds",
    fn: (ids: number[]) => number[],
  ) {
    const entry = this.entries.find((e) => e.path === entryPath);
    if (entry) entry[field] = fn(entry[field]);
  }

  async assignTagToSelected(entryPath: string | undefined, tagId: number) {
    if (!entryPath) return;
    await this.tags.assignTag(entryPath, tagId);
    this.mutateEntryIds(entryPath, "tagIds", (ids) =>
      ids.includes(tagId) ? ids : [...ids, tagId],
    );
  }

  async unassignTagFromEntry(entryPath: string | undefined, tagId: number) {
    if (!entryPath) return;
    await this.tags.unassignTag(entryPath, tagId);
    this.mutateEntryIds(entryPath, "tagIds", (ids) =>
      ids.filter((id) => id !== tagId),
    );
  }

  async assignStatusToSelected(entryPath: string | undefined, statusId: number) {
    if (!entryPath) return;
    await this.statuses.setStatus(entryPath, statusId);
    this.mutateEntryIds(entryPath, "statusIds", (ids) =>
      ids.includes(statusId) ? ids : [...ids, statusId],
    );
  }

  async unassignStatusFromEntry(entryPath: string | undefined, statusId: number) {
    if (!entryPath) return;
    await this.statuses.unassignStatus(entryPath, statusId);
    this.mutateEntryIds(entryPath, "statusIds", (ids) =>
      ids.filter((id) => id !== statusId),
    );
  }
}
