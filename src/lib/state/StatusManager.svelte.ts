import type { Status } from "$lib/types";
import { invoke } from "@tauri-apps/api/core";
import { SvelteMap } from "svelte/reactivity";

export class StatusManager {
  statusesById = new SvelteMap<number, Status>();

  async loadAllStatuses() {
    const statuses = await invoke<Status[]>("get_statuses");
    this.statusesById.clear();
    statuses.forEach((s) => this.statusesById.set(s.id, s));
  }

  async createStatus(name: string): Promise<Status> {
    const status = await invoke<Status>("create_status", { status: name });
    this.statusesById.set(status.id, status);
    return status;
  }

  async setStatus(filePath: string, statusId: number): Promise<Status> {
    const status = await invoke<Status>("assign_status", {
      path: filePath,
      statusId,
    });

    if (!this.statusesById.has(status.id)) {
      this.statusesById.set(status.id, status);
    }
    return status;
  }

  resolveStatus(statusId: number): Status | undefined {
    return this.statusesById.get(statusId);
  }

  resolveStatuses(statusIds: number[]): Status[] {
    return statusIds
      .map((id) => this.statusesById.get(id))
      .filter((s): s is Status => s !== undefined);
  }

  get allStatuses(): Status[] {
    return Array.from(this.statusesById.values());
  }
}

export const statusManager = new StatusManager();
