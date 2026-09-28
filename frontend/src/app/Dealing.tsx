import * as React from "react";
import { WebsocketContext } from "../WebsocketProvider";
import { useEngine } from "../useEngine";
import { Bid, DrawPhase, Trump } from "../gen-types";
import { displayName, face, handCards } from "./cards";
import HandTray from "./HandTray";
import MiniCard from "./MiniCard";
import { useHandGroups } from "./useHandGroups";

import type { JSX } from "react";

interface IProps {
  phase: DrawPhase;
  me: number;
  onHome: () => void;
}

/** Dealing: your cards arrive automatically; bid a level card to name trump. */
const Dealing = (props: IProps): JSX.Element => {
  const { send } = React.useContext(WebsocketContext);
  const engine = useEngine();
  const d = props.phase;
  const players = d.propagated.players;
  const myPlayer = players.find((p) => p.id === props.me);
  const landlord = d.propagated.landlord ?? null;
  const levelPlayer = players.find((p) => p.id === (landlord ?? props.me));
  const level = levelPlayer?.level ?? "2";
  const cards = handCards(d.hands, props.me);
  const [validBids, setValidBids] = React.useState<Bid[]>([]);
  const deckLeft = d.deck.length;
  const myTurnToDraw = deckLeft > 0 && players[d.position]?.id === props.me;

  // Draw your own cards automatically, at a steady pace.
  React.useEffect(() => {
    if (!myTurnToDraw) {
      return;
    }
    const t = setTimeout(() => send({ Action: "DrawCard" }), 90);
    return () => clearTimeout(t);
  }, [myTurnToDraw, deckLeft]);

  React.useEffect(() => {
    let live = true;
    engine
      .findValidBids({
        id: props.me,
        bids: d.bids,
        hands: d.hands,
        players,
        landlord,
        epoch: 0,
        bid_policy: d.propagated.bid_policy!,
        bid_reinforcement_policy: d.propagated.bid_reinforcement_policy!,
        joker_bid_policy: d.propagated.joker_bid_policy!,
        num_decks: d.num_decks,
      })
      .then((bids) => {
        if (live) {
          setValidBids(bids);
        }
      })
      .catch(() => {
        if (live) {
          setValidBids([]);
        }
      });
    return () => {
      live = false;
    };
  }, [JSON.stringify([d.bids, d.hands.hands[props.me], landlord])]);

  const sortTrump: Trump = {
    NoTrump: { number: level === "NT" ? null : (level as any) },
  };
  const groups = useHandGroups(cards, sortTrump);

  const winning =
    d.bids.length > 0 ? d.bids[d.bids.length - 1] : (d.autobid ?? null);
  const winnerName = (id: number): string =>
    id === props.me
      ? "You"
      : displayName(players.find((p) => p.id === id)?.name ?? "");
  const whoPicksUp = landlord ?? winning?.id ?? null;
  const canBid = (d.revealed_cards ?? 0) === 0 && d.autobid == null;

  let status: string;
  if (deckLeft > 0) {
    status = `Dealing · ${deckLeft} cards left`;
  } else if (winning === null) {
    status =
      landlord === props.me
        ? "Nobody bid. Reveal cards from the kitty to set trump."
        : "Nobody has bid yet.";
  } else {
    status =
      whoPicksUp === props.me
        ? "Dealing done. Pick up the kitty when you're ready."
        : `Waiting for ${winnerName(whoPicksUp!)} to pick up the kitty…`;
  }

  return (
    <div className="tta-table">
      <div className="tta-hud">
        <div className="tta-trump">
          <span className="tta-label">Level</span>
          <b>{level}</b>
        </div>
        <div className="tta-levels">
          <span className="tta-level ours">
            <i>{myPlayer?.level}</i>You
          </span>
        </div>
        <button
          type="button"
          className="tta-round"
          aria-label="Home"
          onClick={props.onHome}
        >
          ‹
        </button>
      </div>

      <div className="tta-felt">
        <div className="tta-felt-note" style={{ top: "34%", width: "80%" }}>
          {status}
        </div>
        <div
          style={{
            position: "absolute",
            left: 12,
            right: 12,
            bottom: 12,
            display: "flex",
            flexDirection: "column",
            gap: 6,
          }}
        >
          <span className="tta-label" style={{ color: "var(--gold-soft)" }}>
            Bids
          </span>
          {d.bids.length === 0 && d.autobid == null ? (
            <span className="tta-note" style={{ color: "var(--felt-muted)" }}>
              Show a {level} (or jokers) to make its suit trump. A pair beats a
              single.
            </span>
          ) : (
            <div style={{ display: "flex", flexWrap: "wrap", gap: 10 }}>
              {d.bids.map((b, i) => (
                <div
                  key={i}
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: 6,
                    color: "var(--felt-text)",
                    fontSize: 13,
                    fontWeight: 700,
                  }}
                >
                  {winnerName(b.id)}
                  <span style={{ display: "flex" }}>
                    {Array.from({ length: b.count }, (_, j) => (
                      <MiniCard
                        key={j}
                        card={b.card}
                        small
                        style={{ marginLeft: j === 0 ? 0 : -18 }}
                      />
                    ))}
                  </span>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      <div className="tta-actions">
        {canBid &&
          validBids.slice(0, 3).map((b, i) => (
            <button
              key={i}
              type="button"
              className="tta-btn outline"
              onClick={() => send({ Action: { Bid: [b.card, b.count] } })}
            >
              Bid {b.count > 1 ? `${b.count}× ` : ""}
              {face(b.card).rank}
              {face(b.card).rank === "★" ? " joker" : face(b.card).suit}
            </button>
          ))}
        {deckLeft === 0 && whoPicksUp === props.me && winning !== null && (
          <button
            type="button"
            className="tta-btn primary grow"
            onClick={() => send({ Action: "PickUpKitty" })}
          >
            Pick up the kitty
          </button>
        )}
        {deckLeft === 0 && winning === null && landlord === props.me && (
          <button
            type="button"
            className="tta-btn primary grow"
            onClick={() => send({ Action: "RevealCard" })}
          >
            Reveal a card
          </button>
        )}
      </div>
      <div className="tta-tip">
        {canBid && validBids.length > 0
          ? "You can bid now, or wait for a stronger hand."
          : deckLeft > 0
            ? "Your cards are drawn for you."
            : ""}
      </div>

      <HandTray groups={groups} selected={[]} title="Your hand" />
    </div>
  );
};

export default Dealing;
