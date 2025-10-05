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

<style>
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
