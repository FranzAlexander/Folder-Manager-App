import type {
  ColumnKey,
  ConflictingEntry,
  ConflictResolution,
  ExtractProgress,
  FileSystemEntry,
} from "$lib/types";
import { Channel, invoke } from "@tauri-apps/api/core";
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

// A folder is offered as an update target only when its similarity to the
// incoming zip's name reaches this fraction (0..1). Bump it up to require
// closer matches, down to catch looser ones.
const FOLDER_MATCH_THRESHOLD = 0.8;

function isZipEntry(e: FileSystemEntry): boolean {
  return e.isFile && e.name.toLowerCase().endsWith(".zip");
}

function zipStem(name: string): string {
  return name.replace(/\.zip$/i, "");
}

function levenshtein(a: string, b: string): number {
  const m = a.length;
  const n = b.length;
  if (m === 0) return n;
  if (n === 0) return m;

  let prev = Array.from({ length: n + 1 }, (_, i) => i);
  let curr = new Array<number>(n + 1);
  for (let i = 1; i <= m; i++) {
    curr[0] = i;
    for (let j = 1; j <= n; j++) {
      const cost = a[i - 1] === b[j - 1] ? 0 : 1;
      curr[j] = Math.min(prev[j] + 1, curr[j - 1] + 1, prev[j - 1] + cost);
    }
    [prev, curr] = [curr, prev];
  }
  return prev[n];
}

function similarity(a: string, b: string): number {
  const max = Math.max(a.length, b.length);
  return max === 0 ? 1 : 1 - levenshtein(a, b) / max;
}

// A separator or version digit after the shared prefix lets "report-final" or
// "backup2024" match folder "report"/"backup", while "reporting" (next char is
// a letter) does not.
function isBoundary(ch: string | undefined): boolean {
  return ch === undefined || !/[a-z]/.test(ch);
}

// 0..1 score for how well a zip's stem matches a folder name. Exact names score
// 1; a prefix relationship at a boundary scores high; otherwise an edit-distance
// ratio handles near-misses/typos.
function folderMatchScore(stem: string, folder: string): number {
  const a = stem.trim().toLowerCase();
  const b = folder.trim().toLowerCase();
  if (!a || !b) return 0;
  if (a === b) return 1;
  if (a.startsWith(b) && isBoundary(a[b.length]))
    return Math.max(0.9, similarity(a, b));
  if (b.startsWith(a) && isBoundary(b[a.length]))
    return Math.max(0.9, similarity(a, b));
  return similarity(a, b);
}

function bestFolderMatch(
  zipName: string,
  entries: FileSystemEntry[],
): FileSystemEntry | null {
  const stem = zipStem(zipName);
  let best: FileSystemEntry | null = null;
  let bestScore = 0;
  for (const e of entries) {
    if (!e.isDir) continue;
    const score = folderMatchScore(stem, e.name);
    if (score >= FOLDER_MATCH_THRESHOLD && score > bestScore) {
      best = e;
      bestScore = score;
    }
  }
  return best;
}

interface ZipUpdate {
  zipPath: string;
  zipName: string;
  folderPath: string;
  folderName: string;
  phase: "ask" | "conflicts" | "extracting";
  conflicts: ConflictingEntry[];
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

  // Path of the archive awaiting an extract-destination choice. Non-null while
  // the ExtractDialog is open (rendered by FileTable, like the rename input).
  extractZipPath = $state<string | null>(null);
  // Set once extraction starts; drives the dialog's progress phase. Extraction
  // runs off the main thread in Rust, streaming progress over a channel.
  extractProgress = $state<ExtractProgress | null>(null);

  get isExtracting() {
    return this.extractProgress !== null;
  }

  startExtract = (path: string) => {
    this.extractZipPath = path;
  };

  cancelExtract = () => {
    // Ignore dismiss attempts once extraction is underway.
    if (this.isExtracting) return;
    this.extractZipPath = null;
  };

  // Aborts an in-progress extraction; the backend stops at the next chunk and
  // removes any folder it created, then confirmExtract resolves with null.
  requestCancelExtract = async () => {
    if (!this.isExtracting) return;
    await invoke("cancel_extract");
  };

  confirmExtract = async (dest: string) => {
    const path = this.extractZipPath;
    if (!path) return;

    this.extractProgress = { current: 0, total: 0 };
    const onEvent = new Channel<ExtractProgress>();
    onEvent.onmessage = (progress) => (this.extractProgress = progress);

    try {
      // null = cancelled by the user; leave the listing untouched.
      const newPath: string | null = await invoke("execute_zip_extract", {
        path,
        dest,
        resolutions: {},
        onEvent,
      });
      await this.updateEntries(this.currentDir);
      // Only selectable when extracted into the directory we're viewing.
      const entry = newPath
        ? this.entries.find((e) => e.path === newPath)
        : undefined;
      if (entry) {
        const index = this.entries.indexOf(entry);
        this.selection.selectSingle(entry, index);
        this.pendingScrollToIndex = index;
      }
    } catch (e) {
      await this.reportError(e, "Extraction failed");
    } finally {
      this.extractProgress = null;
      this.extractZipPath = null;
    }
  };

