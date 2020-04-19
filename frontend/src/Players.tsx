import * as React from 'react';
import {IPlayer} from './types';
import classNames from 'classnames';

type Props = {
  players: IPlayer[];
  observers: IPlayer[];
  landlord?: number | null;
  landlords_team?: number[];
  next?: number | null;
  name: string;
};

const Players = (props: Props) => {
  const {
    players,
    observers,
    landlord,
    landlords_team,
    next,
    name,
  } = props;

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
                {descriptor}
              </td>
            );
          })}
          {observers.map((player) => {
            const className = classNames('player observer');
            let descriptor = `${player.name} (rank ${player.level})`;

            if (player.name === name) {
              descriptor = descriptor + ' (You!)';
            }

            return (
              <td key={player.id} className={className}>
                <span style={{textDecoration: 'line-through'}}>
                  {descriptor}
                </span>
              </td>
            );
          })}
        </tr>
      </tbody>
    </table>
  );
};

export default Players;
