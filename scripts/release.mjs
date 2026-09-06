import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import process from "node:process";

const root = resolve(import.meta.dirname, "..");
const [command, version] = process.argv.slice(2);
const versionPattern = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/;

if (!versionPattern.test(version ?? "") || !["prepare", "publish"].includes(command)) {
  throw new Error("Usage: pnpm release:prepare <X.Y.Z> or pnpm release:publish <X.Y.Z>");
}

const paths = {
  cargo: resolve(root, "src-tauri/Cargo.toml"),
  notes: resolve(root, `release-notes/v${version}.json`),
  package: resolve(root, "package.json"),
  tauri: resolve(root, "src-tauri/tauri.conf.json"),
};

if (command === "prepare") {
  updateJsonVersion(paths.package, version);
  updateCargoVersion(paths.cargo, version);
  updateJsonVersion(paths.tauri, version);
  if (!existsSync(paths.notes)) {
    writeJson(paths.notes, { version, fr: "", en: "" });
  }
  console.log(`Release v${version} prepared. Add the French and English release notes, then run:`);
  console.log(`pnpm release:publish ${version}`);
  process.exit(0);
}

const notes = readJson(paths.notes);
if (notes.version !== version || !nonEmptyString(notes.fr) || !nonEmptyString(notes.en)) {
  throw new Error(
    `release-notes/v${version}.json must contain non-empty French and English notes.`,
  );
}
if (
  readJson(paths.package).version !== version ||
  readJson(paths.tauri).version !== version ||
  cargoVersion(paths.cargo) !== version
) {
  throw new Error(`Run pnpm release:prepare ${version} before publishing.`);
}

if (hasStagedChanges()) {
  throw new Error("Commit or unstage the currently staged changes before publishing a release.");
}
runGit([
  "add",
  "package.json",
  "src-tauri/Cargo.toml",
  "src-tauri/tauri.conf.json",
  `release-notes/v${version}.json`,
]);
if (hasStagedChanges()) {
  runGit(["commit", "-m", `chore(release): prepare v${version}`]);
}
runGit(["tag", `v${version}`]);
runGit(["push", "origin", "HEAD:main", "--follow-tags"]);

function updateJsonVersion(path, nextVersion) {
  const content = readFileSync(path, "utf8");
  const pattern = /"version":\s*"[^"]+"/;
  if (!pattern.test(content)) throw new Error(`Unable to update the version in ${path}.`);
  writeFileSync(path, content.replace(pattern, `"version": "${nextVersion}"`));
}

function updateCargoVersion(path, nextVersion) {
  const content = readFileSync(path, "utf8");
  const pattern = /(\[package\][\s\S]*?^version = ")[^"]+(")/m;
  if (!pattern.test(content)) throw new Error("Unable to update the Cargo package version.");
  const updated = content.replace(pattern, `$1${nextVersion}$2`);
  writeFileSync(path, updated);
}

function cargoVersion(path) {
  const match = readFileSync(path, "utf8").match(/\[package\][\s\S]*?^version = "([^"]+)"/m);
  return match?.[1];
}

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

function writeJson(path, value) {
  writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`);
}

function nonEmptyString(value) {
  return typeof value === "string" && value.trim().length > 0;
}

function runGit(args) {
  execFileSync("git", args, { cwd: root, stdio: "inherit" });
}

function hasStagedChanges() {
  try {
    execFileSync("git", ["diff", "--cached", "--quiet"], { cwd: root, stdio: "ignore" });
    return false;
  } catch {
    return true;
  }
}
