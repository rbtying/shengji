# Third-party license audit: Shengji iOS app

Audit date: 2026-09-27. Repo `C:\Claude\shengji`, branch `computer-players`. Scope: what ships in a closed-source iOS (App Store) build. That means the webpack bundle, the `shengji-wasm` WebAssembly module and the static assets, all wrapped in Capacitor.

## Summary

**Verdict: no blockers.** Nothing that ships is GPL, LGPL, AGPL, MPL-only, EPL, SSPL, CC-BY-SA, CC-NC, unlicensed or of unknown license. Every shipped component allows closed-source commercial distribution, including through the App Store. The remaining obligations are attribution and notice requirements. Most of them are met by shipping `THIRD_PARTY_NOTICES.txt` in the app and linking to it from an in-app "Licenses / Credits" screen.

**Actions needed before release**

1. **Ship the notices.** Bundle `docs/licenses/THIRD_PARTY_NOTICES.txt`, for example by copying it into `dist/` with CopyWebpackPlugin, and link it from the Credits/Settings screen. It covers MIT, BSD-3-Clause, ISC, 0BSD, BSL-1.0, Unicode-3.0 and the CC credits. It must keep Robert Ying's MIT notice for the upstream project. Regenerate it with `node docs/licenses/gen-notices.js` after any dependency change.
2. **Credit the CC-BY-4.0 sound on screen.** `434472_dersuperanton_taking-card.mp3` needs a visible credit: "taking card" by dersuperanton (freesound.org/s/434472), CC BY 4.0, with a link to the license. The existing Credits changelog line ("Sound sourced from dersuperanton at freesound.org") has no license name or link. Put the full credit on the Credits screen or in the notices, and link to it. The notices file already includes it. The other option is to replace the sound with a CC0 one or your own recording.
3. **Handle the Font Awesome icons (CC-BY-4.0).** `frontend/src/icons/Gear.tsx` and `PaperPlane.tsx` use the SVG paths of Font Awesome Free 5 "cog" and "paper-plane". Font Awesome Free icons are CC BY 4.0, and no credit exists in the repo today. Either keep the credit that is now in THIRD_PARTY_NOTICES.txt, or swap both for MIT/ISC icons such as Bootstrap Icons, Heroicons or Lucide. `BarChart.tsx` matches Bootstrap Icons "bar-chart-fill" (MIT) and is covered in the notices. These icon identifications come from comparing the SVG paths by eye, not from a documented source, so treat them as highly likely rather than certain.
4. **Record the dual-license elections (no code change).** 40 shipped Rust crates offer a choice of licenses. We elect **MIT** wherever it is offered: "MIT OR Apache-2.0", "Apache-2.0/MIT", "Unlicense OR MIT", "BSD-2-Clause OR Apache-2.0 OR MIT" and `slog`'s "MPL-2.0 OR MIT OR Apache-2.0". `ryu` (Apache-2.0 OR BSL-1.0) is taken under **BSL-1.0**, which needs no notice for binary distribution; we list it anyway. The Rust standard library compiled into the .wasm (MIT OR Apache-2.0) is also taken under MIT. Because no Apache-2.0 election is made, no Apache NOTICE files need reproducing, and none exist among shipped components anyway. The elections are recorded in the notices file.
5. **Remove the upstream author's personal and payment links (branding, not license).** MIT grants no trademark or endorsement rights. Remove or reword these before submitting:
   - The Credits screen (`frontend/src/Credits.tsx`) has "Made by Robert Ying (me@robertying.com)…", a Venmo handle and a Stripe donation link for the upstream author.
   - `static/index.html` has `og:url`/`twitter:*` tags pointing at robertying.com and @rbtying.

   Keep an attribution to the upstream project, for example "Based on shengji by Robert Ying, MIT License", and remove anything implying he makes, endorses or collects money for the app. Donation links to a third party are also an App Store review risk under guideline 3.1.1 / 3.2.1(vii).
