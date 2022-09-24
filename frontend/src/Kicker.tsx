import * as React from "react";
import { Player } from "./gen-types";

interface IProps {
  onKick: (playerId: number) => void;
  players: Player[];
}
const Kicker = (props: IProps): JSX.Element => {
  const [selection, setSelection] = React.useState<number | null>(null);

  const handleChange = (e: React.ChangeEvent<HTMLSelectElement>): void => {
    setSelection(e.target.value === "" ? null : parseInt(e.target.value, 10));
  };

  return (
    <div className="kicker">
      <label>
        Kick player:{" "}
        <select
          value={selection === null ? "" : selection}
          onChange={handleChange}
        >
          <option value="" />
          {props.players.map((player) => (
            <option value={player.id} key={player.id}>
              {player.name}
            </option>
          ))}
        </select>
        <button
          onClick={() => props.onKick(selection)}
          disabled={selection === null}
        >
          Kick
        </button>
      </label>
    </div>
  );
};

export default Kicker;
