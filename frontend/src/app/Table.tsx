import * as React from "react";
import classNames from "classnames";
import { AppStateContext } from "../AppStateProvider";
import { WebsocketContext } from "../WebsocketProvider";
import { PlayPhase, TrickUnit } from "../gen-types";
import {
  displayName,
  face,
  fromMe,
  handCards,
  pointsIn,
  seatPositions,
  suitSymbol,
  trumpLabel,
} from "./cards";
import HandTray from "./HandTray";
import MiniCard from "./MiniCard";
import { useHandGroups } from "./useHandGroups";

import type { JSX } from "react";

interface IProps {
  phase: PlayPhase;
  me: number;
  onHome: () => void;
}

const unitWidth = (u: TrickUnit): number =>
  "Repeated" in u ? u.Repeated.count : u.Tractor.count;

const unitName = (u: TrickUnit): string => {
  if ("Tractor" in u) {
    return "tractor";
  }
  const n = u.Repeated.count;
  return n === 1
    ? "single"
    : n === 2
      ? "pair"
      : n === 3
        ? "triple"
        : `${n} of a kind`;
};

/** Turn "<card>#<copy>" keys back into cards. */
const cardsOf = (keys: string[]): string[] => keys.map((k) => k.split("#")[0]);

const Table = (props: IProps): JSX.Element => {
  const { state, updateState } = React.useContext(AppStateContext);
  const { send } = React.useContext(WebsocketContext);
  const p = props.phase;
  const me = props.me;
  const players = p.propagated.players;
  const trick = p.trick;
  const hand = handCards(p.hands, me);
  const groups = useHandGroups(hand, p.trump);
  const [selected, setSelected] = React.useState<string[]>([]);
  const [coachFor, setCoachFor] = React.useState<string | null>(null);
  const [toast, setToast] = React.useState<string | null>(null);
  const isFriends = p.game_mode !== "Tractor";
  const friends =
    isFriends && p.game_mode !== "Tractor"
      ? p.game_mode.FindingFriends.friends
      : [];

  const name = (id: number): string =>
    id === me
      ? "You"
      : displayName(players.find((pl) => pl.id === id)?.name ?? "");

  const next = trick.player_queue.length > 0 ? trick.player_queue[0] : null;
  const myTurn = next === me;
  const trickDone =
    trick.player_queue.length === 0 && trick.played_cards.length > 0;
  const trickKey = JSON.stringify([
    p.played_tricks?.length ?? 0,
    trick.played_cards.length,
  ]);

  // A new decision clears the selection and any stale coaching.
  React.useEffect(() => {
    setSelected([]);
  }, [trickKey]);

  // Announce a friend the moment they're revealed.
  const teamSize = p.landlords_team.length;
  const prevTeam = React.useRef<number[]>(p.landlords_team);
  React.useEffect(() => {
    const joined = p.landlords_team.filter(
      (id) => !prevTeam.current.includes(id),
    );
    prevTeam.current = p.landlords_team;
    if (isFriends && joined.length > 0) {
      const who = joined.map(name).join(" and ");
      setToast(
        `${who} ${joined.length > 1 || who === "You" ? "are" : "is"} ${name(p.landlord)}'s friend!`,
      );
      const t = setTimeout(() => setToast(null), 3500);
      return () => clearTimeout(t);
    }
  }, [teamSize]);

  // Scoring: points held by players not (yet) on the landlord's team.
  const defenderPoints = players
    .filter((pl) => !p.landlords_team.includes(pl.id))
    .reduce((s, pl) => s + pointsIn(p.points[pl.id] ?? []), 0);
  const params = p.propagated.game_scoring_parameters!;
  const numDecks = p.decks?.length ?? p.num_decks;
  const threshold =
    params.step_size_per_deck *
    numDecks *
    params.num_steps_to_non_landlord_turnover;
  const totalPoints = 100 * numDecks;
  const iAmLandlordSide = p.landlords_team.includes(me);

  // Seats, starting with you at the bottom.
  const seated = fromMe(players, me);
  const positions = seatPositions(seated.length);
  const played: { [id: number]: string[] } = {};
  trick.played_cards.forEach((pc) => {
    played[pc.id] = pc.cards;
  });
  const allFriendsFound = friends.every((f) => f.player_id != null);
  const role = (
    id: number,
  ): { text: string; tone: "gold" | "teal" | "" } | null => {
    if (id === p.landlord) {
      return { text: "♛ Landlord", tone: "gold" };
    }
    if (!isFriends) {
      const myIdx = players.findIndex((pl) => pl.id === me);
      const idx = players.findIndex((pl) => pl.id === id);
      return id !== me && (idx - myIdx + players.length) % 2 === 0
        ? { text: "Partner", tone: "teal" }
        : null;
    }
    if (p.landlords_team.includes(id)) {
      return { text: "Friend", tone: "gold" };
    }
    if (id === me) {
      return null;
    }
    return allFriendsFound
      ? { text: "Defender", tone: "teal" }
      : { text: "?", tone: "" };
  };

  // Levels for the header.
  const level = (id: number): string =>
    String(players.find((pl) => pl.id === id)?.level ?? "");
  const opponents = players.filter((pl) =>
    isFriends
      ? false
      : (players.findIndex((x) => x.id === pl.id) -
          players.findIndex((x) => x.id === me) +
          players.length) %
          2 ===
        1,
  );
  const partner = seated[2];

  // What's happening, in words.
  const format = trick.trick_format ?? null;
  const leader =
    trick.played_cards.length > 0 ? trick.played_cards[0].id : null;
  let note: string;
  if (trickDone) {
    const pts = pointsIn(trick.played_cards.flatMap((pc) => pc.cards));
    note = `${name(trick.current_winner!)} win${trick.current_winner === me ? "" : "s"} the trick${pts > 0 ? ` · ${pts} points` : ""}. Tap to continue.`;
  } else if (myTurn && format !== null && leader !== null) {
    const suitChar: { [k: string]: string } = {
      Spades: "♤",
      Hearts: "♡",
      Clubs: "♧",
      Diamonds: "♢",
    };
    const suit =
      format.suit === "Trump"
        ? "trump"
        : suitSymbol(suitChar[format.suit] ?? "");
    const units = format.units.map(unitName).join(" + ");
    note = `Your turn. ${name(leader)} led ${units} of ${suit}.`;
  } else if (myTurn) {
    note = "Your lead.";
  } else if (next !== null) {
    note = `${name(next)} is thinking…`;
  } else {
    note = "";
  }

  // Tapping a card: when following a pair (or bigger), tap once to take the whole set.
  const tap = (card: string, key: string): void => {
    if (selected.includes(key)) {
      setSelected(selected.filter((k) => k.split("#")[0] !== card));
      return;
    }
    const need =
      format !== null && trick.played_cards.length > 0
        ? Math.max(...format.units.map(unitWidth))
        : 1;
    const copies = hand.filter((c) => c === card).length;
    if (myTurn && need >= 2 && copies >= need) {
      const keys = Array.from({ length: need }, (_, i) => `${card}#${i + 1}`);
      setSelected([...selected.filter((k) => !keys.includes(k)), ...keys]);
    } else {
      setSelected([...selected, key]);
    }
  };

  const play = (): void => {
    send({ Action: { PlayCards: cardsOf(selected) } });
  };

  const askCoach = (): void => {
    setCoachFor(trickKey);
    updateState({ hint: null, hintPending: true });
    send("RequestHint");
  };
  const coachOpen = coachFor === trickKey && myTurn;
  const hint = coachOpen ? state.hint : null;
  const hintKeys = (cards: string[]): string[] => {
    const seen: { [c: string]: number } = {};
    return cards.map((c) => {
      seen[c] = (seen[c] ?? 0) + 1;
      return `${c}#${seen[c]}`;
    });
  };

  const trumpText = trumpLabel(p.trump);

  return (
    <div className="tta-table">
      <div className="tta-hud">
        <button
          type="button"
          className="tta-round"
          aria-label="Home"
          onClick={props.onHome}
        >
          ‹
        </button>
        <div className="tta-trump">
          <span className="tta-label">Trump</span>
          <b className={classNames({ red: trumpText.red })}>{trumpText.text}</b>
        </div>
        <div className="tta-levels">
          {isFriends ? (
            <>
              <span className="tta-level ours">
                <i>{level(me)}</i>You
              </span>
              <span className="tta-level">
                <i>{level(p.landlord)}</i>
                {name(p.landlord)} ♛
              </span>
            </>
          ) : (
            <>
              <span className="tta-level ours">
                <i>{level(me)}</i>You &amp; {name(partner.id)}
              </span>
              <span className="tta-level">
                <i>{opponents.length > 0 ? level(opponents[0].id) : ""}</i>
                {opponents.map((o) => name(o.id)).join(" & ")}
              </span>
            </>
          )}
        </div>
      </div>

      <div className="tta-score">
        <div className="tta-score-text">
          <span>
            <strong>
              {iAmLandlordSide ? "Opponents" : "Your side"}: {defenderPoints}
            </strong>{" "}
            of {threshold} to take over
          </span>
          <span>{hand.length} left</span>
        </div>
        <div className="tta-bar" aria-hidden="true">
          <span
            style={{
              width: `${Math.min(100, (100 * defenderPoints) / totalPoints)}%`,
            }}
          />
          <em style={{ left: `${(100 * threshold) / totalPoints}%` }} />
        </div>
      </div>

      <div
        className="tta-felt"
        onClick={() => {
          if (trickDone) {
            send({ Action: "EndTrick" });
          }
        }}
      >
        {seated.map((pl, i) => {
          const pos = positions[i];
          const r = role(pl.id);
          return (
            <div
              key={pl.id}
              className={classNames("tta-seat", {
                next: next === pl.id,
                winning:
                  trick.current_winner === pl.id &&
                  trick.played_cards.length > 1,
              })}
              style={{ left: `${pos.x}%`, top: `${pos.y}%` }}
            >
              {i !== 0 && (
                <span className="tta-seat-name">
                  {name(pl.id)}
                  {r !== null && (
                    <span className={classNames("tta-badge", r.tone)}>
                      {r.text}
                    </span>
                  )}
                </span>
              )}
              <div className="tta-played">
                {(played[pl.id] ?? []).map((c, j) => (
                  <MiniCard key={j} card={c} small />
                ))}
              </div>
              {i === 0 && (
                <span className="tta-seat-name">
                  You
                  {r !== null && (
                    <span className={classNames("tta-badge", r.tone)}>
                      {r.text}
                    </span>
                  )}
                </span>
              )}
            </div>
          );
        })}
        {!coachOpen && <div className="tta-felt-note">{note}</div>}
        {toast !== null && (
          <div className="tta-toast" role="status">
            {toast}
          </div>
        )}
        {isFriends && !allFriendsFound && (
          <div
            className="tta-badge gold"
            style={{ position: "absolute", left: 10, top: 10 }}
          >
            Called:{" "}
            {friends
              .filter((f) => f.player_id == null)
              .map(
                (f) =>
                  `${["first", "second", "third", "fourth"][f.initial_skip]} ${face(f.card).rank}${face(f.card).suit}`,
              )
              .join(", ")}
          </div>
        )}
        {coachOpen && (
          <div
            className="tta-coach"
            role="dialog"
            aria-label="Coach"
            onClick={(e) => e.stopPropagation()}
          >
            <header>
              <span>Coach</span>
              <button
                type="button"
                aria-label="Close coach"
                onClick={() => setCoachFor(null)}
              >
                ×
              </button>
            </header>
            {state.hintPending || hint === null ? (
              <p>
                {state.hintPending
                  ? "Thinking through possible deals…"
                  : "No suggestion right now."}
              </p>
            ) : (
              <>
                <p>{hint.summary}</p>
                {hint.options.length > 1 && (
                  <table>
                    <tbody>
                      {hint.options.map((o, i) => (
                        <tr key={i}>
                          <td>
                            {(o.cards as string[])
                              .map(
                                (c) =>
                                  `${face(c).rank}${face(c).rank === "★" ? "" : face(c).suit}`,
                              )
                              .join(" ")}
                          </td>
                          <td>
                            {i === 0 ? (
                              <strong>{Math.round(o.win_rate * 100)}%</strong>
                            ) : (
                              `${Math.round(o.win_rate * 100)}%`
                            )}{" "}
                            your side wins
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                )}
                <button
                  type="button"
                  className="tta-btn felt"
                  style={{ minHeight: 44 }}
                  onClick={() => {
                    setSelected(hintKeys(hint.cards as string[]));
                    setCoachFor(null);
                  }}
                >
                  Select these cards
                </button>
              </>
            )}
          </div>
        )}
      </div>

      <div className="tta-actions">
        <button
          type="button"
          className="tta-btn outline"
          onClick={askCoach}
          disabled={!myTurn || state.hintPending}
        >
          Hint
        </button>
        <button
          type="button"
          className="tta-btn"
          onClick={() => setSelected([])}
          disabled={selected.length === 0}
        >
          Clear
        </button>
        <button
          type="button"
          className="tta-btn primary grow"
          onClick={play}
          disabled={!myTurn || selected.length === 0}
        >
          {myTurn
            ? `Play${selected.length > 0 ? ` ${selected.length}` : ""}`
            : "Wait"}
        </button>
      </div>
      <div className="tta-tip">
        {myTurn &&
        format !== null &&
        trick.played_cards.length > 0 &&
        Math.max(...format.units.map(unitWidth)) >= 2
          ? "Tap one card of a pair to select both."
          : myTurn
            ? "Tap cards to select them, then Play."
            : ""}
      </div>

      <HandTray
        groups={groups}
        selected={selected}
        suggested={
          coachOpen && hint !== null
            ? hintKeys(hint.cards as string[])
            : undefined
        }
        onTap={tap}
      />
    </div>
  );
};

export default Table;
