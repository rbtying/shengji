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


def sigmoid(z):
    if z >= 0:
        return 1 / (1 + math.exp(-z))
    e = math.exp(z)
    return e / (1 + e)


def log_likelihood(points, a, b):
    total = 0.0
    for x, y in points:
        q = min(max(sigmoid(a + b * x), 1e-300), 1 - 1e-16)
        total += y * math.log(q) + (1 - y) * math.log(1 - q)
    return total


def fit(points):
    """Two-parameter logistic regression of won on logit(p), by Newton's method.

    Steps that would lower the likelihood are halved: full Newton steps can oscillate and
    diverge on this data (they did on Hard/Tractor/attacking)."""
    a, b = 0.0, 1.0
    for _ in range(200):
        ga = gb = haa = hab = hbb = 0.0
        for x, y in points:
            q = sigmoid(a + b * x)
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
        base = log_likelihood(points, a, b)
        t = 1.0
        while t > 1e-8 and log_likelihood(points, a + t * da, b + t * db) < base:
            t /= 2
        a += t * da
        b += t * db
        if abs(t * da) + abs(t * db) < 1e-9:
            break
    return a, b


# Hands a table needs, on its own side, to go into FITS.
MIN_HANDS = 30

# Real fits stay well inside this; separable data drives the coefficients toward infinity.
MAX_COEF = 10.0


def degenerate(points, a, b):
    """Why the fit of `points` to (a, b) can't be trusted, or None if it can."""
    outcomes = {y for _, y in points}
    if len(outcomes) < 2:
        return f"every hand was {'won' if 1 in outcomes else 'lost'}, nothing to fit"
    if abs(a) > MAX_COEF or abs(b) > MAX_COEF:
        return (f"no finite fit (a={a:+.1f} b={b:.1f}): the predictions separate wins from "
                f"losses perfectly; add hands")
    return None


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
    configs = defaultdict(set)  # (coach_smart, bots_smart) -> files
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
                    configs[(r.get("coach_smart"), r.get("bots_smart"))].add(path)
                    for pr in r["predictions"]:
                        key = (r["bots"], friends, pr["landlord_side"])
                        cal[key].append((pr["p"], 1 if r["won"] else 0, f"{path}:{n}"))

    # FITS describes one configuration; fitting hands from several together would be wrong.
    if len(configs) > 1:
        print("error: calibration hands from different configurations; analyze them "
              "separately:", file=sys.stderr)
        for (coach, bots), files in sorted(configs.items(), key=str):
            print(f"  coach_smart={str(coach).lower()} bots_smart={str(bots).lower()}: "
                  f"{', '.join(sorted(files))}",
                  file=sys.stderr)
        sys.exit(1)

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
    skipped = []
    for key in sorted(cal):
        bots, friends, side = key
        pts = cal[key]
        game = "Finding Friends" if friends else "Tractor"
        who = "landlord side" if side else "attacking"
        # A Finding Friends hand can count on both sides: seat 0 may switch when a friend shows.
        hands = len({hand for _, _, hand in pts})
        print(f"== {bots} bots, {game}, {who}: {len(pts)} predictions from {hands} hands")
        raw = [(p, y) for p, y, _ in pts]
        mp = sum(p for p, _ in raw) / len(raw)
        my = sum(y for _, y in raw) / len(raw)
        print(f"  raw: predicted {mp:.2f}, actual {my:.2f}")
        for line in bins(raw):
            print("   ", line)
        points = [(logit(p), y) for p, y in raw]
        a, b = fit(points)
        reason = degenerate(points, a, b)
        if reason:
            print(f"  fit: skipped, {reason}")
            skipped.append(f"{bots} bots, {game}, {who}: {reason}")
            print()
            continue
        adj = [(sigmoid(a + b * logit(p)), y) for p, y in raw]
        print(f"  fit: a={a:+.3f} b={b:.3f}; after: predicted "
              f"{sum(p for p, _ in adj) / len(adj):.2f}")
        for line in bins(adj):
            print("   ", line)
        if hands >= MIN_HANDS:
            rust.append(
                f"    (Difficulty::{bots}, {str(friends).lower()}, {str(side).lower()}, "
                f"{a:.4f}, {b:.4f}),"
            )
        print()

    if rust:
        (coach, bots), = configs
        print(f"== FITS for core/src/bot/calibration.rs (tables with {MIN_HANDS}+ hands; "
              f"coach_smart={str(coach).lower()} bots_smart={str(bots).lower()})")
        print("const FITS: &[(Difficulty, bool, bool, f64, f64)] = &[")
        print("\n".join(rust))
        print("];")
    if skipped:
        print(f"\nWARNING: {len(skipped)} table(s) left out of FITS:", file=sys.stderr)
        for s in skipped:
            print(f"  {s}", file=sys.stderr)


if __name__ == "__main__":
    main(sys.argv[1:])
