import { FileExplorerState } from "./FileExplorerState.svelte";

export class TabsState {
  private _tabs = $state<FileExplorerState[]>([]);
  activeIndex = $state(0);

  constructor(initialTab: FileExplorerState) {
    this._tabs = [initialTab];
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
    const tab = new FileExplorerState();
    await tab.setRootDir(path);
    this._tabs = [...this._tabs, tab];
    if (focus) this.activeIndex = this._tabs.length - 1;
  }

  closeTab(index: number) {
    if (this._tabs.length <= 1) return;

    const tab = this._tabs[index];
    tab.stopWatching();

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
