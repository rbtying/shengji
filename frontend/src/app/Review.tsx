import * as React from "react";
import { useEngine } from "../useEngine";
import { GameScoreResult, PlayPhase } from "../gen-types";
import { displayName, pointsIn } from "./cards";

import type { JSX } from "react";

interface IProps {
  phase: PlayPhase;
  me: number;
  onNextHand: () => void;
  onHome: () => void;
}

/** The end of a hand: who won, by how much, and where the points went. */
const Review = (props: IProps): JSX.Element => {
  const engine = useEngine();
  const p = props.phase;
  const players = p.propagated.players;
  const name = (id: number): string =>
    id === props.me
      ? "You"
      : displayName(players.find((pl) => pl.id === id)?.name ?? "");
  const landlordSide = p.landlords_team;
  const iAmLandlordSide = landlordSide.includes(props.me);
  const defenderPoints = players
    .filter((pl) => !landlordSide.includes(pl.id))
    .reduce((s, pl) => s + pointsIn(p.points[pl.id] ?? []), 0);
  const params = p.propagated.game_scoring_parameters!;
  const numDecks = p.decks?.length ?? p.num_decks;
  const threshold =
    params.step_size_per_deck *
    numDecks *
    params.num_steps_to_non_landlord_turnover;
  const [score, setScore] = React.useState<GameScoreResult | null>(null);

  React.useEffect(() => {
    let live = true;
    const numFriends =
      p.game_mode !== "Tractor" ? p.game_mode.FindingFriends.num_friends : 0;
    engine
      .computeScore({
        params,
        decks: p.decks ?? [],
        non_landlord_points: defenderPoints,
        smaller_landlord_team_size:
          p.game_mode !== "Tractor" && landlordSide.length < numFriends + 1,
      })
      .then((r) => live && setScore(r.score))
      .catch(() => live && setScore(null));
    return () => {
      live = false;
    };
  }, [defenderPoints]);

  const landlordNames = landlordSide.map(name).join(" & ");
  const defenders = players
    .filter((pl) => !landlordSide.includes(pl.id))
    .map((pl) => name(pl.id))
    .join(" & ");
  let headline = "";
  if (score !== null) {
    const weWon = score.landlord_won === iAmLandlordSide;
    const delta = score.landlord_won
      ? score.landlord_delta
      : score.non_landlord_delta;
    const winners = score.landlord_won ? landlordNames : defenders;
    const plural = winners === "You" || winners.includes("&");
    headline =
      delta > 0
        ? `${winners} go${plural ? "" : "es"} up ${delta} level${delta === 1 ? "" : "s"}`
        : `${winners} take${plural ? "" : "s"} over as landlord`;
    headline = (weWon ? "You win! " : "") + headline;
  }

  const tricks = p.played_tricks ?? [];
  const pointTricks = tricks
    .map((t, i) => ({
      n: i + 1,
      winner: t.current_winner ?? -1,
      points: pointsIn(t.played_cards.flatMap((pc) => pc.cards)),
    }))
    .filter((t) => t.points > 0);

  return (
    <div className="tta-screen">
      <div className="tta-topbar">
        <button
          type="button"
          className="tta-round"
          aria-label="Back to home"
          onClick={props.onHome}
        >
          ‹
        </button>
        <h1 className="tta-title">Hand review</h1>
      </div>

      <div className="tta-result">
        <h2>{headline || "Counting points…"}</h2>
        <p>
          {defenders} took {defenderPoints} of the {threshold} points they
          needed.
        </p>
        <div className="tta-bar" aria-hidden="true">
          <span
            style={{
              width: `${Math.min(100, (100 * defenderPoints) / (threshold * 2))}%`,
            }}
          />
          <em style={{ left: "50%" }} />
        </div>
      </div>

      <div className="tta-section">
        <div className="tta-label">Where the points went</div>
        {pointTricks.length === 0 ? (
          <div className="tta-note">No trick carried points this hand.</div>
        ) : (
          <div className="tta-list">
            {pointTricks.map((t) => (
              <div
                key={t.n}
                className={`tta-list-item ${landlordSide.includes(t.winner) === iAmLandlordSide ? "ours" : "theirs"}`}
              >
                <span>
                  Trick {t.n} · {name(t.winner)}
                </span>
                <b>{t.points} pts</b>
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="tta-row tta-bottom">
        <button type="button" className="tta-btn" onClick={props.onHome}>
          Home
        </button>
        <button
          type="button"
          className="tta-btn primary grow"
          onClick={props.onNextHand}
        >
          Next hand
        </button>
      </div>
    </div>
  );
};

export default Review;