6. **Clean up Sentry (housekeeping).**
   - `@sentry/react` 9.13 plus `@sentry/browser` and `@sentry/core` **are still bundled**: `index.tsx` imports `Sentry.ErrorBoundary`. They are MIT, so the license is fine. Since `Sentry.init` is gone, no data is sent.
   - Consider replacing `Sentry.ErrorBoundary` with a 15-line React error boundary and removing `@sentry/react` and `@sentry/tracing`. That shrinks the bundle and avoids questions in the App Store privacy review.
   - `@sentry/tracing` (7.x) is declared but not bundled; delete it.
7. **Fix `package.json` classification (housekeeping).**
   - Move `jest`, `ts-jest`, `typescript`, `ts-loader`, `json-schema-to-typescript` and `@types/*` to devDependencies. They are build/test tools and are not in the bundle.
   - `react-color` is listed under devDependencies but **is bundled**, because `SettingsPane.tsx` imports `CompactPicker`. Move it to dependencies. Its license is MIT, so this is not a license problem.
8. **Capacitor (not installed yet).** `@capacitor/core`, `@capacitor/ios` and the CLI are MIT (Ionic). After adding them, re-run the script. Also add any Capacitor plugins and their CocoaPods/SPM native dependencies to the notices by hand, because the script does not scan iOS native code.

Nothing needs to be replaced for license reasons. The only "replace" suggestions are optional: the sound in item 2 and the icons in item 3, to avoid attribution.

## How the inventory was built

- **Rust.** Ran `cargo metadata --format-version 1 --filter-platform wasm32-unknown-unknown` and walked `resolve.nodes` from `shengji-wasm` over normal dependency edges only. Dev-deps were skipped. Proc-macro crates, build-dependencies and anything reachable only through them are listed separately as build-time only; they run in the compiler and put no code into the .wasm.
- **npm.** Read the webpack source maps (`frontend/dist/*.map`, `yarn build` of 2026-09-27). This gives the exact set of `node_modules` packages whose code is in the shipped JS. The list is saved to `bundled-packages.json`. The package.json/LICENSE of each package was then read. `license-checker` was not needed.
- **Assets.** Checked by hand.
- All results are reproducible with `node docs/licenses/gen-notices.js [--markdown]`. It writes `THIRD_PARTY_NOTICES.txt` and `inventory.json`.

## License counts (shipped, third-party)

| Ecosystem | Components | Elected licenses |
|---|---|---|
| Rust crates in .wasm | 58 third-party (+6 workspace crates, +Rust std) | MIT 42, Unicode-3.0 15, BSL-1.0 1 |
| npm packages in JS bundle | 47 (incl. duplicate nested versions) | MIT 42, ISC 2, BSD-3-Clause 2, 0BSD 1 (+webpack runtime, MIT) |
| Assets | 4 groups | CC-BY-4.0 2 (sound, FA icons), CC0 1 (cards), MIT 1 (Bootstrap icon) |

The 58 Rust crates as declared (before election) break down as follows: dual/multi-licensed 40 (39 with an MIT option, plus ryu: Apache-2.0 OR BSL-1.0), MIT-only 3 (ruzstd, schemars, twox-hash), Unicode-3.0-only 15 (ICU4X crates pulled in by `url` → `idna`). There is no copyleft-only crate. `slog` offers MPL-2.0 only as one option among three, and we elect MIT.

## Flagged items

