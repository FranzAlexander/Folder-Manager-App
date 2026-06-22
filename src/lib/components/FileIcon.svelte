<script lang="ts">
  import raw from "$lib/icon-manifest.json";
  import type { FileSystemEntry } from "$lib/types";

  interface FileProps {
    file: FileSystemEntry;
    size?: number;
    isOpen?: boolean;
  }

  interface IconDefinition {
    iconPath?: string;
    fontCharacter?: string;
    fontColor?: string;
    fontSize?: string;
    fontId?: string;
  }

  interface IconManifestMap {
    iconDefinitions?: Record<string, IconDefinition>;

    fileExtensions?: Record<string, string>;
    fileNames?: Record<string, string>;
    folderNames?: Record<string, string>;
    folderNamesExpanded?: Record<string, string>;
    languageIds?: Record<string, string>;

    file?: string;
    folder?: string;
    folderExpanded?: string;

    light?: Omit<IconManifestMap, "iconDefinitions" | "light" | "highContrast">;
    highContrast?: Omit<
      IconManifestMap,
      "iconDefinitions" | "light" | "highContrast"
    >;
  }

  const iconManifest = raw as IconManifestMap;

  let { file, size = 20, isOpen = false }: FileProps = $props();

  let extension = $derived.by(() => {
    if (file.isDir) return null;
    const parts = file.name.split(".");
    return parts.length > 1 ? parts.pop()?.toLowerCase() : null;
  });

  let iconName = $derived.by(() => {
    let iconDefId = "";

    if (file.isDir) {
      const folderName = file.name.toLowerCase();

      if (isOpen) {
        iconDefId =
          iconManifest.folderNamesExpanded?.[folderName] ||
          iconManifest.folderExpanded ||
          "folder-open";
      } else {
        iconDefId =
          iconManifest.folderNames?.[folderName] ||
          iconManifest.folder ||
          "folder";
      }
    } else {
      const fileName = file.name.toLowerCase();

      if (iconManifest.fileNames?.[fileName]) {
        iconDefId = iconManifest.fileNames[fileName];
      } else if (extension && iconManifest.fileExtensions?.[extension]) {
        iconDefId = iconManifest.fileExtensions[extension];
      } else {
        iconDefId = iconManifest.file || "file";
      }
    }

    const iconDef = iconManifest.iconDefinitions?.[iconDefId];
    if (iconDef?.iconPath) {
      const filename = iconDef.iconPath.split("/").pop();
      return filename?.replace(".svg", "") || "file";
    }

    return iconDefId || "file";
  });

  let hasError = $state(false);
</script>

<span
  class="icon-wrapper"
  style="width: {size}px; height: {size}px;"
  title={file.isSymlink ? "Symbolic link" : undefined}
>
  {#if !hasError}
    <img
      src="/icons/{iconName}.svg"
      alt="{file.name} icon"
      width={size}
      height={size}
      class="material-icon"
      onerror={() => (hasError = true)}
    />
  {:else}
    <div
      class="fallback-icon"
      style="width: {size}px; height: {size}px; font-size: {Math.round(
        size * 0.7,
      )}px;"
    >
      {file.isDir ? "📁" : "📄"}
    </div>
  {/if}

  {#if file.isSymlink}
    <svg
      class="symlink-badge"
      style="width: {Math.round(size * 0.55)}px; height: {Math.round(
        size * 0.55,
      )}px;"
      viewBox="0 0 24 24"
      aria-hidden="true"
    >
      <rect x="1" y="1" width="22" height="22" rx="5" fill="#ffffff" />
      <path
        d="M9 16 L15 9 M15 9 H10 M15 9 V14"
        fill="none"
        stroke="#1f6feb"
        stroke-width="2.5"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
  {/if}
</span>

<style>
  .icon-wrapper {
    position: relative;
    display: inline-flex;
    flex-shrink: 0;
    align-items: center;
    justify-content: center;
  }

  .symlink-badge {
    position: absolute;
    left: -2px;
    bottom: -2px;
    pointer-events: none;
    filter: drop-shadow(0 0 1px rgba(0, 0, 0, 0.35));
  }

  .material-icon {
    flex-shrink: 0;
    object-fit: contain;
    transition: transform 0.1s ease;
  }

  .fallback-icon {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #6b7280;
  }
</style>
