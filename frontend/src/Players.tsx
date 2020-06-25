import * as React from "react";

import classNames from "classnames";
import { MovePlayerLeft, MovePlayerRight } from "./MovePlayerButton";
import { IPlayer } from "./types";
import { WebsocketContext } from "./WebsocketProvider";

interface IProps {
  players: IPlayer[];
  observers: IPlayer[];
  landlord?: number | null;
  landlords_team?: number[];
  movable?: boolean;
  next?: number | null;
  name: string;
}

const Players = (props: IProps): JSX.Element => {
  const {
    players,
    observers,
    landlord,
    // eslint-disable-next-line @typescript-eslint/naming-convention
    landlords_team,
    movable,
    next,
    name,
  } = props;
  const { send } = React.useContext(WebsocketContext);

  const makeDescriptor = (p: IPlayer): Array<JSX.Element | string> => {
    if (p.metalevel <= 1) {
      return [`${p.name} (rank ${p.level})`];
    } else {
      return [
        `${p.name} (rank ${p.level}`,
        <sup key={`meta-${p.id}`}>{p.metalevel}</sup>,
        ")",
      ];
    }
  };

  return (
    <table className="players">
      <tbody>
        <tr>
          {players.map((player) => {
            const className = classNames("player", {
              landlord:
                player.id === landlord || landlords_team?.includes(player.id),
              movable,
              next: player.id === next,
            });

            const descriptor = makeDescriptor(player);

            if (player.id === landlord) {
              descriptor.push(" (当庄)");
            }
            if (player.name === name) {
              descriptor.push(" (You!)");
            }

            return (
              <td key={player.id} className={className}>
                {descriptor}
                {movable && (
                  <span
                    style={{
                      display: "block",
                      marginTop: "6px",
                      textAlign: "center",
                      width: "100%",
                    }}
                  >
                    <MovePlayerLeft players={players} player={player} />
                    <span
                      style={{ cursor: "pointer" }}
                      onClick={(_) => {
                        send({ Action: { MakeObserver: player.id } });
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
            const className = classNames("player observer", { movable });
            const descriptor = makeDescriptor(player);

            if (player.name === name) {
              descriptor.push(" (You!)");
            }

            return (
              <td key={player.id} className={className}>
                <span style={{ textDecoration: "line-through" }}>
                  {descriptor}
                </span>
                {movable && (
                  <span
                    style={{
                      display: "block",
                      marginTop: "6px",
                      textAlign: "center",
                      width: "100%",
                    }}
                  >
                    <span
                      style={{ cursor: "pointer" }}
                      onClick={(_) => {
                        send({ Action: { MakePlayer: player.id } });
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