| Item | Where | Issue | Recommended fix |
|---|---|---|---|
| taking-card.mp3 | frontend/static | CC-BY-4.0, attribution required | Comply: full credit plus license link in Credits/notices (done in notices). Or replace with a CC0 sound |
| Gear / PaperPlane icons | frontend/src/icons | Font Awesome Free, CC-BY-4.0, no credit in repo | Comply: credit (done in notices). Or replace with MIT icons |
| BarChart icon | frontend/src/icons | Bootstrap Icons, MIT, no credit in repo | Comply: covered in notices |
| ICU4X crates (15) | wasm via url → idna | Unicode-3.0 requires the notice in documentation | Comply: Unicode license text is in notices |
| exenv, hoist-non-react-statics | npm bundle | BSD-3-Clause: reproduce notice, no endorsement | Comply: verbatim texts in notices |
| canvas-confetti, material-colors, tslib | npm bundle | ISC / 0BSD | Comply: texts in notices (0BSD needs none) |
| 40 dual-licensed crates, Rust std | wasm | License choice | Elect MIT (ryu: BSL-1.0); recorded in notices |
| slog | wasm | MPL-2.0 OR MIT OR Apache-2.0 | Elect MIT; MPL does not apply |
| gloo-utils 0.1.7 | wasm | No LICENSE file in the crate package (declares MIT/Apache-2.0) | Comply under MIT; copyright "Rust and WebAssembly Working Group" is in notices |
| @sentry/react, browser, core | npm bundle | MIT (OK); telemetry library still bundled | Optional: remove (see action 6) |
| react-color | npm bundle | MIT (OK); misfiled as devDependency | Move to dependencies |
| Upstream branding and donation links | Credits.tsx, index.html | Not a license issue; implies endorsement, App Store risk | Remove or reword; keep the MIT attribution |
| Workspace crates local-game, wasm-rpc-impl, shengji-core, shengji-mechanics, shengji-types | repo | No `license` field in Cargo.toml | The code comes from the MIT upstream or is your own. No action needed for closed source. Optionally add `publish = false` / `license = "UNLICENSED"` to your own crates |


## Assets

| Asset | Ships? | Source / provenance | License | Obligation / status |
|---|---|---|---|---|
| `frontend/static/434472_dersuperanton_taking-card.mp3` | Yes (copied to dist) | freesound.org #434472 by dersuperanton | CC-BY-4.0 | Credit author, title and source, link the license, note changes. **Action 2** |
| `svgs/*` → `frontend/src/generated/playing-cards*` (card faces, bundled as JS) | Yes | Adrian Kennard, https://www.me.uk/cards/ (per `svgs/README.md`) | CC0-1.0 | None required; courtesy credit in notices |
| `frontend/src/icons/Gear.tsx`, `PaperPlane.tsx` | Yes | Path data matches Font Awesome Free 5 "cog" / "paper-plane" | CC-BY-4.0 | Credit (in notices) or replace. **Action 3** |
| `frontend/src/icons/BarChart.tsx` | Yes | Matches Bootstrap Icons "bar-chart-fill" | MIT | Notice (in notices) |
| `favicon/*` (PNG/ICO icons, manifest) | Not by webpack. Served by the Rust server as a fallback dir; referenced from index.html | "Card games icon", https://freesvg.org/card-games-icon (per `favicon/README.md`) | Public domain (freesvg.org) | None. For the iOS app you will need a proper 1024px App Store icon anyway; an original icon is recommended so the app is distinguishable from the upstream site |
| `frontend/static/index.html`, `rules.html`, `timer-worker.js` | Yes | Upstream shengji authors | MIT (upstream) | Covered by Robert Ying's notice. rules.html only links to benzhang.io; no copied text or images |
| Fonts | No font files ship | CSS uses system font names only (sans-serif, "Apple Symbols", "Segoe UI Symbol", "Symbola", "DejaVu Sans") | n/a | None |
| Emoji | No emoji images ship | emoji-picker-react (MIT) is used with `EmojiStyle.NATIVE`, so the OS emoji font renders them and no CDN images are fetched | n/a | None. Keep `EmojiStyle.NATIVE`: the Apple/Google/Twitter styles would load third-party emoji images from a CDN, with their own licenses |
| Sounds other than the mp3 | n/a | `beep.ts` synthesises tones with WebAudio | n/a | None |

No unknown-provenance images, sounds or fonts were found.

## Server-only components (not shipped in the app; lower priority)

`backend/` (crate `shengji`), `storage/`, `bot-client/` (crate `shengji-bot`): `cargo metadata --filter-platform x86_64-unknown-linux-gnu`, normal and build deps, 178 crates including workspace crates.

