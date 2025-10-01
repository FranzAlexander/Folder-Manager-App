export interface DirectoryEntry {
  name: string;
  isDir: boolean;
  isFile: boolean;
  size: number | null;
  path: string;
  dateModified: string;
  type: string;
  tagIds: number[];
}

export interface Tag {
  id: number;
  name: string;
}
