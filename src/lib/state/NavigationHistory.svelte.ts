/**
 * Back/forward navigation history for a single tab: an ordered stack of paths
 * plus a cursor. Pure data — no IPC or side effects. The explorer is
 * responsible for (re)loading the directory after the cursor moves.
 */
export class NavigationHistory {
  private paths = $state<string[]>([]);
  private index = $state(0);

  get current(): string {
    return this.paths[this.index] ?? "";
  }

  get canGoBack(): boolean {
    return this.index > 0;
  }

  get canGoForward(): boolean {
    return this.index < this.paths.length - 1;
  }

  /** Replaces the whole history with a single entry (e.g. a new root). */
  reset(path: string) {
    this.paths = [path];
    this.index = 0;
  }

  /** Pushes a new entry, discarding any forward history first. */
  push(path: string) {
    if (this.canGoForward) {
      this.paths = this.paths.slice(0, this.index + 1);
    }
    this.paths.push(path);
    this.index = this.paths.length - 1;
  }

  /** Moves the cursor back one entry; returns whether it actually moved. */
  back(): boolean {
    if (!this.canGoBack) return false;
    this.index--;
    return true;
  }

  /** Moves the cursor forward one entry; returns whether it actually moved. */
  forward(): boolean {
    if (!this.canGoForward) return false;
    this.index++;
    return true;
  }
}