  // --- Auto-update a matching folder from an incoming zip -------------------

  // Drives the "a new zip matches an existing folder" flow. null = no prompt.
  // The extracting phase reuses extractProgress / requestCancelExtract.
  zipUpdate = $state<ZipUpdate | null>(null);
  // Zip paths the user dismissed this session, so we don't re-prompt for them.
  private ignoredZipUpdates = new Set<string>();

  // Called by updateEntries on a same-dir refresh: prompts for the first newly
  // arrived zip that closely matches an existing folder name.
  private detectIncomingZip(
    entries: FileSystemEntry[],
    prevZips: Set<string>,
  ) {
    if (this.zipUpdate || this.isExtracting) return;
    for (const zip of entries) {
      if (!isZipEntry(zip)) continue;
      if (prevZips.has(zip.path) || this.ignoredZipUpdates.has(zip.path)) continue;
      const folder = bestFolderMatch(zip.name, entries);
      if (!folder) continue;
      this.zipUpdate = {
        zipPath: zip.path,
        zipName: zip.name,
        folderPath: folder.path,
        folderName: folder.name,
        phase: "ask",
        conflicts: [],
      };
      return; // one prompt at a time
    }
  }

  ignoreZipUpdate = () => {
    if (this.zipUpdate) this.ignoredZipUpdates.add(this.zipUpdate.zipPath);
    this.zipUpdate = null;
  };

  cancelZipUpdate = () => {
    this.zipUpdate = null;
  };

  // "Update" pressed: check for per-file conflicts, then either show the
  // conflict dialog or go straight to extracting.
  beginZipUpdate = async () => {
    const update = this.zipUpdate;
    if (!update) return;
    try {
      const conflicts: ConflictingEntry[] = await invoke("prepare_zip_extract", {
        path: update.zipPath,
        dest: update.folderPath,
      });
      if (this.zipUpdate !== update) return; // dismissed meanwhile
      if (conflicts.length > 0) {
        this.zipUpdate = { ...update, phase: "conflicts", conflicts };
      } else {
        await this.runZipUpdate({});
      }
    } catch (e) {
      await this.reportError(e, "Update failed");
      this.zipUpdate = null;
    }
  };

  // ConflictDialog onApply for the zip flow.
  applyZipUpdateResolutions = async (
    resolutions: Record<string, ConflictResolution>,
  ) => {
    await this.runZipUpdate(resolutions);
  };

  private runZipUpdate = async (
    resolutions: Record<string, ConflictResolution>,
  ) => {
    const update = this.zipUpdate;
    if (!update) return;

    this.zipUpdate = { ...update, phase: "extracting" };
    this.extractProgress = { current: 0, total: 0 };
    const onEvent = new Channel<ExtractProgress>();
    onEvent.onmessage = (progress) => (this.extractProgress = progress);

    try {
      await invoke("execute_zip_extract", {
        path: update.zipPath,
        dest: update.folderPath,
        resolutions,
        onEvent,
      });
      await this.updateEntries(this.currentDir);
    } catch (e) {
      await this.reportError(e, "Update failed");
    } finally {
      this.extractProgress = null;
      this.zipUpdate = null;
    }
  };

  private reportError = async (e: unknown, title: string) => {
    const msg =
      typeof e === "object" && e !== null && "message" in e
        ? String((e as { message: unknown }).message)
        : String(e);
    await message(msg, { title, kind: "error" });
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

  // Tracks which directory `_dirEntries` currently reflects, so updateEntries
  // can tell a refresh-of-the-same-dir (where a zip may have just arrived) from
  // a navigation/first-load (where everything is "new" and we must not prompt).
  private loadedDir: string | null = null;

  updateEntries = async (path: string) => {
    const incoming: FileSystemEntry[] = await invoke("read_directory", { path });
    const prevZips =
      this.loadedDir === path
        ? new Set(this._dirEntries.filter(isZipEntry).map((e) => e.path))
        : null;
    this._dirEntries = incoming;
    this.loadedDir = path;
    if (prevZips) this.detectIncomingZip(incoming, prevZips);
  };

  setRootDir = (path: string) => {
    this.rootDir = path;
    this.history.reset(path);
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

  // Clears search/selection that don't carry across directories. The actual
  // load (and watcher repointing) is driven by an effect in +page.svelte that
  // reacts to currentDir on the active tab — so callers here only move the
  // history cursor and let that effect fetch the entries.
  private resetForNavigation = () => {
    this.search.reset();
    this.selection.clearSelection();
  };

  navigateToDirectory = (path: string) => {
    this.history.push(path);
    this.resetForNavigation();
  };

  goBack = () => {
    if (this.history.back()) this.resetForNavigation();
  };

  goForward = () => {
    if (this.history.forward()) this.resetForNavigation();
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
