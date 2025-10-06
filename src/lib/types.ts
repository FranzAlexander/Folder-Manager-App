export interface FileSystemEntry {
  name: string;
  isDir: boolean;
  isFile: boolean;
  size: number | null;
  path: string;
  dateModified: string;
  fileType: string;
  tagIds: number[];
  statusIds: number[];
}

export interface Tag {
  id: number;
  name: string;
}

export interface Status {
  id: number;
  name: string;
}

export type ColumnKey =
  | "name"
  | "dateModified"
  | "fileType"
  | "size"
  | "tags"
  | "status";
