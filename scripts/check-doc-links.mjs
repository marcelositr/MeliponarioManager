import { access, readdir, readFile, stat } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = process.cwd();
const entryFiles = ["README.md", "CONTRIBUTING.md", "SECURITY.md", "CHANGELOG.md"];
const entryDirs = ["docs", "wiki"];
const markdownLink = /!?\[[^\]]*\]\(([^)]+)\)/g;

async function collectMarkdownFiles(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const files = [];
  for (const entry of entries) {
    const absolute = path.join(directory, entry.name);
    if (entry.isDirectory()) files.push(...await collectMarkdownFiles(absolute));
    else if (entry.isFile() && entry.name.endsWith(".md")) files.push(absolute);
  }
  return files;
}

function normalizeTarget(rawTarget) {
  const trimmed = rawTarget.trim();
  const unwrapped = trimmed.startsWith("<") && trimmed.endsWith(">") ? trimmed.slice(1, -1) : trimmed;
  const withoutTitle = unwrapped.match(/^(\S+)/)?.[1] ?? unwrapped;
  return withoutTitle.split("#", 1)[0];
}

function isExternalOrAnchor(rawTarget, target) {
  return rawTarget.trim().startsWith("#") || target === "" || /^[a-z][a-z0-9+.-]*:/i.test(target) || target.startsWith("//");
}

async function exists(target) {
  try {
    await access(target);
    return true;
  } catch {
    return false;
  }
}

async function resolveLocalTarget(sourceFile, target) {
  const decoded = decodeURIComponent(target);
  const base = path.resolve(path.dirname(sourceFile), decoded);
  if (await exists(base)) return base;

  if (sourceFile.includes(`${path.sep}wiki${path.sep}`) && path.extname(base) === "") {
    const wikiPage = `${base}.md`;
    if (await exists(wikiPage)) return wikiPage;
  }

  return null;
}

const markdownFiles = [
  ...entryFiles.map((file) => path.join(root, file)),
  ...((await Promise.all(entryDirs.map((directory) => collectMarkdownFiles(path.join(root, directory))))).flat()),
];

const failures = [];
for (const file of markdownFiles) {
  if (!(await exists(file)) || !(await stat(file)).isFile()) continue;
  const content = await readFile(file, "utf8");
  for (const match of content.matchAll(markdownLink)) {
    const rawTarget = match[1];
    const target = normalizeTarget(rawTarget);
    if (isExternalOrAnchor(rawTarget, target)) continue;
    if (!(await resolveLocalTarget(file, target))) {
      failures.push(`${path.relative(root, file)} -> ${rawTarget}`);
    }
  }
}

if (failures.length > 0) {
  console.error("Broken local Markdown links:");
  for (const failure of failures) console.error(`- ${failure}`);
  process.exit(1);
}

console.log(`Checked local Markdown links in ${markdownFiles.length} files.`);
