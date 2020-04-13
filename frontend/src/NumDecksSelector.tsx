import * as React from 'react';
import ArrayUtils from './util/array';

type Props = {
  numPlayers: number;
  numDecks: number | null;
  onChange: (newNumDecks: number | null) => void;
};

const NumDecksSelector = (props: Props) => {
  const handleChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const newNumDecks =
      e.target.value === '' ? null : parseInt(e.target.value, 10);
    props.onChange(newNumDecks);
  };

  return (
    <div className="num-decks-picker">
      <label>
        Number of decks:{' '}
        <select
          value={props.numDecks === null ? '' : props.numDecks}
          onChange={handleChange}
        >
          <option value="">default</option>
          {ArrayUtils.range(props.numPlayers, (idx) => {
            const val = idx + 1;
            return (
              <option value={val} key={idx}>
                {val}
              </option>
            );
          })}
        </select>
      </label>
    </div>
  );
};

export default NumDecksSelector;
