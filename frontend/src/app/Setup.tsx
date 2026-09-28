import * as React from "react";
import { AppStateContext } from "../AppStateProvider";
import { WebsocketContext } from "../WebsocketProvider";
import Initialize from "../Initialize";
import { InitializePhase } from "../gen-types";
import { DIFFICULTIES, botLevel } from "./cards";

import type { JSX } from "react";

type Mode = "tractor" | "friends";
type Preset = "Pagat" | "Standard" | "Beijing" | "Custom";

const LEVEL_KEY = "bot_difficulty";

const PRESETS: { [k in Preset]: string } = {
  Pagat:
    "The rules from pagat.com · failed throws cost 10 points per card taken back · the declarer stays on after winning · defenders go up at 60 points per deck (40–59: nobody goes up)",
  Standard:
    "Kitty of 8 · pairs must follow pairs · failed throws are shown, not penalized · defenders need 40 points per deck to take over",
  Beijing:
    "Longer sets are protected (a triple doesn't have to break for a pair) · levels 5, 10 and K must be won as landlord",
  Custom:
    "Adjust every rule yourself: kitty size, throws, friend calls, bidding, scoring and more.",
};

interface IProps {
  phase: InitializePhase;
  me: number;
  onBack: () => void;
}

/** Presets offered for each game; the first is the default. */
const PRESETS_FOR: { [m in Mode]: Preset[] } = {
  friends: ["Pagat", "Standard", "Beijing", "Custom"],
  tractor: ["Standard", "Beijing", "Custom"],
};

/** Pagat: 2 decks for 5–7 players, 3 for 8–11, 4 for 12. */
const pagatDecks = (players: number): number =>
  players <= 7 ? 2 : players <= 11 ? 3 : 4;

/** A kitty of 6–8 cards, choosing the size that needs the fewest cards removed to deal evenly. */
const pagatKitty = (players: number, decks: number): number => {
  const total = 54 * decks;
  return [8, 7, 6].reduce((best, k) =>
    (total - k) % players < (total - best) % players ? k : best,
  );
};

const loadLevel = (): string => {
  try {
    const v = window.localStorage.getItem(LEVEL_KEY);
    if (DIFFICULTIES.some((d) => d.value === v)) {
      return v as string;
    }
  } catch {
    // Use the default.
  }
  return "Intermediate";
};

