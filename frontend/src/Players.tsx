import * as React from "react";

import classNames from "classnames";
import { MovePlayerLeft, MovePlayerRight } from "./MovePlayerButton";
import { Player } from "./gen-types";
import { WebsocketContext } from "./WebsocketProvider";

import type { JSX } from "react";

interface IProps {
  players: Player[];
  observers: Player[];
  landlord?: number | null;
  landlords_team?: number[];
  riverCrossingInitiators?: number[];
  movable?: boolean;
  next?: number | null;
  name: string;
}

const Players = (props: IProps): JSX.Element => {
  const {
    players,
    observers,
    landlord,

    landlords_team,
    movable,
    next,
    name,
  } = props;
  const { send } = React.useContext(WebsocketContext);

  const makeDescriptor = (p: Player): Array<JSX.Element | string> => {
    const descriptor: Array<JSX.Element | string> = [p.name];
    if (props.riverCrossingInitiators?.includes(p.id)) {
      descriptor.push(
        <span
          key={`river-crossing-${p.id}`}
          className="river-crossing-badge"
          title="本小局曾发起五主过河；下一小局自动清除 / Initiated river crossing this deal"
        >
          已过河
        </span>,
      );
    }
    if (p.metalevel <= 1) {
      descriptor.push(` (rank ${p.level})`);
    } else {
      descriptor.push(
        ` (rank ${p.level}`,
        <sup key={`meta-${p.id}`}>{p.metalevel}</sup>,
        ")",
      );
    }
    return descriptor;
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
