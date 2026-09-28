#!/usr/bin/env python3
"""Summarize selfplay output: head-to-head results and the coach's calibration fits.

Usage: python3 selfplay/analyze.py results/*.jsonl

Prints the smart-vs-basic rollout comparison, then for each (opponent level, game, side) the
coach's raw calibration, a logistic fit logit(q) = a + b*logit(p), and the calibration after the
fit, followed by the Rust `FITS` table for core/src/bot/calibration.rs. Pure Python, no packages.
"""
import json
import math
import sys
from collections import defaultdict


def logit(p):
    p = min(max(p, 0.005), 0.995)
    return math.log(p / (1 - p))


def fit(points):
    """Two-parameter logistic regression of won on logit(p), by Newton's method."""
    a, b = 0.0, 1.0
    for _ in range(50):
        ga = gb = haa = hab = hbb = 0.0
        for x, y in points:
            q = 1 / (1 + math.exp(-(a + b * x)))
            w = q * (1 - q)
            ga += y - q
            gb += (y - q) * x
            haa += w
            hab += w * x
            hbb += w * x * x
        det = haa * hbb - hab * hab
        if det <= 1e-12:
            break
        da = (hbb * ga - hab * gb) / det
        db = (haa * gb - hab * ga) / det
        a += da
        b += db
        if abs(da) + abs(db) < 1e-9:
            break
    return a, b


def bins(pairs, n=5):
    out = []
    for i in range(n):
        lo, hi = i / n, (i + 1) / n
        sel = [(p, y) for p, y in pairs if lo <= p < hi or (i == n - 1 and p == 1.0)]
        if sel:
            out.append(
                f"{lo:.1f}-{hi:.1f}: predicted {sum(p for p, _ in sel) / len(sel):.2f}, "
                f"actual {sum(y for _, y in sel) / len(sel):.2f} (n={len(sel)})"
            )
    return out


def main(paths):
    h2h = []
    cal = defaultdict(list)  # (bots, friends, side) -> [(p, won, hand_id)]
    hands = defaultdict(int)
    for path in paths:
        with open(path, encoding="utf-8") as f:
            for n, line in enumerate(f):
                line = line.strip()
                if not line:
                    continue
                r = json.loads(line)
                if r["kind"] == "h2h":
                    h2h.append(r)
                elif r["kind"] == "calibrate":
                    friends = r["mode"].startswith("friends")
                    hands[(r["bots"], friends)] += 1
                    for pr in r["predictions"]:
                        key = (r["bots"], friends, pr["landlord_side"])
                        cal[key].append((pr["p"], 1 if r["won"] else 0, f"{path}:{n}"))

    if h2h:
        diffs = [r["smart_levels"] - r["basic_levels"] for r in h2h]
        m = sum(diffs) / len(diffs)
        sd = math.sqrt(sum((d - m) ** 2 for d in diffs) / max(1, len(diffs) - 1))
        se = sd / math.sqrt(len(diffs))
        print(f"== Head to head ({len(h2h)} hands, {h2h[0]['bots']} bots)")
        print(f"smart minus basic: {m:+.3f} levels per hand (standard error {se:.3f})")
        for side in (True, False):
            sel = [r for r in h2h if r["smart_landlord"] == side]
            if sel:
                wins = sum(1 for r in sel if r["landlord_won"] == side)
                print(
                    f"  smart team as {'landlord side' if side else 'attackers'}: "
                    f"won {wins}/{len(sel)} ({wins / len(sel):.0%})"
                )
        print()

    rust = []
    for key in sorted(cal):
        bots, friends, side = key
        pts = cal[key]
        game = "Finding Friends" if friends else "Tractor"
        who = "landlord side" if side else "attacking"
        print(f"== {bots} bots, {game}, {who}: {len(pts)} predictions "
              f"from {hands[(bots, friends)]} hands")
        raw = [(p, y) for p, y, _ in pts]
        mp = sum(p for p, _ in raw) / len(raw)
        my = sum(y for _, y in raw) / len(raw)
        print(f"  raw: predicted {mp:.2f}, actual {my:.2f}")
        for line in bins(raw):
            print("   ", line)
        a, b = fit([(logit(p), y) for p, y in raw])
        adj = [(1 / (1 + math.exp(-(a + b * logit(p)))), y) for p, y in raw]
        print(f"  fit: a={a:+.3f} b={b:.3f}; after: predicted "
              f"{sum(p for p, _ in adj) / len(adj):.2f}")
        for line in bins(adj):
            print("   ", line)
        if hands[(bots, friends)] >= 60:
            rust.append(
                f"    (Difficulty::{bots}, {str(friends).lower()}, {str(side).lower()}, "
                f"{a:.4f}, {b:.4f}),"
            )
        print()

    if rust:
        print("== FITS for core/src/bot/calibration.rs (tables with 60+ hands)")
        print("const FITS: &[(Difficulty, bool, bool, f64, f64)] = &[")
        print("\n".join(rust))
        print("];")


if __name__ == "__main__":
    main(sys.argv[1:])
