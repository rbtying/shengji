import * as React from "react";
import { AppStateContext } from "../AppStateProvider";
import { WebsocketContext } from "../WebsocketProvider";
import { cardLookup } from "../util/cardHelpers";
import { ExchangePhase } from "../gen-types";
import { displayName, handCards, suitSymbol, trumpLabel } from "./cards";
import HandTray from "./HandTray";
import MiniCard from "./MiniCard";
import { useHandGroups } from "./useHandGroups";

import type { JSX } from "react";

interface IProps {
  phase: ExchangePhase;
  me: number;
}

const ORDINALS = ["first", "second", "third", "fourth"];
const RANKS = ["A", "K", "Q", "J"];
const SUITS = ["♤", "♡", "♧", "♢"];

const cardFor = (rank: string, suit: string): string | undefined =>
  Object.values(cardLookup).find((c) => c.number === rank && c.typ === suit)
    ?.value;

interface FriendCall {
  rank: string;
  suit: string;
  skip: number;
}

/** Bury the kitty (and, in Finding Friends, call your friend) before play starts. */
const Kitty = (props: IProps): JSX.Element => {
  const { state, updateState } = React.useContext(AppStateContext);
  const { send } = React.useContext(WebsocketContext);
  const e = props.phase;
  const players = e.propagated.players;
  const name = (id: number): string =>
    id === props.me
      ? "You"
      : displayName(players.find((p) => p.id === id)?.name ?? "");
  const trump = trumpLabel(e.trump);
  const exchanging = e.exchanger === props.me && !e.finalized;
  const hand = handCards(e.hands, props.me);
  const groups = useHandGroups(hand, e.trump);
  const kittyFull = e.kitty.length === e.kitty_size;
  const numFriends =
    e.game_mode !== "Tractor" ? e.game_mode.FindingFriends.num_friends : 0;
  const trumpSuit = "Standard" in e.trump ? e.trump.Standard.suit : null;
  const trumpNumber =
    "Standard" in e.trump ? e.trump.Standard.number : e.trump.NoTrump.number;
  const allowedSuits = SUITS.filter((s) => s !== trumpSuit);
  const allowedRanks = RANKS.filter((r) => r !== trumpNumber);
  const [calls, setCalls] = React.useState<FriendCall[]>(() =>
    Array.from({ length: numFriends }, (_, i) => ({
      rank: allowedRanks[0],
      suit: allowedSuits[i % allowedSuits.length],
      skip: 0,
    })),
  );
  const [asked, setAsked] = React.useState(false);
  const hint = asked ? state.hint : null;

  if (!exchanging) {
    return (
      <div className="tta-screen">
        <div className="tta-center">
          <div className="tta-label">Trump is {trump.text}</div>
          <h1 className="tta-title">
            {name(e.exchanger)} is burying the kitty
          </h1>
          <p className="tta-note">
            {numFriends > 0
              ? "Then they'll call a card. Whoever plays it becomes their secret friend."
              : "Play starts in a moment."}
          </p>
        </div>
        <HandTray groups={groups} selected={[]} title="Your hand" />
      </div>
    );
  }

  const bury = (card: string): void => {
    if (!kittyFull) {
      send({ Action: { MoveCardToKitty: card } });
    }
  };
  const unbury = (card: string): void =>
    send({ Action: { MoveCardToHand: card } });

  const suggest = (): void => {
    setAsked(true);
    updateState({ hint: null, hintPending: true });
    send("RequestHint");
  };
  const applySuggestion = (cards: string[]): void => {
    const want = [...cards];
    const back: string[] = [];
    e.kitty.forEach((c) => {
      const i = want.indexOf(c);
      if (i >= 0) {
        want.splice(i, 1);
      } else {
        back.push(c);
      }
    });
    back.forEach(unbury);
    want.forEach((c) => send({ Action: { MoveCardToKitty: c } }));
  };

  const start = (): void => {
    if (numFriends > 0) {
      const friends = calls
        .map((c) => ({ card: cardFor(c.rank, c.suit), initial_skip: c.skip }))
        .filter((f) => f.card !== undefined);
      send({ Action: { SetFriends: friends } });
    }
    if (
      e.propagated.kitty_theft_policy === "AllowKittyTheft" &&
      e.autobid == null
    ) {
      send({ Action: "PutDownKitty" });
    }
    send({ Action: "BeginPlay" });
  };

  const updateCall = (i: number, change: Partial<FriendCall>): void =>
    setCalls(calls.map((c, j) => (j === i ? { ...c, ...change } : c)));

  return (
    <div className="tta-table">
      <div className="tta-screen tta-kitty-top">
        <div className="tta-topbar">
          <div>
            <h1 className="tta-title">Bury the kitty</h1>
            <div className="tta-note">
              Trump is{" "}
              <b style={{ color: trump.red ? "var(--red)" : undefined }}>
                {trump.text}
              </b>{" "}
              ·{" "}
              <span lang="zh" className="tta-zh">
                底牌
              </span>
            </div>
          </div>
        </div>

        <div className="tta-kitty-box">
          <div className="tta-label">
            <span>Kitty · tap a card to take it back</span>
            <span>
              {e.kitty.length} / {e.kitty_size}
            </span>
          </div>
          <div className="tta-kitty-slots">
            {e.kitty.map((c, i) => (
              <MiniCard key={`${c}-${i}`} card={c} onClick={() => unbury(c)} />
            ))}
            {Array.from(
              { length: Math.max(0, e.kitty_size - e.kitty.length) },
              (_, i) => (
                <div key={`slot-${i}`} className="tta-slot" />
              ),
            )}
          </div>
        </div>

        {asked && (state.hintPending || hint !== null) && (
          <div className="tta-panel" role="status">
            {state.hintPending ? (
              "Thinking…"
            ) : (
              <>
                <strong>Coach:</strong> {hint!.summary}{" "}
                <button
                  type="button"
                  className="tta-chip"
                  onClick={() => applySuggestion(hint!.cards as string[])}
                >
                  Use this
                </button>
              </>
            )}
          </div>
        )}

        {numFriends > 0 &&
          calls.map((c, i) => (
            <div className="tta-section" key={i}>
              <div className="tta-label">
                Call {numFriends > 1 ? `friend ${i + 1}` : "your friend"}: the{" "}
                {ORDINALS[c.skip]} {c.rank}
                {suitSymbol(c.suit)} played
              </div>
              <div className="tta-friend-pick">
                {allowedRanks.map((r) => (
                  <button
                    key={r}
                    type="button"
                    className="tta-chip"
                    aria-pressed={c.rank === r}
                    onClick={() => updateCall(i, { rank: r })}
                  >
                    {r}
                  </button>
                ))}
                {allowedSuits.map((s) => (
                  <button
                    key={s}
                    type="button"
                    className="tta-chip"
                    aria-pressed={c.suit === s}
                    onClick={() => updateCall(i, { suit: s })}
                    style={{
                      color: s === "♡" || s === "♢" ? "var(--red)" : undefined,
                    }}
                  >
                    {suitSymbol(s)}
                  </button>
                ))}
              </div>
              <div className="tta-friend-pick">
                {ORDINALS.slice(0, e.num_decks).map((o, k) => (
                  <button
                    key={o}
                    type="button"
                    className="tta-chip"
                    aria-pressed={c.skip === k}
                    onClick={() => updateCall(i, { skip: k })}
                  >
                    {o}
                  </button>
                ))}
              </div>
            </div>
          ))}
      </div>

      <div className="tta-actions">
        <button
          type="button"
          className="tta-btn outline"
          onClick={suggest}
          disabled={state.hintPending}
        >
          Suggest
        </button>
        <button
          type="button"
          className="tta-btn primary grow"
          onClick={start}
          disabled={!kittyFull}
        >
          {kittyFull
            ? "Start play"
            : `Bury ${e.kitty_size - e.kitty.length} more`}
        </button>
      </div>
      <div className="tta-tip">Tap cards in your hand to bury them.</div>

      <HandTray
        groups={groups}
        selected={[]}
        onTap={(card) => bury(card)}
        title="Your hand"
      />
    </div>
  );
};

export default Kitty;
