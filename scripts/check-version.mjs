import fs from "node:fs";

const packageJson = JSON.parse(fs.readFileSync("package.json", "utf8"));
const tauriConfig = JSON.parse(fs.readFileSync("src-tauri/tauri.conf.json", "utf8"));
const cargoToml = fs.readFileSync("src-tauri/Cargo.toml", "utf8");

const packageSection = cargoToml.split(/\n(?=\[)/)[0];
const cargoVersion = packageSection.match(/^version\s*=\s*"([^"]+)"/m)?.[1];

const versions = {
  packageJson: packageJson.version,
  cargoToml: cargoVersion,
  tauriConfig: tauriConfig.version,
};

for (const [source, version] of Object.entries(versions)) {
  if (typeof version !== "string" || version.trim() === "") {
    throw new Error(`Missing version in ${source}`);
  }
}

const uniqueVersions = new Set(Object.values(versions));
if (uniqueVersions.size !== 1) {
  throw new Error(`Version mismatch: ${JSON.stringify(versions)}`);
}

const version = packageJson.version;
const tagIndex = process.argv.indexOf("--tag");

if (tagIndex !== -1) {
  const tag = process.argv[tagIndex + 1];

  if (!tag || !tag.startsWith("v")) {
    throw new Error("Expected --tag in the form v0.x.y");
  }

  const tagVersion = tag.slice(1);
  if (tagVersion !== version) {
    throw new Error(`Tag ${tag} does not match application version ${version}`);
  }
}

console.log(`Version metadata OK: ${version}`);