| Declared license | Count |
|---|---|
| MIT OR Apache-2.0 (all spellings) | 99 |
| MIT | 39 |
| Unicode-3.0 | 19 |
| MPL-2.0 OR MIT OR Apache-2.0 (slog, slog-async, slog-json) | 3 |
| Unlicense OR MIT / BSD-2-Clause OR Apache-2.0 OR MIT / Apache-2.0 OR BSL-1.0 / (MIT OR Apache-2.0) AND Unicode-3.0 | 2 / 2 / 1 / 1 |
| BSD-3-Clause (redis 0.23.3, sha1_smol 1.0.1) | 2 |
| MIT AND BSD-3-Clause (matchit 0.7.3) | 1 |
| Apache-2.0 only (sync_wrapper 0.1.2) | 1 |
| **MPL-2.0 only (slog-bunyan 2.5.0)** | 1 |
| workspace crates (no license field) | 7 |

Notes for hosting:

- Running the server as a hosted service is not "distribution" under any of these licenses. MIT, BSD, Apache and MPL impose no obligations on network use, and there is no AGPL or SSPL anywhere.
- **slog-bunyan (MPL-2.0)** is the only weak-copyleft crate. It is fine to use unmodified. If you ever *distribute* server binaries (for example, a public Docker image), you must say where its source can be obtained, and any modifications you make to its own files must stay MPL. Swapping it for `slog-json` (which offers MIT) removes even that.
- If you distribute server binaries, ship a server notices file too. `sync_wrapper` is Apache-2.0-only and has no NOTICE file. `redis`, `sha1_smol` and `matchit` need their BSD-3 texts.

## Shipped Rust crates (wasm32-unknown-unknown, reachable from shengji-wasm)

"Elected" is the license we use when the crate offers a choice. The 6 workspace crates are this fork's/upstream's own code (MIT upstream).

