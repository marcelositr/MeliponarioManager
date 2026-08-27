import { access, readFile } from "node:fs/promises";
import { constants } from "node:fs";
import { resolve } from "node:path";

const configPath = resolve("src-tauri/tauri.conf.json");
const config = JSON.parse(await readFile(configPath, "utf8"));

const requiredIcons = [
  "icons/32x32.png",
  "icons/128x128.png",
  "icons/128x128@2x.png",
  "icons/icon.icns",
  "icons/icon.ico"
];

const configuredIcons = config?.bundle?.icon;

if (!Array.isArray(configuredIcons)) {
  console.error("bundle.icon must be an array in src-tauri/tauri.conf.json");
  process.exit(1);
}

const missingConfigEntries = requiredIcons.filter(
  (icon) => !configuredIcons.includes(icon)
);

if (missingConfigEntries.length > 0) {
  console.error(
    `Missing bundle.icon entries: ${missingConfigEntries.join(", ")}`
  );
  process.exit(1);
}

const missingFiles = [];
for (const icon of requiredIcons) {
  const filePath = resolve("src-tauri", icon);
  try {
    await access(filePath, constants.R_OK);
  } catch {
    missingFiles.push(icon);
  }
}

if (missingFiles.length > 0) {
  console.error(
    `Generated bundle icons are missing: ${missingFiles.join(", ")}`
  );
  process.exit(1);
}

console.log("Bundle icon configuration OK");
