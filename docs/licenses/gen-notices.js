#!/usr/bin/env node
/*
 * gen-notices.js - regenerate the third-party license inventory and
 * THIRD_PARTY_NOTICES.txt for the shengji iOS/web client.
 *
 * Usage (from anywhere):
 *   node docs/licenses/gen-notices.js            # writes THIRD_PARTY_NOTICES.txt + inventory.json
 *   node docs/licenses/gen-notices.js --markdown # also prints markdown tables (for AUDIT.md)
 *   node docs/licenses/gen-notices.js --metadata path/to/meta.json   # reuse a saved cargo metadata dump
 *   node docs/licenses/gen-notices.js --dist path/to/dist             # read source maps from another build dir
 *
 * What it inventories (only what ships in the app):
 *   Rust:  every crate reachable from `shengji-wasm` over *normal* dependency edges for
 *          target wasm32-unknown-unknown (cargo metadata --filter-platform). Proc-macro
 *          crates, build-dependencies and anything only reachable through them are listed
 *          as build-time only (they do not end up in the .wasm). Dev-deps are skipped.
 *   npm:   the packages that are actually inside the webpack bundle, read from the
 *          source maps in frontend/dist/*.map (run `yarn build` first so they are current).
 *          The package list is snapshotted to bundled-packages.json; when no source maps
 *          exist the snapshot is used, and failing that it walks package.json
 *          "dependencies" (minus known build-only tools), which over-approximates.
 *   Manual entries: upstream shengji (MIT), Rust std, webpack runtime, assets.
 *
 * No network access and no writes outside this folder.
 */
"use strict";
const fs = require("fs");
const path = require("path");
const { execFileSync } = require("child_process");

const HERE = __dirname;
const REPO = path.resolve(HERE, "..", "..");
const FRONTEND = path.join(REPO, "frontend");
const args = process.argv.slice(2);
const wantMarkdown = args.includes("--markdown");
const metaArg = args.indexOf("--metadata") >= 0 ? args[args.indexOf("--metadata") + 1] : null;
const distArg = args.indexOf("--dist") >= 0 ? path.resolve(args[args.indexOf("--dist") + 1]) : null;

// ---------------------------------------------------------------------------
// SPDX handling: pick the most permissive alternative of any OR expression.
// Lower rank = preferred. Copyleft is ranked last so it is never chosen when
// an alternative exists.
const RANK = {
  MIT: 1, "MIT-0": 0, "0BSD": 0, "CC0-1.0": 0, Unlicense: 5, ISC: 2, "BSL-1.0": 1.5,
  "BSD-2-Clause": 3, "BSD-3-Clause": 3, Zlib: 3, "Apache-2.0": 4,
  "Apache-2.0 WITH LLVM-exception": 4, "Unicode-3.0": 4, "Unicode-DFS-2016": 4,
  "CC-BY-4.0": 6, "MPL-2.0": 50, "LGPL-2.1": 80, "LGPL-3.0": 80, "GPL-2.0": 99,
  "GPL-3.0": 99, "AGPL-3.0": 100,
};
const COPYLEFT = /GPL|MPL|EPL|CDDL|SSPL|CC-BY-SA|CC-BY-NC|EUPL|OSL|UNLICENSED|UNKNOWN/i;
// Ryu is Apache-2.0 OR BSL-1.0; BSL needs no notice in binary form, so prefer it there.

