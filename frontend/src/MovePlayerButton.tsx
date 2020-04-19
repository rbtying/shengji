import * as React from 'react';
import {IPlayer} from './types';
import {WebsocketContext} from './WebsocketProvider';
import IconButton from './IconButton';
import CaretUp from './icons/CaretUp';
import CaretDown from './icons/CaretDown';

type Props = {
  players: IPlayer[];
  player: IPlayer;
};

const MovePlayerButton = (relative: number, children: React.ReactNode) => (
  props: Props,
) => {
  const {players, player} = props;
  const {send} = React.useContext(WebsocketContext);

  const movePlayer = () => {
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

  return <IconButton onClick={movePlayer}>{children}</IconButton>;
};

export const MovePlayerLeft = MovePlayerButton(-1, '<');
export const MovePlayerRight = MovePlayerButton(1, '>');
export const MovePlayerUp = MovePlayerButton(-1, <CaretUp width="1em"/>);
export const MovePlayerDown = MovePlayerButton(1, <CaretDown width="1em"/>);
