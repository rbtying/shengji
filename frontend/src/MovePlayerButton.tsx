import * as React from "react";
import { IPlayer } from "./types";
import { WebsocketContext } from "./WebsocketProvider";

interface Props {
  players: IPlayer[];
  player: IPlayer;
}

function MovePlayerButton(
  relative: number,
  children: string
): React.FunctionComponent<Props> {
  const component = (props: Props): JSX.Element => {
    const { players, player } = props;
    const { send } = React.useContext(WebsocketContext);

    const movePlayer = (): void => {
      const index = players.findIndex((p) => p === player);
      const newIndex = (index + relative) % players.length;
      const withoutPlayer = players.filter((p) => p !== player);
      const newPlayers = [
        ...withoutPlayer.slice(0, newIndex),
        player,
        ...withoutPlayer.slice(newIndex, withoutPlayer.length),
      ];
      send({ Action: { ReorderPlayers: newPlayers.map((p) => p.id) } });
    };

    return <button onClick={movePlayer}>{children}</button>;
  };
  component.displayName = "MovePlayerButton";
  return component;
}

export const MovePlayerLeft = MovePlayerButton(-1, "<");
export const MovePlayerRight = MovePlayerButton(1, ">");
