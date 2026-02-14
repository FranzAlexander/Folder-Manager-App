import type { Tag } from "$lib/types";
import { invoke } from "@tauri-apps/api/core";
import { SvelteMap } from "svelte/reactivity";

export class TagManager {
  tagsById = new SvelteMap<number, Tag>();

  async loadAllTags() {
    const tags = await invoke<Tag[]>("get_tags");
    this.tagsById.clear();
    tags.forEach((t) => this.tagsById.set(t.id, t));
  }

  async createTag(name: string): Promise<Tag> {
    const tag = await invoke<Tag>("create_tag", { tag: name });
    this.tagsById.set(tag.id, tag);
    return tag;
  }

  async assignTag(filePath: string, tagName: string): Promise<Tag> {
    const tag = await invoke<Tag>("assign_tag", { filePath, tagName });

    this.tagsById.set(tag.id, tag);

    return tag;
  }

  resolveTag(tagId: number): Tag | undefined {
    return this.tagsById.get(tagId);
  }

  resolveTags(tagIds: number[]): Tag[] {
    return tagIds
      .map((id) => this.tagsById.get(id))
      .filter((t): t is Tag => t !== undefined);
  }

  get allTags(): Tag[] {
    return Array.from(this.tagsById.values());
  }
}

export const tagManager = new TagManager();
