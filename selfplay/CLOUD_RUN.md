# Cloud self-play run

Instructions for a cloud Claude Code session. Be frugal with turns: start long jobs in the
background with nohup and check on them rarely (`sleep 1500` between checks). Stop within about
8 hours of wall-clock; if time runs short, go straight to the report with the data you have.

## Setup
1. Work on branch `computer-players` of https://github.com/cisapro-bot/shengji (clone it if this
   session started on a different repo or branch).
2. Rust toolchain (install with rustup if `cargo` is missing) and python3.
3. `cargo build --release -p selfplay` — only this crate; don't build the frontend or wasm.
4. Note `nproc`; keep the default `--jobs`. Results go in `selfplay/results/` (58 head-to-head
   hands are already there from an earlier run).

## Phase 1 — does the smarter rollout policy play better?
Continue the head-to-head to 300 hands in total (appending to the same file):
`nohup ./target/release/selfplay h2h --bots Hard --hands 242 --out selfplay/results/h2h-hard.jsonl > h2h.log 2>&1 &`
After ~10 minutes, estimate the total time from `h2h.log`. If it would take more than ~3 hours,
stop it once the file has at least 150 lines. Then:
`python3 selfplay/analyze.py selfplay/results/h2h-hard.jsonl`
SMART=true if "smart minus basic" is positive and more than 2 × its standard error, else false.

## Phase 2 — coach calibration with the shipping configuration
With `--coach-smart $SMART --bots-smart $SMART`, run one at a time, most important first,
`--hands 80` each, output `selfplay/results/cal-<bots>-<mode>.jsonl`:
1. `--bots Expert --mode tractor`
2. `--bots Hard --mode tractor`
3. `--bots Expert --mode friends5`
4. `--bots Hard --mode friends5`
5. `--bots Intermediate --mode tractor`
6. `--bots Intermediate --mode friends5`
7. `--bots Easy --mode tractor`
8. `--bots Easy --mode friends5`

Time the first run and budget the rest to fit: cut hands per run (never below 60) or drop the
last runs rather than overrunning.

## Phase 3 — report
1. `python3 selfplay/analyze.py selfplay/results/*.jsonl > selfplay/results/analysis.txt`
2. Commit `selfplay/results/` to a new branch `selfplay-results` and push it.
3. Final message: core count; hands and time per run; the SMART decision with the head-to-head
   section; the full `analysis.txt` (it ends with the FITS table); whether the push worked.

If something fails unexpectedly (build error, the tool crashing), report the exact error rather
than attempting large fixes. Don't change code outside `selfplay/results/`.
