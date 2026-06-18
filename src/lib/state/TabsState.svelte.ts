import { ClipboardState } from "./ClipboardState.svelte";
import { FileExplorerState } from "./FileExplorerState.svelte";
import { TrashState } from "./TrashState.svelte";

export class TabsState {
  private _tabs = $state<FileExplorerState[]>([]);
  activeIndex = $state(0);

  // Shared across every tab: one clipboard (cross-tab copy/paste) and one
  // trash view for the whole window.
  readonly clipboard = new ClipboardState();
  readonly trash = new TrashState();

  constructor() {
    this._tabs = [this.createTab()];
  }

  private createTab(): FileExplorerState {
    return new FileExplorerState(this.clipboard, this.trash);
  }

  get tabs(): FileExplorerState[] {
    return this._tabs;
  }

  get activeTab(): FileExplorerState {
    return this._tabs[this.activeIndex];
  }

  async newTab() {
    const startDir = this.activeTab.currentDir;
    const path =
      !startDir || startDir === "trash://"
        ? this._tabs[0]?.rootDir
        : startDir;
    if (path) await this.openInNewTab(path);
  }

  async openInNewTab(path: string, focus = true) {
    const tab = this.createTab();
    await tab.setRootDir(path);
    this._tabs = [...this._tabs, tab];
    if (focus) this.activeIndex = this._tabs.length - 1;
  }

  closeTab(index: number) {
    if (this._tabs.length <= 1) return;

    const newTabs = this._tabs.filter((_, i) => i !== index);
    this._tabs = newTabs;

    if (this.activeIndex >= newTabs.length) {
      this.activeIndex = newTabs.length - 1;
    } else if (this.activeIndex > index) {
      this.activeIndex--;
    }
  }

  switchTo(index: number) {
    if (index >= 0 && index < this._tabs.length) {
      this.activeIndex = index;
    }
  }

  nextTab() {
    this.activeIndex = (this.activeIndex + 1) % this._tabs.length;
  }

  prevTab() {
    this.activeIndex = (this.activeIndex - 1 + this._tabs.length) % this._tabs.length;
  }

  get tabTitle(): (tab: FileExplorerState) => string {
    return (tab) => {
      const dir = tab.currentDir;
      if (!dir) return "New Tab";
      if (dir === "trash://") return "Trash";
      const parts = dir.replace(/\\/g, "/").split("/").filter(Boolean);
      return parts[parts.length - 1] || dir;
    };
  }
}
