# selfplay

Bot self-play for tuning the coach and the bots, outside the app.

```bash
cargo build --release -p selfplay
# Smart vs basic rollouts: four bots of one level, Tractor, teams swap seats every hand.
./target/release/selfplay h2h --bots Hard --hands 300 --out results/h2h-hard.jsonl
# Coach calibration: seat 0 follows the coach against bots of one level.
./target/release/selfplay calibrate --bots Expert --mode tractor --hands 120 \
    --coach-smart true --bots-smart true --out results/cal-expert-tractor.jsonl
python3 selfplay/analyze.py results/*.jsonl
```

Options: `--bots Beginner|Easy|Intermediate|Hard|Expert`, `--mode tractor|friends5|friends6`
(the app's Pagat preset), `--jobs N` hands at once (default: cores / 4; the bots' simulation
threads share the rest), `--coach-smart` / `--bots-smart` (rollout policy, default false).

Simulations use a fixed number of samples per decision (no wall-clock cut-off), so results don't
depend on the machine. Output is appended one JSON line per hand and flushed, so a run can be
stopped and restarted, and files from several runs can be analyzed together.

`analyze.py` prints the head-to-head result (levels per hand, smart minus basic, with standard
error) and, per opponent level, game and side, how the coach's predicted win chance compares
with the real outcome, a logistic fit that corrects it, and the `FITS` table to paste into
`core/src/bot/calibration.rs`.

The fit only applies to the configuration it was measured with: rerun calibration after
changing the rollout policy, the sample counts or the bots.
