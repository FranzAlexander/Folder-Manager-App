<!-- FileIcon.svelte - Fixed Version -->
<script lang="ts">
  import {
    // Folders
    Folder,
    FolderOpen,

    // Generic files
    File,
    FileText,
    FileImage,
    FileAudio,
    FileVideo,

    // Code & Development
    Code,
    FileCode,
    Terminal,

    // Documents & Data
    FileSpreadsheet,
    Database,
    Archive,
    Book,
    FileType,

    // System & Apps
    Settings,
    Package,
    Zap,
    Monitor,
    Smartphone,

    // Special
    Lock,
    Key,
    Globe,
    Palette,
  } from "@lucide/svelte";

  interface FileProps {
    file: {
      name: string;
      is_dir: boolean;
      is_file: boolean;
      size?: number | null;
    };
    size?: number;
    isOpen?: boolean;
    class?: string;
  }

  let {
    file,
    size = 20,
    isOpen = false,
    class: className = "",
  }: FileProps = $props();

  // Extract extension from filename
  let extension = $derived.by(() => {
    if (file.is_dir) return null;
    const parts = file.name.split(".");
    return parts.length > 1 ? parts.pop()?.toLowerCase() : null;
  });

  let IconComponent = $derived.by(() => {
    if (file.is_dir) {
      return isOpen ? FolderOpen : Folder;
    }

    if (!extension) return File;

    // Web Technologies
    if (extension === "html") return Globe;
    if (extension === "css") return Palette;
    if (["js", "jsx"].includes(extension)) return FileCode;
    if (["ts", "tsx"].includes(extension)) return Code;
    if (extension === "json") return FileType;
    if (extension === "xml") return Code;

    // Programming Languages
    if (extension === "py") return FileCode;
    if (extension === "java") return FileCode;
    if (["cpp", "c", "h"].includes(extension)) return Code;
    if (extension === "cs") return FileCode;
    if (extension === "php") return Code;
    if (extension === "rb") return FileCode;
    if (extension === "go") return FileCode;
    if (extension === "rs") return Code;
    if (extension === "swift") return FileCode;
    if (extension === "kt") return FileCode;
    if (extension === "dart") return FileCode;

    // Shell & Scripts
    if (["sh", "bash", "zsh", "fish"].includes(extension)) return Terminal;
    if (["bat", "cmd", "ps1"].includes(extension)) return Terminal;

    // Config Files
    if (["yml", "yaml", "toml", "ini", "conf", "config"].includes(extension))
      return Settings;
    if (["env", "gitignore", "dockerignore"].includes(extension))
      return Settings;

    // Documents
    if (["txt", "rtf"].includes(extension)) return FileText;
    if (["md", "markdown", "rst"].includes(extension)) return Book;
    if (["pdf", "doc", "docx"].includes(extension)) return FileText;

    // Spreadsheets & Data
    if (["xls", "xlsx", "csv", "ods"].includes(extension))
      return FileSpreadsheet;
    if (["sql", "db", "sqlite", "sqlite3"].includes(extension)) return Database;

    // Images
    if (
      [
        "jpg",
        "jpeg",
        "png",
        "gif",
        "bmp",
        "svg",
        "webp",
        "ico",
        "tiff",
      ].includes(extension)
    )
      return FileImage;

    // Audio
    if (["mp3", "wav", "flac", "ogg", "m4a", "aac", "wma"].includes(extension))
      return FileAudio;

    // Video
    if (
      ["mp4", "avi", "mkv", "mov", "wmv", "webm", "flv", "m4v"].includes(
        extension
      )
    )
      return FileVideo;

    // Archives
    if (["zip", "rar", "7z", "tar", "gz", "bz2", "xz"].includes(extension))
      return Archive;

    // Executables & Apps
    if (["exe", "msi"].includes(extension)) return Package;
    if (["app", "dmg"].includes(extension)) return Package;
    if (["deb", "rpm", "pkg"].includes(extension)) return Package;
    if (["apk", "ipa"].includes(extension)) return Smartphone;

    // Fonts
    if (["ttf", "otf", "woff", "woff2"].includes(extension)) return FileType;

    // Security & Certificates
    if (["key", "pem", "crt", "p12", "pfx"].includes(extension)) return Key;
    if (["gpg", "asc"].includes(extension)) return Lock;

    // Special Development Files
    if (extension === "dockerfile") return Settings;
    if (extension === "makefile") return Code;
    if (extension === "license") return FileText;
    if (extension === "readme") return Book;

    // Default fallback
    return File;
  });

  // Get file type for styling
  let fileType = $derived.by(() => {
    if (file.is_dir) return "folder";
    if (!extension) return "file";

    if (
      [
        "js",
        "ts",
        "jsx",
        "tsx",
        "html",
        "css",
        "py",
        "java",
        "cpp",
        "c",
        "rs",
      ].includes(extension)
    ) {
      return "code";
    }
    if (["jpg", "png", "gif", "svg"].includes(extension)) {
      return "image";
    }
    if (["mp3", "wav", "flac"].includes(extension)) {
      return "audio";
    }
    if (["mp4", "avi", "mkv"].includes(extension)) {
      return "video";
    }
    if (["zip", "rar", "7z"].includes(extension)) {
      return "archive";
    }
    if (["exe", "msi", "app"].includes(extension)) {
      return "executable";
    }
    if (["txt", "md", "pdf", "doc"].includes(extension)) {
      return "document";
    }

    return "file";
  });

  // Get Tailwind color class for file type
  function getIconColor(type: string): string {
    const colorMap: Record<string, string> = {
      folder: "text-blue-500",
      code: "text-purple-500",
      image: "text-emerald-500",
      audio: "text-red-500",
      video: "text-pink-500",
      archive: "text-amber-500",
      executable: "text-red-600",
      document: "text-indigo-500",
      file: "text-gray-500",
    };
    return colorMap[type] || "text-gray-500";
  }
</script>

<IconComponent
  {size}
  class="flex-shrink-0 transition-colors duration-200 {getIconColor(
    fileType
  )} {className}"
/>