| Name | Version | Declared license | Elected | Notes |
|---|---|---|---|---|
| anyhow | 1.0.93 | MIT OR Apache-2.0 | MIT | choice |
| byteorder | 1.5.0 | Unlicense OR MIT | MIT | choice |
| cfg-if | 1.0.0 | MIT/Apache-2.0 | MIT | choice |
| console_error_panic_hook | 0.1.7 | Apache-2.0/MIT | MIT | choice |
| dyn-clone | 1.0.17 | MIT OR Apache-2.0 | MIT | choice |
| either | 1.13.0 | MIT OR Apache-2.0 | MIT | choice |
| erased-serde | 0.3.31 | MIT OR Apache-2.0 | MIT | choice |
| form_urlencoded | 1.2.1 | MIT OR Apache-2.0 | MIT | choice |
| getrandom | 0.2.15 | MIT OR Apache-2.0 | MIT | choice |
| gloo-utils | 0.1.7 | MIT/Apache-2.0 | MIT | choice, no license file |
| icu_collections | 1.5.0 | Unicode-3.0 | Unicode-3.0 |  |
| icu_locid | 1.5.0 | Unicode-3.0 | Unicode-3.0 |  |
| icu_locid_transform | 1.5.0 | Unicode-3.0 | Unicode-3.0 |  |
| icu_locid_transform_data | 1.5.0 | Unicode-3.0 | Unicode-3.0 |  |
| icu_normalizer | 1.5.0 | Unicode-3.0 | Unicode-3.0 |  |
| icu_normalizer_data | 1.5.0 | Unicode-3.0 | Unicode-3.0 |  |
| icu_properties | 1.5.1 | Unicode-3.0 | Unicode-3.0 |  |
| icu_properties_data | 1.5.0 | Unicode-3.0 | Unicode-3.0 |  |
| icu_provider | 1.5.0 | Unicode-3.0 | Unicode-3.0 |  |
| idna | 1.0.3 | MIT OR Apache-2.0 | MIT | choice |
| idna_adapter | 1.2.0 | Apache-2.0 OR MIT | MIT | choice |
| itertools | 0.10.5 | MIT/Apache-2.0 | MIT | choice |
| itoa | 1.0.14 | MIT OR Apache-2.0 | MIT | choice |
| js-sys | 0.3.74 | MIT OR Apache-2.0 | MIT | choice |
| lazy_static | 1.5.0 | MIT OR Apache-2.0 | MIT | choice |
| litemap | 0.7.4 | Unicode-3.0 | Unicode-3.0 |  |
| local-game | 0.1.0 | MIT (workspace crate, upstream shengji / this fork) | MIT | workspace crate |
| memchr | 2.7.4 | Unlicense OR MIT | MIT | choice |
| once_cell | 1.20.2 | MIT OR Apache-2.0 | MIT | choice |
| percent-encoding | 2.3.1 | MIT OR Apache-2.0 | MIT | choice |
| ppv-lite86 | 0.2.20 | MIT/Apache-2.0 | MIT | choice |
| rand | 0.8.5 | MIT OR Apache-2.0 | MIT | choice |
| rand_chacha | 0.3.1 | MIT OR Apache-2.0 | MIT | choice |
| rand_core | 0.6.4 | MIT OR Apache-2.0 | MIT | choice |
| ruzstd | 0.4.0 | MIT | MIT |  |
| ryu | 1.0.18 | Apache-2.0 OR BSL-1.0 | BSL-1.0 | choice |
| schemars | 0.8.21 | MIT | MIT |  |
| serde | 1.0.215 | MIT OR Apache-2.0 | MIT | choice |
| serde_json | 1.0.133 | MIT OR Apache-2.0 | MIT | choice |
| shengji-core | 0.1.0 | MIT (workspace crate, upstream shengji / this fork) | MIT | workspace crate |
| shengji-mechanics | 0.1.0 | MIT (workspace crate, upstream shengji / this fork) | MIT | workspace crate |
| shengji-types | 0.1.0 | MIT (workspace crate, upstream shengji / this fork) | MIT | workspace crate |
| shengji-wasm | 0.1.0 | MIT | MIT | workspace crate |
| slog | 2.7.0 | MPL-2.0 OR MIT OR Apache-2.0 | MIT | choice |
| smallvec | 1.13.2 | MIT OR Apache-2.0 | MIT | choice |
| stable_deref_trait | 1.2.0 | MIT/Apache-2.0 | MIT | choice |
| static_assertions | 1.1.0 | MIT OR Apache-2.0 | MIT | choice |
| thiserror | 1.0.69 | MIT OR Apache-2.0 | MIT | choice |
| thiserror-core | 1.0.50 | MIT OR Apache-2.0 | MIT | choice |
| tinystr | 0.7.6 | Unicode-3.0 | Unicode-3.0 |  |
| twox-hash | 1.6.3 | MIT | MIT |  |
| url | 2.5.4 | MIT OR Apache-2.0 | MIT | choice |
| utf16_iter | 1.0.5 | Apache-2.0 OR MIT | MIT | choice |
| utf8_iter | 1.0.4 | Apache-2.0 OR MIT | MIT | choice |
| wasm-bindgen | 0.2.97 | MIT OR Apache-2.0 | MIT | choice |
| wasm-rpc-impl | 0.1.0 | MIT (workspace crate, upstream shengji / this fork) | MIT | workspace crate |
| web-sys | 0.3.74 | MIT OR Apache-2.0 | MIT | choice |
| web-time | 1.1.0 | MIT OR Apache-2.0 | MIT | choice |
| write16 | 1.0.0 | Apache-2.0 OR MIT | MIT | choice |
| writeable | 0.5.5 | Unicode-3.0 | Unicode-3.0 |  |
| yoke | 0.7.5 | Unicode-3.0 | Unicode-3.0 |  |
| zerocopy | 0.7.35 | BSD-2-Clause OR Apache-2.0 OR MIT | MIT | choice |
| zerofrom | 0.1.5 | Unicode-3.0 | Unicode-3.0 |  |
| zerovec | 0.10.4 | Unicode-3.0 | Unicode-3.0 |  |

