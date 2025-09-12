import * as React from "react";
import { Deck } from "./gen-types";
import ArrayUtils from "./util/array";
import { useEngine } from "./useEngine";

import type { JSX } from "react";

interface IProps {
  numPlayers: number;
  decks: Deck[];
  kittySize: number | null | undefined;
  onChange: (newKittySize: number | null) => void;
}

const KittySizeSelector = (props: IProps): JSX.Element => {
  const engine = useEngine();
  const [deckLen, setDeckLen] = React.useState<number>(0);
  const [isLoading, setIsLoading] = React.useState<boolean>(true);

  React.useEffect(() => {
    setIsLoading(true);
    engine
      .computeDeckLen(props.decks)
      .then((len) => {
        setDeckLen(len);
        setIsLoading(false);
      })
      .catch((error) => {
        console.error("Error computing deck length:", error);
        // Fallback: estimate based on number of decks
        setDeckLen(props.decks.length * 54);
        setIsLoading(false);
      });
  }, [props.decks, engine]);

  const handleChange = (e: React.ChangeEvent<HTMLSelectElement>): void => {
    const newKittySize =
      e.target.value === "" ? null : parseInt(e.target.value, 10);
    props.onChange(newKittySize);
  };
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
    (v) => v,
  );

  const options = potentialOptions.filter(
    (v) =>
      !defaultOptions.includes(v) &&
      v < deckLen - props.numPlayers &&
      // Note: this isn't quite right, but it seems fine for the common case of no short decks.
      (deckLen - v) % props.numPlayers <= props.decks.length * 4,
  );

  if (isLoading) {
    return <div>Loading kitty size options...</div>;
  }

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
