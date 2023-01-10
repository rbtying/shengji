import * as React from "react";
import { Player } from "./gen-types";
import { WebsocketContext } from "./WebsocketProvider";
import ArrayUtils from "./util/array";

interface Props {
  players: Player[];
  children: string | JSX.Element | JSX.Element[];
}

export const RandomizePlayersButton = (props: Props): JSX.Element => {
  const { players } = props;
  const { send } = React.useContext(WebsocketContext);

  const randomize = (): void => {
    send({
      Action: { ReorderPlayers: ArrayUtils.shuffled(players.map((p) => p.id)) },
    });
  };

  return (
    <button className="big" onClick={randomize}>
      {props.children}
    </button>
  );
};