const Setup = (props: IProps): JSX.Element => {
  const { state } = React.useContext(AppStateContext);
  const { send } = React.useContext(WebsocketContext);
  const [mode, setMode] = React.useState<Mode>(
    props.phase.propagated.game_mode === "Tractor" ? "tractor" : "friends",
  );
  const [players, setPlayers] = React.useState<number>(5);
  const [level, setLevel] = React.useState<string>(loadLevel);
  const [preset, setPreset] = React.useState<Preset>(
    PRESETS_FOR[
      props.phase.propagated.game_mode === "Tractor" ? "tractor" : "friends"
    ][0],
  );
  const [customOpen, setCustomOpen] = React.useState(false);

  const chooseMode = (m: Mode): void => {
    setMode(m);
    setPreset(PRESETS_FOR[m][0]);
  };

  const seats = mode === "tractor" ? 4 : players;
  const decks =
    mode === "tractor"
      ? 2
      : preset === "Pagat"
        ? pagatDecks(players)
        : players >= 7
          ? 4
          : 3;
  const kitty = preset === "Pagat" ? pagatKitty(seats, decks) : null;

  const chooseLevel = (v: string): void => {
    setLevel(v);
    try {
      window.localStorage.setItem(LEVEL_KEY, v);
    } catch {
      // Not remembering the choice is fine.
    }
  };

  const deal = (): void => {
    const current = props.phase.propagated.players;
    // Replace the computer players so the table has exactly the chosen number and level.
    current
      .filter((p) => p.id !== props.me && botLevel(p.name) !== null)
      .forEach((p) => send({ Kick: p.id }));
    const humans = current.filter((p) => botLevel(p.name) === null).length;
    for (let i = 0; i < Math.max(0, seats - humans); i++) {
      send({ AddBot: level });
    }
    send({
      Action: {
        SetGameMode:
          mode === "tractor"
            ? "Tractor"
            : { FindingFriends: { num_friends: null } },
      },
    });
    send({ Action: { SetNumDecks: decks } });
    if (preset !== "Custom") {
      send({ Action: { SetKittySize: kitty } });
      send({
        Action: {
          SetThrowPenalty: preset === "Pagat" ? "TenPointsPerCard" : "None",
        },
      });
      send({
        Action: {
          SetNextLandlordPolicy:
            preset === "Pagat" ? "LandlordStaysIfWon" : "NextOnWinningSide",
        },
      });
      send({
        Action: {
          SetTrickDrawPolicy:
            preset === "Beijing" ? "LongerTuplesProtected" : "NoProtections",
        },
      });
      send({
        Action: {
          SetAdvancementPolicy:
            preset === "Beijing" ? "DefendPoints" : "Unrestricted",
        },
      });
    }
    send({ Action: "StartGame" });
  };

  const note = DIFFICULTIES.find((d) => d.value === level)?.note ?? "";

  return (
    <div className="tta-screen">
      <div className="tta-topbar">
        <button
          type="button"
          className="tta-round"
          aria-label="Back to home"
          onClick={props.onBack}
        >
          ‹
        </button>
        <h1 className="tta-title">New game</h1>
      </div>

      <div className="tta-section">
        <div className="tta-label">Game</div>
        <div className="tta-modes">
          <button
            type="button"
            className="tta-mode"
            aria-pressed={mode === "tractor"}
            onClick={() => chooseMode("tractor")}
          >
            <strong>Tractor</strong>
            <span className="tta-zh" lang="zh">
              升级
            </span>
            <span>4 players · partners across</span>
          </button>
          <button
            type="button"
            className="tta-mode"
            aria-pressed={mode === "friends"}
            onClick={() => chooseMode("friends")}
          >
            <strong>Finding Friends</strong>
            <span className="tta-zh" lang="zh">
              找朋友
            </span>
            <span>5–12 players · secret partners</span>
          </button>
        </div>
      </div>

      {mode === "friends" && (
        <div className="tta-section">
          <div className="tta-label">Players</div>
          <div className="tta-chips">
            {[5, 6, 7, 8, 9, 10, 11, 12].map((n) => (
              <button
                key={n}
                type="button"
                className="tta-chip"
                aria-pressed={players === n}
                onClick={() => setPlayers(n)}
              >
                {n}
              </button>
            ))}
          </div>
        </div>
      )}

      <div className="tta-section">
        <div className="tta-label">Computer players</div>
        <div className="tta-chips">
          {DIFFICULTIES.map((d) => (
            <button
              key={d.value}
              type="button"
              className="tta-chip"
              aria-pressed={level === d.value}
              onClick={() => chooseLevel(d.value)}
            >
              {d.label}
            </button>
          ))}
        </div>
        <div className="tta-note">{note}</div>
      </div>

      <div className="tta-section">
        <div className="tta-label">House rules</div>
        <div className="tta-chips">
          {PRESETS_FOR[mode].map((p) => (
            <button
              key={p}
              type="button"
              className="tta-chip"
              aria-pressed={preset === p}
              onClick={() => {
                setPreset(p);
                if (p === "Custom") {
                  setCustomOpen(true);
                }
              }}
            >
              {p}
            </button>
          ))}
        </div>
        <div className="tta-panel">
          {decks} decks{kitty !== null ? ` · kitty of ${kitty}` : ""} ·{" "}
          {PRESETS[preset]}
          {preset === "Custom" && (
            <>
              {" "}
              <button
                type="button"
                className="tta-chip"
                onClick={() => setCustomOpen(true)}
              >
                Edit rules
              </button>
            </>
          )}
        </div>
      </div>

      <button
        type="button"
        className="tta-btn primary tta-bottom"
        onClick={deal}
      >
        Deal
      </button>

      {customOpen && (
        <div
          className="tta-sheet-backdrop"
          onClick={() => setCustomOpen(false)}
        >
          <div
            className="tta-sheet"
            role="dialog"
            aria-label="All rules"
            onClick={(e) => e.stopPropagation()}
          >
            <div
              className="tta-topbar"
              style={{ justifyContent: "space-between" }}
            >
              <h2 className="tta-title">All rules</h2>
              <button
                type="button"
                className="tta-chip"
                onClick={() => setCustomOpen(false)}
              >
                Done
              </button>
            </div>
            <p className="tta-note">
              Changes apply as you make them. Game mode, decks and players still
              come from the previous screen when you tap Deal.
            </p>
            <div className="tta-classic">
              <Initialize state={props.phase} name={state.name} />
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default Setup;
