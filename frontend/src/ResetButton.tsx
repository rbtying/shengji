import * as React from "react";
import { WebsocketContext } from "./WebsocketProvider";
import { GameState } from "./gen-types";

interface IProps {
  state: GameState;
  name: string;
}

const ResetButton = (props: IProps): JSX.Element => {
  const { send } = React.useContext(WebsocketContext);

  let requester: string | undefined = undefined;
  if ("Draw" in props.state) {
    const state = props.state.Draw;
    requester = state.propagated.players.find(
      (player) => player.id === state.player_requested_reset,
    )?.name;
  } else if ("Exchange" in props.state) {
    const state = props.state.Exchange;
    requester = state.propagated.players.find(
      (player) => player.id === state.player_requested_reset,
    )?.name;
  } else if ("Play" in props.state) {
    const state = props.state.Play;
    requester = state.propagated.players.find(
      (player) => player.id === state.player_requested_reset,
    )?.name;
  }

  if (requester == null) {
    return (
      <div className="reset-block">
        <a
          href={window.location.href}
          onClick={(evt) => {
            evt.preventDefault();
            send({ Action: "ResetGame" });
          }}
          title="Request to return to the game settings screen and re-deal all cards"
        >
          Reset game
        </a>
      </div>
    );
  } else if (requester === props.name) {
    return (
      <div className="reset-block">
        <p>Waiting for confirmation...</p>
        <a
          href={window.location.href}
          onClick={(evt) => {
            evt.preventDefault();
            send({ Action: "CancelResetGame" });
          }}
          title="Continue playing the game"
        >
          Cancel
        </a>
      </div>
    );
  } else {
    return (
      <div className="reset-block">
        <p>{requester} wants to reset the game</p>
        <a
          href={window.location.href}
          onClick={() => {
            send({ Action: "ResetGame" });
          }}
          title="Return to the game settings screen and re-deal all cards"
          style={{
            marginRight: "8px",
          }}
        >
          Accept
        </a>
        <a
          href={window.location.href}
          onClick={(evt) => {
            evt.preventDefault();
            send({ Action: "CancelResetGame" });
          }}
          title="Continue playing the game"
        >
          Cancel
        </a>
      </div>
    );
  }
};

export default ResetButton;