Plus the **Rust standard library** (core/alloc/std, dlmalloc) linked into the .wasm: MIT OR Apache-2.0, elected MIT.

## Build-time-only Rust crates (proc-macros and their deps; no code in the .wasm)

These are listed for completeness. They have no notice obligation for the app, but all are MIT/Apache/Unicode anyway.

| Name | Version | Declared license | Elected | Notes |
|---|---|---|---|---|
| bumpalo | 3.16.0 | MIT OR Apache-2.0 | MIT | choice; dep of proc-macro/build |
| displaydoc | 0.2.5 | MIT OR Apache-2.0 | MIT | choice; proc-macro |
| icu_provider_macros | 1.5.0 | Unicode-3.0 | Unicode-3.0 | proc-macro |
| log | 0.4.22 | MIT OR Apache-2.0 | MIT | choice; dep of proc-macro/build |
| proc-macro2 | 1.0.92 | MIT OR Apache-2.0 | MIT | choice; dep of proc-macro/build |
| quote | 1.0.37 | MIT OR Apache-2.0 | MIT | choice; dep of proc-macro/build |
| schemars_derive | 0.8.21 | MIT | MIT | proc-macro |
| serde_derive | 1.0.215 | MIT OR Apache-2.0 | MIT | choice; proc-macro |
| serde_derive_internals | 0.29.1 | MIT OR Apache-2.0 | MIT | choice; dep of proc-macro/build |
| slog_derive | 0.2.0 | MIT/Apache-2.0 | MIT | choice; proc-macro |
| syn | 1.0.109 | MIT OR Apache-2.0 | MIT | choice; dep of proc-macro/build |
| syn | 2.0.90 | MIT OR Apache-2.0 | MIT | choice; dep of proc-macro/build |
| synstructure | 0.13.1 | MIT | MIT | dep of proc-macro/build |
| thiserror-core-impl | 1.0.50 | MIT OR Apache-2.0 | MIT | choice; proc-macro |
| thiserror-impl | 1.0.69 | MIT OR Apache-2.0 | MIT | choice; proc-macro |
| unicode-ident | 1.0.14 | (MIT OR Apache-2.0) AND Unicode-3.0 | MIT AND Unicode-3.0 | choice; dep of proc-macro/build |
| wasm-bindgen-backend | 0.2.97 | MIT OR Apache-2.0 | MIT | choice; dep of proc-macro/build |
| wasm-bindgen-macro | 0.2.97 | MIT OR Apache-2.0 | MIT | choice; proc-macro |
| wasm-bindgen-macro-support | 0.2.97 | MIT OR Apache-2.0 | MIT | choice; dep of proc-macro/build |
| wasm-bindgen-shared | 0.2.97 | MIT OR Apache-2.0 | MIT | choice; dep of proc-macro/build |
| yoke-derive | 0.7.5 | Unicode-3.0 | Unicode-3.0 | proc-macro |
| zerocopy-derive | 0.7.35 | BSD-2-Clause OR Apache-2.0 OR MIT | MIT | choice; proc-macro |
| zerofrom-derive | 0.1.5 | Unicode-3.0 | Unicode-3.0 | proc-macro |
| zerovec-derive | 0.10.3 | Unicode-3.0 | Unicode-3.0 | proc-macro |

## Shipped npm packages (from webpack source maps)

The module count is the number of source files from that package in the bundle. "Nested copy" means a second version resolved under another package's node_modules. The webpack runtime bootstrap (MIT) is also in the bundle. Declared-but-not-bundled production deps: @sentry/tracing (MIT, unused), react-is 19 (MIT; the bundled copy is 16.13.1 via hoist-non-react-statics/prop-types), stylis 4.3.6 top-level (bundled copies are 4.2.0/4.3.2 nested). The build-only tools are jest, ts-jest, typescript, ts-loader, json-schema-to-typescript and @types/*.