function tokenize(expr) {
  return expr.replace(/\//g, " OR ").replace(/\(/g, " ( ").replace(/\)/g, " ) ")
    .split(/\s+/).filter(Boolean);
}
// returns array of alternatives; each alternative is an array of license ids (AND-ed)
function parseSpdx(expr) {
  const t = tokenize(expr); let i = 0;
  function primary() {
    if (t[i] === "(") { i++; const r = orExpr(); i++; return r; }
    let id = t[i++];
    if (t[i] === "WITH") { id += " WITH " + t[i + 1]; i += 2; }
    return [[id]];
  }
  function andExpr() {
    let l = primary();
    while (t[i] === "AND") { i++; const r = primary(); const out = [];
      for (const a of l) for (const b of r) out.push([...a, ...b]); l = out; }
    return l;
  }
  function orExpr() { let l = andExpr(); while (t[i] === "OR") { i++; l = l.concat(andExpr()); } return l; }
  return orExpr();
}
function choose(expr) {
  if (!expr) return { chosen: ["UNKNOWN"], dual: false };
  const alts = parseSpdx(expr);
  const score = (a) => Math.max(...a.map((x) => RANK[x] ?? 60)) * 10 + a.length;
  alts.sort((a, b) => score(a) - score(b));
  return { chosen: [...new Set(alts[0])], dual: alts.length > 1 };
}

// ---------------------------------------------------------------------------
function licenseFiles(dir) {
  try {
    return fs.readdirSync(dir).filter((f) => /^(licen[cs]e|copying|unlicense|notice|copyright)/i.test(f))
      .map((f) => path.join(dir, f)).filter((f) => fs.statSync(f).isFile());
  } catch { return []; }
}
// Only look at license files that belong to the elected license(s): e.g. for
// "MIT OR Apache-2.0" elected as MIT, skip LICENSE-APACHE (whose boilerplate
// contains lines that look like copyright statements).
const FILE_TAG = { "Apache-2.0": /apache/i, "MPL-2.0": /mpl/i, Unlicense: /unlicense/i,
  "BSL-1.0": /boost|bsl/i, "BSD-2-Clause": /bsd/i, MIT: /mit/i, "Unicode-3.0": /unicode/i };
function electedFiles(files, chosen) {
  return files.filter((f) => {
    const b = path.basename(f);
    if (/notice/i.test(b)) return false;
    for (const [id, re] of Object.entries(FILE_TAG)) if (re.test(b) && !chosen.includes(id)) return false;
    return true;
  });
}
function copyrightLines(files, chosen) {
  const out = new Set();
  for (const f of electedFiles(files, chosen)) {
    const txt = fs.readFileSync(f, "utf8");
    for (const line of txt.split(/\r?\n/)) {
      const l = line.trim();
      if (!/^(copyright\b|©|\(c\)\s*\d)/i.test(l)) continue;
      if (/copyright (notice|holders?|owner|license|and related)|copyrights in|\[yyyy\]|<year>|\{yyyy\}/i.test(l)) continue;
      if (/^copyright\s*(\(c\)|©)?\s*[\d\s,\-–]*$/i.test(l)) continue; // year only, no holder
      if (l.length < 200) out.add(l);
    }
  }
  return [...out];
}

// ---------------------------------------------------------------------------
// Rust
function rustInventory() {
  let meta;
  if (metaArg) meta = JSON.parse(fs.readFileSync(metaArg, "utf8"));
  else meta = JSON.parse(execFileSync("cargo", ["metadata", "--format-version", "1",
    "--filter-platform", "wasm32-unknown-unknown", "--locked"], { cwd: REPO, maxBuffer: 1 << 28 }).toString());
  const pk = new Map(meta.packages.map((p) => [p.id, p]));
  const nodes = new Map(meta.resolve.nodes.map((n) => [n.id, n]));
  const root = meta.packages.find((p) => p.name === "shengji-wasm").id;
  const runtime = new Set(), buildOnly = new Set();
  const q = [[root, true]];
  while (q.length) {
    const [id, run] = q.shift();
    const p = pk.get(id);
    const isPM = p.targets.some((t) => t.kind.includes("proc-macro"));
    const r = run && !isPM;
    if (r) { if (runtime.has(id)) continue; runtime.add(id); buildOnly.delete(id); }
    else { if (runtime.has(id) || buildOnly.has(id)) continue; buildOnly.add(id); }
    for (const d of nodes.get(id).deps) {
      const kinds = d.dep_kinds.map((k) => k.kind);
      if (kinds.every((k) => k === "dev")) continue;
      q.push([d.pkg, r && kinds.includes(null)]);
    }
  }
  const mk = (id, shipped) => {
    const p = pk.get(id);
    const dir = path.dirname(p.manifest_path);
    const local = p.source == null;
    const files = local ? [] : licenseFiles(dir);
    if (p.license_file) files.push(path.resolve(dir, p.license_file));
    const lic = p.license || (local ? "MIT (workspace crate, upstream shengji / this fork)" : null);
    const { chosen, dual } = local ? { chosen: ["MIT"], dual: false } : choose(p.license);
    let cr = copyrightLines(files, chosen);
    if (!cr.length) cr = [p.authors.length ? `Copyright (c) ${p.authors.join(", ")}` : `Copyright (c) the ${p.name} developers`];
    return { eco: "rust", name: p.name, version: p.version, license: lic, chosen, dual, local, shipped,
      repository: p.repository || "", files, copyright: cr, proc_macro: p.targets.some((t) => t.kind.includes("proc-macro")) };
  };
  const byName = (a, b) => a.name.localeCompare(b.name) || a.version.localeCompare(b.version);
  return {
    shipped: [...runtime].map((id) => mk(id, true)).sort(byName),
    buildOnly: [...buildOnly].map((id) => mk(id, false)).sort(byName),
  };
}

// ---------------------------------------------------------------------------
// npm
const NPM_BUILD_ONLY = new Set(["jest", "ts-jest", "typescript", "ts-loader", "json-schema-to-typescript"]);
const SNAPSHOT = path.join(HERE, "bundled-packages.json");
function npmInventory() {
  const dist = distArg || path.join(FRONTEND, "dist");
  const dirs = new Map(); // pkgDir -> number of bundled modules
  let method;
  const maps = fs.existsSync(dist) ? fs.readdirSync(dist).filter((f) => f.endsWith(".map")) : [];
  if (maps.length) {
    method = `source maps in ${path.relative(REPO, dist) || dist}`;
    for (const f of maps) {
      const m = JSON.parse(fs.readFileSync(path.join(dist, f), "utf8"));
      for (const s of m.sources) {
        const rel = s.replace(/^webpack:\/\/[^/]*\//, "");
        const i = rel.lastIndexOf("node_modules/");
        if (i < 0) continue;
        const rest = rel.slice(i + 13).split("/");
        const name = rest[0].startsWith("@") ? rest[0] + "/" + rest[1] : rest[0];
        const d = path.resolve(FRONTEND, rel.slice(0, i + 13) + name);
        dirs.set(d, (dirs.get(d) || 0) + 1);
      }
    }
    // Snapshot so the notices can be regenerated while dist/ is being rebuilt.
    fs.writeFileSync(SNAPSHOT, JSON.stringify(Object.fromEntries([...dirs].map(([d, n]) =>
      [path.relative(FRONTEND, d).replace(/\\/g, "/"), n])), null, 2));
  } else if (fs.existsSync(SNAPSHOT)) {
    method = "bundled-packages.json snapshot (from the last run that had source maps)";
    for (const [rel, n] of Object.entries(JSON.parse(fs.readFileSync(SNAPSHOT, "utf8"))))
      dirs.set(path.resolve(FRONTEND, rel), n);
  } else {
    method = "package.json dependency walk (no source maps found; over-approximation)";
    const pj = JSON.parse(fs.readFileSync(path.join(FRONTEND, "package.json"), "utf8"));
    const q = Object.keys(pj.dependencies).filter((n) => !NPM_BUILD_ONLY.has(n) && !n.startsWith("@types/"))
      .map((n) => [n, FRONTEND]);
    while (q.length) {
      const [n, from] = q.shift();
      let d = from, found = null;
      while (true) { const c = path.join(d, "node_modules", n); if (fs.existsSync(c)) { found = c; break; }
        const up = path.dirname(d); if (up === d) break; d = up; }
      if (!found || dirs.has(found)) continue;
      dirs.set(found, 1);
      const p = JSON.parse(fs.readFileSync(path.join(found, "package.json"), "utf8"));
      for (const k of Object.keys({ ...p.dependencies, ...p.optionalDependencies })) q.push([k, found]);
    }
  }
  const out = [];
  for (const [d, count] of dirs) {
    const p = JSON.parse(fs.readFileSync(path.join(d, "package.json"), "utf8"));
    let lic = p.license;
    if (lic && typeof lic === "object") lic = lic.type;
    if (!lic && Array.isArray(p.licenses)) lic = p.licenses.map((l) => l.type || l).join(" OR ");
    const files = licenseFiles(d);
    const { chosen, dual } = choose(lic);
    let cr = copyrightLines(files, chosen);
    const author = typeof p.author === "string" ? p.author : p.author && p.author.name;
    if (!cr.length) cr = [`Copyright (c) ${author || "the " + p.name + " authors"}`];
    const repo = typeof p.repository === "string" ? p.repository : (p.repository && p.repository.url) || p.homepage || "";
    out.push({ eco: "npm", name: p.name, version: p.version, license: lic || null, chosen, dual, shipped: true,
      modules: count, repository: repo, files, copyright: cr,
      nested: path.relative(FRONTEND, d).split("node_modules").length > 2 });
  }
  out.sort((a, b) => a.name.localeCompare(b.name) || a.version.localeCompare(b.version));
  return { method, pkgs: out };
}

// ---------------------------------------------------------------------------
// Manual components (not discoverable from package managers)
const MANUAL = [
  { eco: "project", name: "shengji (upstream project this app is forked from)", version: "github.com/rbtying/shengji",
    license: "MIT", chosen: ["MIT"], copyright: ["Copyright (c) 2020 Robert Ying https://robertying.com"],
    repository: "https://github.com/rbtying/shengji", files: [path.join(REPO, "LICENSE")] },
  { eco: "rust", name: "Rust standard library (core, alloc, std, dlmalloc allocator) compiled into the .wasm",
    version: "toolchain", license: "MIT OR Apache-2.0", chosen: ["MIT"], dual: true,
    copyright: ["Copyright (c) The Rust Project Developers"], repository: "https://github.com/rust-lang/rust", files: [] },
  { eco: "npm", name: "webpack runtime (bootstrap code emitted into the bundle)", version: "5.x",
    license: "MIT", chosen: ["MIT"], copyright: ["Copyright JS Foundation and other contributors"],
    repository: "https://github.com/webpack/webpack", files: [] },
];
const ASSETS = [
  { eco: "asset", name: "Sound effect \"taking card\" (frontend/static/434472_dersuperanton_taking-card.mp3)",
    version: "freesound.org sound #434472", license: "CC-BY-4.0", chosen: ["CC-BY-4.0"],
    copyright: ["\"taking card\" by dersuperanton, https://freesound.org/s/434472/ , licensed under CC BY 4.0 (https://creativecommons.org/licenses/by/4.0/). Used unmodified."],
    repository: "https://freesound.org/s/434472/", files: [] },
  { eco: "asset", name: "Playing card SVG images (svgs/, frontend/src/generated/playing-cards*)",
    version: "Adrian Kennard card set", license: "CC0-1.0", chosen: ["CC0-1.0"],
    copyright: ["Card images by Adrian Kennard, https://www.me.uk/cards/ , dedicated to the public domain (CC0 1.0). Optimised with svgo and converted to React components. The app's jokers use separate artwork (below)."],
    repository: "https://www.me.uk/cards/", files: [] },
  { eco: "asset", name: "Joker artwork (frontend/static/jokers/, art/Big_JokerV2.png, art/Small_jokerV2.png; earlier versions art/big-joker-source.png, art/small-joker-source.png)",
    version: "Big and small joker, V2, 2026", license: "Own work (AI-generated)", chosen: ["Own work (AI-generated)"],
    copyright: ["Created for Two to Ace with ChatGPT (OpenAI) image generation. Used under OpenAI's terms of use, which assign the output to the user; no third-party license applies."],
    repository: "", files: [] },
  { eco: "asset", name: "Icons (frontend/src/icons/BarChart.tsx, Gear.tsx, PaperPlane.tsx)",
    version: "Bootstrap Icons 1.11.3 \"bar-chart-fill\", \"gear-fill\", \"send-fill\"", license: "MIT", chosen: ["MIT"],
    copyright: ["Copyright (c) 2019-2024 The Bootstrap Authors"],
    repository: "https://github.com/twbs/icons", files: [] },
];

// ---------------------------------------------------------------------------
// License texts. MIT is emitted once from the canonical template; others are
// taken from a component's own license file where one exists.
const MIT_TEXT = `Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.`;
const CC_TEXT = {
  "CC-BY-4.0": `Creative Commons Attribution 4.0 International (CC BY 4.0).
Full legal code: https://creativecommons.org/licenses/by/4.0/legalcode
You may share and adapt the material for any purpose, including commercially,
provided you give appropriate credit, provide a link to the license, and
indicate if changes were made.`,
  "CC0-1.0": `Creative Commons CC0 1.0 Universal (public domain dedication).
Full legal code: https://creativecommons.org/publicdomain/zero/1.0/legalcode
No attribution is legally required; credit is given as a courtesy.`,
};
const FILE_HINT = { "Apache-2.0": /apache/i, "BSL-1.0": /boost|bsl/i, "Unicode-3.0": /licen/i, "MPL-2.0": /mpl/i };
const PER_COMPONENT = new Set(["BSD-2-Clause", "BSD-3-Clause", "ISC", "0BSD"]);
function licenseText(id, comps) {
  if (id === "MIT") return MIT_TEXT;
  if (CC_TEXT[id]) return CC_TEXT[id];
  // prefer a component whose license is exactly this id with one license file
  for (const c of comps) {
    if (c.chosen.length === 1 && c.chosen[0] === id) {
      const lf = c.files.filter((f) => !/notice/i.test(path.basename(f)));
      const hinted = lf.filter((f) => (FILE_HINT[id] || /./).test(path.basename(f)));
      const pick = lf.length === 1 ? lf[0] : hinted.length === 1 ? hinted[0] : null;
      if (pick) return fs.readFileSync(pick, "utf8").replace(/\r\n/g, "\n").trim();
    }
  }
  return `(Full text: https://spdx.org/licenses/${id}.html)`;
}

// ---------------------------------------------------------------------------
function main() {
  const rust = rustInventory();
  const npm = npmInventory();
  const shippedComps = [MANUAL[0], ...rust.shipped.filter((c) => !c.local), MANUAL[1],
    ...npm.pkgs, MANUAL[2], ...ASSETS];

  // Apache NOTICE files that must be reproduced
  const notices = [];
  for (const c of shippedComps) for (const f of c.files || [])
    if (/^notice/i.test(path.basename(f)) && c.chosen.some((x) => x.startsWith("Apache")))
      notices.push({ comp: c, text: fs.readFileSync(f, "utf8").trim() });

  const ids = [...new Set(shippedComps.flatMap((c) => c.chosen))].sort((a, b) =>
    a === "MIT" ? -1 : b === "MIT" ? 1 : a.localeCompare(b));
  const L = [];
  const hr = "=".repeat(78);
  L.push("THIRD-PARTY SOFTWARE NOTICES AND INFORMATION", "",
    "This application includes the third-party components listed below. Each is",
    "used under the license shown. Where a component is offered under a choice of",
    "licenses, the license we elected is shown (the full offer is in brackets).",
    "Full license texts follow the component list, once per distinct license.",
    "", `Generated by docs/licenses/gen-notices.js on ${new Date().toISOString().slice(0, 10)}.`, "");
  const section = (title, list) => {
    L.push(hr, title, hr, "");
    for (const c of list) {
      const offer = c.dual || (c.license && c.license !== c.chosen.join(" AND ")) ? ` [offered as: ${c.license}]` : "";
      L.push(`${c.name} ${c.version}`, `  License: ${c.chosen.join(" AND ")}${offer}`);
      for (const cr of c.copyright) L.push(`  ${cr}`);
      if (c.repository) L.push(`  Source: ${c.repository.replace(/^git\+/, "")}`);
      L.push("");
    }
  };
  section("UPSTREAM PROJECT", [MANUAL[0]]);
  section("RUST CRATES COMPILED INTO THE WEBASSEMBLY MODULE", [...rust.shipped.filter((c) => !c.local), MANUAL[1]]);
  section("JAVASCRIPT PACKAGES INCLUDED IN THE BUNDLE", [...npm.pkgs, MANUAL[2]]);
  section("MEDIA ASSETS", ASSETS);
  if (notices.length) {
    L.push(hr, "APACHE-2.0 NOTICE FILES", hr, "");
    for (const n of notices) L.push(`--- ${n.comp.name} ${n.comp.version} ---`, n.text, "");
  }
  L.push(hr, "LICENSE TEXTS", hr, "");
  for (const id of ids) {
    const users = shippedComps.filter((c) => c.chosen.includes(id)).map((c) => c.name);
    L.push("-".repeat(78), `${id}`, `Applies to: ${users.join(", ")}`, "-".repeat(78), "");
    if (id === "MIT") L.push("Copyright (c) <the copyright holders listed for each component above>", "");
    if (PER_COMPONENT.has(id)) {
      // BSD/ISC texts name the copyright holder (and BSD-3 names the org in its
      // endorsement clause), so reproduce each component's own file verbatim.
      for (const c of shippedComps.filter((x) => x.chosen.includes(id))) {
        const lf = electedFiles(c.files || [], c.chosen);
        L.push(`[${c.name} ${c.version}]`);
        L.push(lf.length ? lf.map((f) => fs.readFileSync(f, "utf8").replace(/\r\n/g, "\n").trim()).join("\n\n")
          : licenseText(id, shippedComps), "");
      }
    } else L.push(licenseText(id, shippedComps), "");
  }
  fs.writeFileSync(path.join(HERE, "THIRD_PARTY_NOTICES.txt"), L.join("\n").replace(/\n{3,}/g, "\n\n"));

  // Paths inside the cargo registry start at the crate directory, so the inventory is the
  // same whoever regenerates it; everything else is relative to the repo.
  const portable = (f) => {
    const p = f.replace(/\\/g, "/");
    const m = p.match(/\/\.cargo\/registry\/src\/[^/]+\/(.*)$/);
    return m ? "cargo-registry/" + m[1] : path.relative(REPO, f).replace(/\\/g, "/");
  };
  const strip = (c) => ({ ...c, files: (c.files || []).map(portable) });
  const inv = { generated: new Date().toISOString(), npmMethod: npm.method,
    rustShipped: rust.shipped.map(strip), rustBuildOnly: rust.buildOnly.map(strip),
    npmShipped: npm.pkgs.map(strip), manual: MANUAL.map(strip), assets: ASSETS.map(strip) };
  fs.writeFileSync(path.join(HERE, "inventory.json"), JSON.stringify(inv, null, 2));

  const flag = (c) => [
    c.chosen.some((x) => COPYLEFT.test(x)) ? "COPYLEFT/UNKNOWN" : "",
    c.dual ? "choice" : "",
    c.files && !c.files.length && !c.local ? "no license file" : "",
  ].filter(Boolean).join(", ");
  const counts = (list) => { const m = {}; for (const c of list) { const k = c.chosen.join(" AND "); m[k] = (m[k] || 0) + 1; } return m; };
  console.log(`Rust shipped: ${rust.shipped.length} (${rust.shipped.filter((c) => c.local).length} workspace), build-only: ${rust.buildOnly.length}`);
  console.log("  elected licenses:", counts(rust.shipped.filter((c) => !c.local)));
  console.log(`npm shipped: ${npm.pkgs.length} via ${npm.method}`);
  console.log("  elected licenses:", counts(npm.pkgs));
  console.log(`Apache NOTICE files reproduced: ${notices.length}`);
  for (const c of [...rust.shipped, ...npm.pkgs]) if (flag(c).includes("COPYLEFT") || flag(c).includes("no license")) console.log("  FLAG", c.eco, c.name, c.version, c.license, flag(c));

  if (wantMarkdown) {
    const t = (list, extra) => {
      const rows = ["| Name | Version | Declared license | Elected | Notes |", "|---|---|---|---|---|"];
      for (const c of list) rows.push(`| ${c.name} | ${c.version} | ${c.license ?? "(none)"} | ${c.chosen.join(" AND ")} | ${[flag(c), extra ? extra(c) : ""].filter(Boolean).join("; ")} |`);
      return rows.join("\n");
    };
    console.log("\n### Rust shipped\n" + t(rust.shipped, (c) => (c.local ? "workspace crate" : "")));
    console.log("\n### Rust build-only\n" + t(rust.buildOnly, (c) => (c.proc_macro ? "proc-macro" : "dep of proc-macro/build")));
    console.log("\n### npm shipped\n" + t(npm.pkgs, (c) => `${c.modules} module(s)${c.nested ? ", nested copy" : ""}`));
  }
}
main();
