#!/usr/bin/env node
// New-dependency check: every direct dependency must exist on crates.io, be at
// least 14 days old, and have a row in docs/dependencies.md.
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";

const MIN_AGE_DAYS = 14;
const LEDGER = "docs/dependencies.md";

function directDeps() {
  const meta = JSON.parse(
    execFileSync("cargo", ["metadata", "--format-version", "1"], {
      encoding: "utf8",
      maxBuffer: 64 * 1024 * 1024,
    }),
  );
  const members = new Set(meta.workspace_members);
  const byId = new Map(meta.packages.map((p) => [p.id, p]));
  const out = new Map();
  for (const node of meta.resolve.nodes) {
    if (!members.has(node.id)) continue;
    for (const dep of node.deps) {
      const pkg = byId.get(dep.pkg);
      if (pkg && !members.has(dep.pkg)) out.set(pkg.name, pkg.version);
    }
  }
  return [...out.entries()].sort(([a], [b]) => a.localeCompare(b));
}

const ledger = new Set();
for (const line of readFileSync(LEDGER, "utf8").split("\n")) {
  const m = line.match(/^\|\s*`([^`]+)`\s*\|/);
  if (m) ledger.add(m[1].toLowerCase());
}

let failed = false;
const deps = directDeps();
for (const [name, version] of deps) {
  if (!ledger.has(name.toLowerCase())) {
    console.error(`security: '${name}' has no row in ${LEDGER}`);
    failed = true;
  }
  let info;
  try {
    const res = await fetch(`https://crates.io/api/v1/crates/${name}/${version}`, {
      headers: { "user-agent": "template-rust make-security" },
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    info = await res.json();
  } catch (err) {
    console.error(`security: cannot verify ${name} ${version} on crates.io: ${err.message}`);
    failed = true;
    continue;
  }
  const ageDays = (Date.now() - Date.parse(info.version.created_at)) / 86_400_000;
  if (ageDays < MIN_AGE_DAYS) {
    console.error(
      `security: ${name} ${version} is ${ageDays.toFixed(1)} days old; use it once it is ${MIN_AGE_DAYS} days old`,
    );
    failed = true;
  }
}

if (failed) process.exit(1);
console.log(
  `security: ${deps.length} direct dependencies exist, are older than ${MIN_AGE_DAYS} days and are in the ledger`,
);