| Name | Version | Declared license | Elected | Notes |
|---|---|---|---|---|
| @babel/runtime | 7.27.0 | MIT | MIT | 28 module(s) |
| @emotion/cache | 11.14.0 | MIT | MIT | 1 module(s) |
| @emotion/hash | 0.9.2 | MIT | MIT | 1 module(s) |
| @emotion/memoize | 0.9.0 | MIT | MIT | 1 module(s) |
| @emotion/react | 11.14.0 | MIT | MIT | 2 module(s) |
| @emotion/serialize | 1.3.3 | MIT | MIT | 1 module(s) |
| @emotion/sheet | 1.4.0 | MIT | MIT | 1 module(s) |
| @emotion/unitless | 0.10.0 | MIT | MIT | 1 module(s), nested copy |
| @emotion/unitless | 0.8.1 | MIT | MIT | 1 module(s) |
| @emotion/use-insertion-effect-with-fallbacks | 1.2.0 | MIT | MIT | 1 module(s) |
| @emotion/utils | 1.4.2 | MIT | MIT | 1 module(s) |
| @floating-ui/core | 1.6.9 | MIT | MIT | 1 module(s) |
| @floating-ui/dom | 1.6.13 | MIT | MIT | 1 module(s) |
| @floating-ui/utils | 0.2.9 | MIT | MIT | 2 module(s) |
| @icons/material | 0.2.4 | MIT | MIT | 2 module(s) |
| @sentry/browser | 9.13.0 | MIT | MIT | 3 module(s) |
| @sentry/core | 9.13.0 | MIT | MIT | 23 module(s) |
| @sentry/react | 9.13.0 | MIT | MIT | 3 module(s) |
| canvas-confetti | 1.9.3 | ISC | ISC | 1 module(s) |
| classnames | 2.5.1 | MIT | MIT | 1 module(s) |
| emoji-picker-react | 4.12.2 | MIT | MIT | 1 module(s) |
| exenv | 1.2.2 | BSD-3-Clause | BSD-3-Clause | 1 module(s) |
| flairup | 1.0.0 | MIT | MIT | 1 module(s) |
| hoist-non-react-statics | 3.3.2 | BSD-3-Clause | BSD-3-Clause | 1 module(s) |
| lodash | 4.17.21 | MIT | MIT | 157 module(s) |
| lodash-es | 4.17.21 | MIT | MIT | 162 module(s) |
| material-colors | 1.2.6 | ISC | ISC | 1 module(s) |
| memoize-one | 6.0.0 | MIT | MIT | 1 module(s) |
| prop-types | 15.8.1 | MIT | MIT | 3 module(s) |
| react | 19.1.0 | MIT | MIT | 2 module(s) |
| react-color | 2.19.3 | MIT | MIT | 55 module(s) |
| react-dom | 19.1.0 | MIT | MIT | 4 module(s) |
| react-is | 16.13.1 | MIT | MIT | 2 module(s), nested copy |
| react-lifecycles-compat | 3.0.4 | MIT | MIT | 1 module(s) |
| react-modal | 3.16.3 | MIT | MIT | 11 module(s) |
| react-select | 5.10.1 | MIT | MIT | 4 module(s) |
| react-tooltip | 5.28.1 | MIT | MIT | 1 module(s) |
| reactcss | 1.2.3 | MIT | MIT | 7 module(s) |
| scheduler | 0.26.0 | MIT | MIT | 2 module(s) |
| shallowequal | 1.1.0 | MIT | MIT | 1 module(s) |
| styled-components | 6.1.17 | MIT | MIT | 1 module(s) |
| stylis | 4.2.0 | MIT | MIT | 6 module(s), nested copy |
| stylis | 4.3.2 | MIT | MIT | 7 module(s), nested copy |
| tinycolor2 | 1.6.0 | MIT | MIT | 1 module(s) |
| tslib | 2.6.2 | 0BSD | 0BSD | 1 module(s), nested copy |
| use-isomorphic-layout-effect | 1.2.0 | MIT | MIT | 1 module(s) |
| warning | 4.0.3 | MIT | MIT | 1 module(s) |
