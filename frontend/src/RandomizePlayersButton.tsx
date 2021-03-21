import * as React from "react";
import { IPlayer } from "./types";
import { WebsocketContext } from "./WebsocketProvider";
import ArrayUtils from "./util/array";

interface Props {
  players: IPlayer[];
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

  return <button onClick={randomize}>{props.children}</button>;
};
