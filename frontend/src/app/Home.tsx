import * as React from "react";
import { AppStateContext } from "../AppStateProvider";
import { WebsocketContext } from "../WebsocketProvider";
import { clearOfflineGame, OFFLINE_ROOM } from "../OfflineProvider";
import { GameState } from "../gen-types";
import MiniCard from "./MiniCard";
import { setCardStyle, useCardStyle } from "./cardStyle";

import type { JSX } from "react";

const AUTOJOIN_KEY = "tta_autojoin";

interface IProps {
  onEnterGame: () => void;
}

const summary = (gs: GameState): string => {
  if ("Initialize" in gs) {
    return gs.Initialize.propagated.num_games_finished! > 0
      ? "Between hands"
      : "Table set, not dealt yet";
  }
  const mode =
    "Play" in gs
      ? gs.Play.game_mode
      : "Exchange" in gs
        ? gs.Exchange.game_mode
        : gs.Draw.game_mode;
  return mode === "Tractor"
    ? "Tractor · hand in progress"
    : "Finding Friends · hand in progress";
};

const Home = (props: IProps): JSX.Element => {
  const { state, updateState } = React.useContext(AppStateContext);
  const { send } = React.useContext(WebsocketContext);
  const [showSettings, setShowSettings] = React.useState(false);
  const [name, setName] = React.useState(state.name || "");
  const gs = state.gameState;
  const cardStyle = useCardStyle();

  const join = (n: string): void => {
    const trimmed = n.trim().slice(0, 31) || "You";
    updateState({ name: trimmed, roomName: OFFLINE_ROOM });
    send({ room_name: OFFLINE_ROOM, name: trimmed, disable_compression: true });
    props.onEnterGame();
  };

  // "New game" reloads with a fresh table and comes straight back here to join it.
  React.useEffect(() => {
    let auto = false;
    try {
      auto = window.sessionStorage.getItem(AUTOJOIN_KEY) === "1";
      window.sessionStorage.removeItem(AUTOJOIN_KEY);
    } catch {
      // No session storage: stay on the home screen.
    }
    if (auto && state.name) {
      join(state.name);
    }
  }, []);

  const newGame = (): void => {
    clearOfflineGame();
    updateState({ name: name.trim() || state.name || "You" });
    try {
      window.sessionStorage.setItem(AUTOJOIN_KEY, "1");
    } catch {
      // Without it the player just taps Start again.
    }
    window.location.reload();
  };

  return (
    <div className="tta-screen tta-home">
      <div className="tta-brand">
        <div className="tta-brand-fan" aria-hidden="true">
          <MiniCard card="🂢" />
          <MiniCard card="🂾" />
          <MiniCard card="🃑" />
        </div>
        <h1>Two to Ace</h1>
        <div className="zh" lang="zh">
          升级 · 找朋友
        </div>
        <div className="sub">Tractor &amp; Finding Friends</div>
      </div>

      {gs === null ? (
        <div className="tta-menu">
          <label className="tta-field">
            Your name
            <input
              type="text"
              value={name}
              maxLength={31}
              placeholder="You"
              onChange={(e) => setName(e.target.value)}
            />
          </label>
          <button
            type="button"
            className="tta-btn felt"
            onClick={() => join(name)}
          >
            <span>
              Sit down at the table
              <small>You and computer players, on this device</small>
            </span>
            <span aria-hidden="true">›</span>
          </button>
        </div>
      ) : (
        <div className="tta-menu">
          <button
            type="button"
            className="tta-btn felt"
            onClick={props.onEnterGame}
          >
            <span>
              Continue
              <small>{summary(gs)}</small>
            </span>
            <span aria-hidden="true">›</span>
          </button>
          <button type="button" className="tta-btn" onClick={newGame}>
            <span>
              New game
              <small>
                Choose Tractor or Finding Friends and your opponents
              </small>
            </span>
          </button>
        </div>
      )}

      <div className="tta-menu">
        <a
          className="tta-btn outline"
          href="rules.html"
          target="_blank"
          rel="noreferrer"
        >
          <span>
            How to play
            <small>The rules of both games</small>
          </span>
        </a>
      </div>

      <div className="tta-footer tta-bottom">
        <span>No ads · No coins · Plays offline</span>
        <button type="button" onClick={() => setShowSettings(true)}>
          Settings
        </button>
      </div>

      {showSettings && (
        <div
          className="tta-sheet-backdrop"
          onClick={() => setShowSettings(false)}
        >
          <div
            className="tta-sheet"
            role="dialog"
            aria-label="Settings"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="tta-section">
              <div className="tta-topbar">
                <h2 className="tta-title">Settings</h2>
              </div>
              <div className="tta-section">
                <div className="tta-label">Cards</div>
                <div className="tta-chips">
                  <button
                    type="button"
                    className="tta-chip"
                    aria-pressed={cardStyle === "faces"}
                    onClick={() => setCardStyle("faces")}
                  >
                    Real card faces
                  </button>
                  <button
                    type="button"
                    className="tta-chip"
                    aria-pressed={cardStyle === "simple"}
                    onClick={() => setCardStyle("simple")}
                  >
                    Large simple cards
                  </button>
                </div>
                <div className="tta-row" aria-hidden="true">
                  <div style={{ display: "flex", gap: 6, flex: "none" }}>
                    <MiniCard card="🂾" />
                    <MiniCard card="🂪" />
                    <MiniCard card="🃅" />
                    <MiniCard card="🃏" />
                    <MiniCard card="🃟" />
                  </div>
                </div>
              </div>
              <div className="tta-panel">
                <strong>Tip jar.</strong> Two to Ace is free, with every level
                and feature included. If you enjoy it, you&apos;ll be able to
                leave an optional tip here in the iOS app.
              </div>
              <a
                className="tta-btn"
                href="licenses.txt"
                target="_blank"
                rel="noreferrer"
              >
                Licenses and credits
              </a>
              <a
                className="tta-btn"
                href="#"
                onClick={(e) => {
                  e.preventDefault();
                  window.location.hash = "";
                  window.location.reload();
                }}
              >
                Play online with people (classic screens)
              </a>
              <button
                type="button"
                className="tta-btn felt"
                onClick={() => setShowSettings(false)}
              >
                Done
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default Home;
