import * as React from "react";
import { Player } from "./gen-types";

interface IProps {
  landlordId: number | null;
  onChange: (newLandlord: number | null) => void;
  players: Player[];
}
const LandlordSelector = (props: IProps): JSX.Element => {
  const handleChange = (e: React.ChangeEvent<HTMLSelectElement>): void => {
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
          <option value="">determined by the bid</option>
          {props.players.map((player: Player) => (
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
