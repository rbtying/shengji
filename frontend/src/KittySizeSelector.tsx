import * as React from "react";
import { Deck } from "./gen-types";
import ArrayUtils from "./util/array";
import WasmContext from "./WasmContext";

interface IProps {
  numPlayers: number;
  decks: Deck[];
  kittySize: number | null;
  onChange: (newKittySize: number | null) => void;
}

const KittySizeSelector = (props: IProps): JSX.Element => {
  const { computeDeckLen } = React.useContext(WasmContext);
  const handleChange = (e: React.ChangeEvent<HTMLSelectElement>): void => {
    const newKittySize =
      e.target.value === "" ? null : parseInt(e.target.value, 10);
    props.onChange(newKittySize);
  };
  const deckLen = computeDeckLen(props.decks);
  const kittyOffset = deckLen % props.numPlayers;
  const defaultOptions = [
    kittyOffset,
    kittyOffset + props.numPlayers,
    kittyOffset + 2 * props.numPlayers,
    kittyOffset + 3 * props.numPlayers,
    kittyOffset + 4 * props.numPlayers,
  ];
  const potentialOptions = ArrayUtils.range(
    kittyOffset + 4 * props.numPlayers,
    (v) => v
  );

  const options = potentialOptions.filter(
    (v) =>
      !defaultOptions.includes(v) &&
      v < deckLen - props.numPlayers &&
      // Note: this isn't quite right, but it seems fine for the common case of no short decks.
      (deckLen - v) % props.numPlayers <= props.decks.length * 4
  );

  return (
    <div>
      <label>
        Number of cards in the bottom:{" "}
        <select
          value={
            props.kittySize !== undefined && props.kittySize !== null
              ? props.kittySize
              : ""
          }
          onChange={handleChange}
        >
          <optgroup label="Standard">
            <option value="">default</option>
            {defaultOptions
              .filter((v) => v < deckLen - props.numPlayers)
              .map((v) => (
                <option value={v} key={v}>
                  {v} card{v === 1 ? "" : "s"}
                </option>
              ))}
          </optgroup>
          <optgroup label="Requires removing cards from the deck">
            {options.map((v) => (
              <option value={v} key={v}>
                {v} card{v === 1 ? "" : "s"}
              </option>
            ))}
          </optgroup>
        </select>
      </label>
    </div>
  );
};

export default KittySizeSelector;
