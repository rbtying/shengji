# Cloud self-play run

Instructions for a cloud Claude Code session. Safe to rerun: every phase tops up what is already
in `selfplay/results/` to a target, so a rerun only adds what is missing.

Be frugal with turns: start long jobs in the background with nohup and check on them rarely
(about every 25 minutes, with a background wait that watches the job's PID; plain `sleep` in the
foreground is blocked). Write logs outside the repository (e.g. `$HOME/h2h.log`) so they don't
show up as untracked files. Stop within about 8 hours of wall-clock; if time runs short, go
straight to the report with the data you have.

## Setup
1. Clone https://github.com/cisapro-bot/shengji if this session started on a different repo, then
   check out branch `selfplay-results` if it exists on origin (it carries the latest results and
   tools); otherwise `computer-players`, and create `selfplay-results` from it.
2. Rust toolchain (install with rustup if `cargo` is missing) and python3.
3. `cargo build --release -p selfplay` — only this crate; don't build the frontend or wasm.
4. Note `nproc`; keep the default `--jobs`. Results go in `selfplay/results/`; output files are
   appended to, never overwritten. Count what is already there before each run:
   `wc -l selfplay/results/*.jsonl` (one line per hand).
5. Topping up mixes hands from different sessions, which is only valid if the bots haven't
   changed since. If this prints any commits, move every `selfplay/results/*.jsonl` to
   `selfplay/results/archive/` (with `git mv`, names prefixed with the date as in Phase 2) and
   start afresh:
   `git log --oneline $(git log -1 --format=%H -- 'selfplay/results/*.jsonl')..HEAD -- core/src/bot`

## Phase 1 — does the smarter rollout policy play better?
Top the head-to-head up to 300 hands in total, appending to the same file:
```bash
F=selfplay/results/h2h-hard.jsonl
NEED=$(( 300 - $(cat "$F" 2>/dev/null | wc -l) ))
[ "$NEED" -gt 0 ] && nohup ./target/release/selfplay h2h --bots Hard --hands "$NEED" --out "$F" > "$HOME/h2h.log" 2>&1 &
```
If `NEED` is 0 or less, skip straight to the analysis. Otherwise, after ~10 minutes estimate the
total time from the log; if it would take more than ~3 hours, stop it once the file has at least
150 lines. Then:
`python3 selfplay/analyze.py selfplay/results/h2h-hard.jsonl`
SMART=true if "smart minus basic" is positive and more than 2 × its standard error, else false.

## Phase 2 — coach calibration with the shipping configuration
All calibration files must share one configuration: `analyze.py` exits with an error listing
the files per configuration if they don't. Before
touching a file, check that no hand in it was played with the other setting of SMART, and move
it aside if one was (`selfplay/results/archive/` is outside the analysis glob):
```bash
OTHER=$([ "$SMART" = true ] && echo false || echo true)
F=selfplay/results/cal-<bots>-<mode>.jsonl
if [ -f "$F" ] && grep -q "\"coach_smart\":$OTHER\|\"bots_smart\":$OTHER" "$F"; then
  mkdir -p selfplay/results/archive && git mv "$F" "selfplay/results/archive/$(date -u +%Y%m%d-%H%M)-$(basename "$F")"
fi
```
Then, with `--coach-smart $SMART --bots-smart $SMART`, top each file up to 120 hands, one run at
a time, most important first (`--hands` is 120 minus the file's current line count; skip files
that already have 120):
1. `--bots Expert --mode tractor`
2. `--bots Hard --mode tractor`
3. `--bots Expert --mode friends5`
4. `--bots Hard --mode friends5`
5. `--bots Intermediate --mode tractor`
6. `--bots Intermediate --mode friends5`
7. `--bots Easy --mode tractor`
8. `--bots Easy --mode friends5`

Every file needs all 120 hands: each side of a table gets only about half of them, and
`analyze.py` leaves a side with fewer than 30 hands out of FITS. Time the first run and budget
the rest to fit: drop the last runs rather than cutting hands or overrunning. Commit and push
`selfplay/results/` after each run, so a lost container costs at most one run.

## Phase 3 — report
1. `python3 selfplay/analyze.py selfplay/results/*.jsonl > selfplay/results/analysis.txt 2>&1`
   (stderr included, so warnings and any error end up in the file too).
2. Commit `selfplay/results/` to branch `selfplay-results` and push it. Open a draft pull request
   into `computer-players` only if none is open for the branch.
3. Final message: core count; hands added and time per run; the SMART decision with the
   head-to-head section; any files moved to `archive/`; the full `analysis.txt` (it ends with the
   FITS table and a warning for any table left out); whether the push worked.

If something fails unexpectedly (build error, the tool crashing), report the exact error rather
than attempting large fixes. Don't change code outside `selfplay/results/`.
