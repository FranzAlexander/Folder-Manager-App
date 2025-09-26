// scripts/setup-icons.ts
import fs from "fs";
import path from "path";

console.log("🎨 Setting up Material Icon Theme for Tauri...");

try {
  // Dynamic import to avoid module resolution issues
  const { generateManifest } = await import("material-icon-theme");

  // Generate manifest with basic config
  const manifest = generateManifest({
    activeIconPack: "angular",
    folders: { theme: "specific" },
    files: { associations: {} },
  });

  // Create directories
  const libDir = "src/lib";
  if (!fs.existsSync(libDir)) {
    fs.mkdirSync(libDir, { recursive: true });
  }

  const staticIconsDir = "static/icons";
  if (!fs.existsSync(staticIconsDir)) {
    fs.mkdirSync(staticIconsDir, { recursive: true });
  }

  // Write manifest
  fs.writeFileSync(
    path.join(libDir, "icon-manifest.json"),
    JSON.stringify(manifest, null, 2)
  );

  // Copy icons
  const sourceDir = "node_modules/material-icon-theme/icons";
  const files = fs.readdirSync(sourceDir);
  let copiedCount = 0;

  for (const file of files) {
    if (file.endsWith(".svg")) {
      fs.copyFileSync(
        path.join(sourceDir, file),
        path.join("static/icons", file)
      );
      copiedCount++;
    }
  }

  // Results
  console.log(`✅ Copied ${copiedCount} icons to static/icons/`);
  console.log("✅ Generated icon manifest at src/lib/icon-manifest.json");
  console.log("🎉 Material Icon Theme setup complete!");

  console.log("\n📋 Manifest Overview:");
  console.log(
    `- File extensions: ${Object.keys(manifest.fileExtensions || {}).length}`
  );
  console.log(
    `- Special file names: ${Object.keys(manifest.fileNames || {}).length}`
  );
  console.log(
    `- Folder names: ${Object.keys(manifest.folderNames || {}).length}`
  );
} catch (error) {
  console.error("❌ Error setting up Material Icon Theme:", error);
  console.error("Full error:", error);
  process.exit(1);
}
