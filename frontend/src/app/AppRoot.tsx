import * as React from "react";
import { AppStateContext } from "../AppStateProvider";
import { WebsocketContext } from "../WebsocketProvider";
import { GameState, InitializePhase } from "../gen-types";
import { displayName } from "./cards";
import Dealing from "./Dealing";
import Home from "./Home";
import Kitty from "./Kitty";
import Review from "./Review";
import Setup from "./Setup";
import Table from "./Table";

import "./app.css";

import type { JSX } from "react";

const players = (gs: GameState) =>
  ("Initialize" in gs
    ? gs.Initialize
    : "Draw" in gs
      ? gs.Draw
      : "Exchange" in gs
        ? gs.Exchange
        : gs.Play
  ).propagated.players;

/** Between hands: standings, then deal again (or change the table). */
const BetweenHands = (props: {
  phase: InitializePhase;
  me: number;
  onDeal: () => void;
  onChange: () => void;
  onHome: () => void;
}): JSX.Element => {
  const ps = props.phase.propagated.players;
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
        <h1 className="tta-title">Standings</h1>
      </div>
      <div className="tta-list">
        {ps.map((p) => (
          <div
            key={p.id}
            className={`tta-list-item ${p.id === props.me ? "ours" : ""}`}
          >
            <span>
              {p.id === props.me ? "You" : displayName(p.name)}
              {props.phase.propagated.landlord === p.id
                ? " · landlord next"
                : ""}
            </span>
            <b>Level {String(p.level)}</b>
          </div>
        ))}
      </div>
      <div className="tta-row tta-bottom">
        <button type="button" className="tta-btn" onClick={props.onChange}>
          Change table
        </button>
        <button
          type="button"
          className="tta-btn primary grow"
          onClick={props.onDeal}
        >
          Deal next hand
        </button>
      </div>
    </div>
  );
};

/** The phone app: home, setup, dealing, kitty, table and review screens. */
const AppRoot = (): JSX.Element => {
  const { state, updateState } = React.useContext(AppStateContext);
  const { send } = React.useContext(WebsocketContext);
  const [onHome, setOnHome] = React.useState(true);
  const [changingTable, setChangingTable] = React.useState(false);
  const nextHandRequested = React.useRef(false);
  const gs = state.gameState;

  // Errors from the engine show briefly as a banner.
  const lastError =
    state.errors.length > 0 ? state.errors[state.errors.length - 1] : null;
  React.useEffect(() => {
    if (lastError === null) {
      return;
    }
    const t = setTimeout(() => updateState({ errors: [] }), 4000);
    return () => clearTimeout(t);
  }, [state.errors.length]);

  // After "Next hand", deal as soon as the table resets.
  React.useEffect(() => {
    if (gs !== null && "Initialize" in gs && nextHandRequested.current) {
      nextHandRequested.current = false;
      send({ Action: "StartGame" });
    }
  }, [gs !== null && "Initialize" in gs]);

  const me =
    gs !== null
      ? (players(gs).find((p) => p.name === state.name)?.id ?? null)
      : null;

  let screen: JSX.Element;
  if (onHome || gs === null || me === null) {
    screen = <Home onEnterGame={() => setOnHome(false)} />;
  } else if ("Initialize" in gs) {
    const finished = (gs.Initialize.propagated.num_games_finished ?? 0) > 0;
    screen =
      finished && !changingTable ? (
        <BetweenHands
          phase={gs.Initialize}
          me={me}
          onDeal={() => send({ Action: "StartGame" })}
          onChange={() => setChangingTable(true)}
          onHome={() => setOnHome(true)}
        />
      ) : (
        <Setup
          phase={gs.Initialize}
          me={me}
          onBack={() => {
            setChangingTable(false);
            setOnHome(true);
          }}
        />
      );
  } else if ("Draw" in gs) {
    screen = <Dealing phase={gs.Draw} me={me} onHome={() => setOnHome(true)} />;
  } else if ("Exchange" in gs) {
    screen = <Kitty phase={gs.Exchange} me={me} />;
  } else {
    const p = gs.Play;
    const finished =
      p.game_ended_early ||
      (Object.values(p.hands.hands).every((h) =>
        Object.values(h).every((n) => n === 0),
      ) &&
        p.trick.played_cards.length === 0);
    screen = finished ? (
      <Review
        phase={p}
        me={me}
        onHome={() => setOnHome(true)}
        onNextHand={() => {
          nextHandRequested.current = true;
          setChangingTable(false);
          send({ Action: "StartNewGame" });
        }}
      />
    ) : (
      <Table phase={p} me={me} onHome={() => setOnHome(true)} />
    );
  }

  return (
    <div className="tta">
      {lastError !== null && (
        <div
          className="tta-errors"
          role="alert"
          onClick={() => updateState({ errors: [] })}
        >
          {lastError.replace(/^Failed to handle user action: /, "")}
        </div>
      )}
      {screen}
    </div>
  );
};

export default AppRoot;
