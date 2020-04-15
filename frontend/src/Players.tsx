import * as React from 'react';
import {IPlayer} from './types';
import classNames from 'classnames';
import {MovePlayerLeft, MovePlayerRight} from './MovePlayerButton';
import {WebsocketContext} from './WebsocketProvider';

type Props = {
  players: IPlayer[];
  observers: IPlayer[];
  landlord?: number | null;
  landlords_team?: number[];
  movable?: boolean;
  next?: number | null;
  name: string;
};

const Players = (props: Props) => {
  const {
    players,
    observers,
    landlord,
    landlords_team,
    movable,
    next,
    name,
  } = props;
  const {send} = React.useContext(WebsocketContext);

  return (
    <table className="players">
      <tbody>
        <tr>
          {players.map((player) => {
            const className = classNames('player', {
              next: player.id === next,
              landlord:
                player.id === landlord || landlords_team?.includes(player.id),
              movable,
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
                {movable && (
                  <span
                    style={{
                      textAlign: 'center',
                      width: '100%',
                      display: 'block',
                    }}
                  >
                    <MovePlayerLeft players={players} player={player} />
                    <span
                      style={{cursor: 'pointer'}}
                      onClick={(evt) => {
                        send({Action: {MakeObserver: player.id}});
                      }}
                    >
                      ✔️
                    </span>
                    <MovePlayerRight players={players} player={player} />
                  </span>
                )}
              </td>
            );
          })}
          {observers.map((player) => {
            const className = classNames('player observer', {movable});
            let descriptor = `${player.name} (rank ${player.level})`;

            if (player.name === name) {
              descriptor = descriptor + ' (You!)';
            }

            return (
              <td key={player.id} className={className}>
                <span style={{textDecoration: 'line-through'}}>
                  {descriptor}
                </span>
                {movable && (
                  <span
                    style={{
                      textAlign: 'center',
                      width: '100%',
                      display: 'block',
                    }}
                  >
                    <span
                      style={{cursor: 'pointer'}}
                      onClick={(evt) => {
                        send({Action: {MakePlayer: player.id}});
                      }}
                    >
                      💤
                    </span>
                  </span>
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
