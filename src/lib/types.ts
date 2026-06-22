export interface FileSystemEntry {
  name: string;
  isDir: boolean;
  isFile: boolean;
  isSymlink: boolean;
  size: number | null;
  path: string;
  originalPath: string | null;
  dateModified: string;
  fileType: string;
  tagIds: number[];
  statusIds: number[];
  lastOpened: string | null;
}

export interface Tag {
  id: number;
  name: string;
}

export interface Status {
  id: number;
  name: string;
}

export interface ConflictingEntry {
  name: string;
  src: string;
  dest: string;
}

export type ColumnKey =
  | "name"
  | "dateModified"
  | "lastOpened"
  | "fileType"
  | "size"
  | "tags"
  | "status";

export type SearchEvent =
  | {
      event: "searching";
      data: { entries: FileSystemEntry[] };
    }
  | {
      event: "done";
    }
  | {
      event: "notFound";
    };

export type ConflictResolution = "skip" | "keep" | "replace";
