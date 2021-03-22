import * as React from "react";
import classNames from "classnames";
import Errors from "./Errors";
import Initialize from "./Initialize";
import Draw from "./Draw";
import Exchange from "./Exchange";
import JoinRoom from "./JoinRoom";
import { AppStateContext } from "./AppStateProvider";
import { TimerContext } from "./TimerProvider";
import Credits from "./Credits";
import Chat from "./Chat";
import Play from "./Play";
import DebugInfo from "./DebugInfo";

const Confetti = React.lazy(async () => await import("./Confetti"));

const Root = (): JSX.Element => {
  const send = (window as any).send;
  const { state, updateState } = React.useContext(AppStateContext);
  const timerContext = React.useContext(TimerContext);

  const [previousHeaderMessages, setPreviousHeaderMessages] = React.useState<
    string[]
  >([]);
  const [showHeaderMessages, setShowHeaderMessages] = React.useState<boolean>(
    state.headerMessages.length > 0
  );
  React.useEffect(() => {
    if (
      state.headerMessages.length > 0 &&
      (previousHeaderMessages.length !== state.headerMessages.length ||
        !previousHeaderMessages.every((m, i) => state.headerMessages[i] === m))
    ) {
      setShowHeaderMessages(true);
    } else if (state.headerMessages.length === 0) {
      setShowHeaderMessages(false);
    }
    setPreviousHeaderMessages(state.headerMessages);
  }, [state.headerMessages]);

  React.useEffect(() => {
    if (state.settings.darkMode) {
      document.body.classList.add("dark-mode");
    } else {
      document.body.classList.remove("dark-mode");
    }

    return () => {
      document.body.classList.remove("dark-mode");
    };
  }, [state.settings.darkMode]);

  const headerMessages = showHeaderMessages ? (
    <div
      className="header-message"
      onClick={() => setShowHeaderMessages(false)}
    >
      {state.headerMessages.map((msg, idx) => (
        <p key={idx}>{msg}</p>
      ))}
    </div>
  ) : null;
  if (state.connected) {
    if (state.gameState === null || state.roomName.length !== 16) {
      return (
        <div>
          {headerMessages}
          <Errors errors={state.errors} />
          <div className="game">
            <h1>
              升级 / <span className="red">Tractor</span> / 找朋友 /{" "}
              <span className="red">Finding Friends</span>
            </h1>
            <JoinRoom
              name={state.name}
              room_name={state.roomName}
              setName={(name: string) => updateState({ name })}
              setRoomName={(roomName: string) => {
                updateState({ roomName });
                window.location.hash = roomName;
              }}
            />
          </div>
          <hr />
          <Credits />
        </div>
      );
    } else {
      return (
        <div
          className={classNames(
            state.settings.fourColor ? "four-color" : null,
            state.settings.showCardLabels ? "always-show-labels" : null
          )}
        >
          {headerMessages}
          <Errors errors={state.errors} />
          {state.confetti !== null ? (
            <React.Suspense fallback={null}>
              <Confetti
                confetti={state.confetti}
                clearConfetti={() => updateState({ confetti: null })}
              />
            </React.Suspense>
          ) : null}
          <div className="game">
            {state.gameState.Initialize !== undefined ? null : (
              <a
                href={window.location.href}
                className="reset-link"
                onClick={(evt) => {
                  evt.preventDefault();
                  if (window.confirm("Do you really want to reset the game?")) {
                    send({ Action: "ResetGame" });
                  }
                }}
              >
                Reset game
              </a>
            )}
            {state.gameState.Initialize !== undefined ? (
              <Initialize
                state={state.gameState.Initialize}
                name={state.name}
              />
            ) : null}
            {state.gameState.Draw !== undefined ? (
              <Draw
                state={state.gameState.Draw}
                playDrawCardSound={state.settings.playDrawCardSound}
                name={state.name}
                setTimeout={timerContext.setTimeout}
                clearTimeout={timerContext.clearTimeout}
              />
            ) : null}
            {state.gameState.Exchange !== undefined ? (
              <Exchange state={state.gameState.Exchange} name={state.name} />
            ) : null}
            {state.gameState.Play !== undefined ? (
              <Play
                playPhase={state.gameState.Play}
                name={state.name}
                showLastTrick={state.settings.showLastTrick}
                unsetAutoPlayWhenWinnerChanges={
                  state.settings.unsetAutoPlayWhenWinnerChanges
                }
                showTrickInPlayerOrder={state.settings.showTrickInPlayerOrder}
                beepOnTurn={state.settings.beepOnTurn}
              />
            ) : null}
            {state.settings.showDebugInfo ? <DebugInfo /> : null}
          </div>
          <Chat messages={state.messages} />
          <hr />
          <Credits />
        </div>
      );
    }
  } else if (state.everConnected) {
    return (
      <>
        <p>
          It looks like you got disconnected from the server, please refresh! If
          the game is still ongoing, you should be able to re-join with the same
          name and pick up where you left off.
        </p>
      </>
    );
  } else {
    return (
      <div>
        <div className="game">
          <h1>
            升级 / <span className="red">Tractor</span> / 找朋友 /{" "}
            <span className="red">Finding Friends</span>
          </h1>
          <p>
            Welcome! This website helps you play 升级 / Tractor / 找朋友 /
            Finding Friends with other people online.
          </p>
          <p>
            If you&apos;re not familiar with the rules, check them out{" "}
            <a href="rules">here</a>!
          </p>
          <p>Connecting to the server...</p>
        </div>
        <hr />
        <Credits />
      </div>
    );
  }
};

export default Root;
