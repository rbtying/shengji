import * as React from 'react';
import {IPlayer} from './types';
import classNames from 'classnames';
import {MovePlayerLeft, MovePlayerRight} from './MovePlayerButton';

type Props = {
  players: IPlayer[];
  landlord?: number | null;
  landlords_team?: number[];
  movable?: boolean;
  next?: number | null;
  name: string;
};

const Players = (props: Props) => {
  const {players, landlord, landlords_team, movable, next, name} = props;

  return (
    <table className="players">
      <tbody>
        <tr>
          {players.map((player) => {
            const className = classNames('player', {
              next: player.id === next,
              landlord:
                player.id === landlord || landlords_team?.includes(player.id),
            });

            let descriptor = `${player.name} (rank ${player.level})`;

            if (player.id === landlord) {
              descriptor = descriptor + ' (当庄)';
            }
            if (player.name === name) {
              descriptor = descriptor + ' (You!)';
            }

            return (
              <td key={player.id} className={className}>
                {movable && (
                  <MovePlayerLeft players={players} player={player} />
                )}
                {descriptor}
                {movable && (
                  <MovePlayerRight players={players} player={player} />
                )}
              </td>
            );
          })}
        </tr>
      </tbody>
    </table>
  );
};

export default Players;
