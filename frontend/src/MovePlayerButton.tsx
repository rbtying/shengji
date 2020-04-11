import * as React from 'react';
import {IPlayer} from './types';
import {WebsocketConsumer} from './WebsocketProvider';

const movePlayer = (
  players: IPlayer[],
  player: IPlayer,
  relative: number,
  send: (value: any) => void,
) => () => {
  const index = players.findIndex((p) => p === player);
  const newIndex = (index + relative) % players.length;
  const withoutPlayer = players.filter((p) => p !== player);
  const newPlayers = [
    ...withoutPlayer.slice(0, newIndex),
    player,
    ...withoutPlayer.slice(newIndex, withoutPlayer.length),
  ];
  send({Action: {ReorderPlayers: newPlayers.map((p) => p.id)}});
};

type Props = {
  players: IPlayer[];
  player: IPlayer;
};

const MovePlayerButton = (relative: number, children: string) => (
  props: Props,
) => (
  <WebsocketConsumer>
    {({send}) => (
      <button onClick={movePlayer(props.players, props.player, relative, send)}>
        {children}
      </button>
    )}
  </WebsocketConsumer>
);

export const MovePlayerLeft = MovePlayerButton(-1, '<');
export const MovePlayerRight = MovePlayerButton(1, '>');
