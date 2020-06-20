import * as React from "react";
import { IPlayer } from "./types";

type Props = {
  landlordId: number | null;
  onChange: (newLandlord: number | null) => void;
  players: IPlayer[];
};
const LandlordSelector = (props: Props) => {
  const handleChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    if (e.target.value === "") {
      props.onChange(null);
    } else {
      props.onChange(parseInt(e.target.value, 10));
    }
  };

  return (
    <div className="landlord-picker">
      <label>
        Current leader:{" "}
        <select
          value={props.landlordId === null ? "" : props.landlordId}
          onChange={handleChange}
        >
          <option value="">winner of the bid</option>
          {props.players.map((player: IPlayer) => (
            <option value={player.id} key={player.id}>
              {player.name}
            </option>
          ))}
        </select>
      </label>
    </div>
  );
};

export default LandlordSelector;
