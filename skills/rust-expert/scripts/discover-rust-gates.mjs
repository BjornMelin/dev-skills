#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";

const root = path.resolve(process.argv[2] ?? ".");
const commands = new Set();

if (exists("Cargo.toml")) {
  commands.add("cargo fmt --all -- --check");
  commands.add("cargo clippy --workspace --all-targets --all-features --locked -- -D warnings");
  commands.add("cargo test --workspace --all-targets --all-features --locked");
}
if (exists("nextest.toml") || exists(".config/nextest.toml")) {
  commands.add("cargo nextest run --workspace --all-features --locked");
}
if (exists("deny.toml")) commands.add("cargo deny check");
if (exists("rust-toolchain.toml")) commands.add("rustup show active-toolchain");
if (exists("Justfile") || exists("justfile")) addRecipes("just");
if (exists("mise.toml")) commands.add("mise tasks # inspect Rust-related tasks");
if (exists("Makefile")) commands.add("make help # inspect Rust-related targets");

for (const workflow of listFiles(path.join(root, ".github", "workflows"))) {
  const text = fs.readFileSync(workflow, "utf8");
  for (const line of text.split(/\r?\n/)) {
    const match = /^\s*run:\s*(cargo .+|just .+|mise run .+|make .+)/.exec(line);
    if (match) commands.add(match[1].trim());
  }
}

console.log([...commands].join("\n") || "No Rust gates discovered. Inspect repo scripts and CI manually.");

function exists(rel) {
  return fs.existsSync(path.join(root, rel));
}

function addRecipes(runner) {
  const file = exists("Justfile") ? "Justfile" : "justfile";
  const text = fs.readFileSync(path.join(root, file), "utf8");
  for (const line of text.split(/\r?\n/)) {
    const match = /^([a-zA-Z0-9_-]+):/.exec(line);
    if (match && /test|check|lint|fmt|verify|clippy|bench|audit|deny/.test(match[1])) {
      commands.add(`${runner} ${match[1]}`);
    }
  }
}

function listFiles(dir) {
  if (!fs.existsSync(dir)) return [];
  const out = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) out.push(...listFiles(full));
    else out.push(full);
  }
  return out;
}
