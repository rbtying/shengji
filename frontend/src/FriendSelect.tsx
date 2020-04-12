import * as React from 'react';
import {ITrump} from './types';
import ArrayUtils from './util/array';

type FriendSelection = {
  card: string;
  skip: number;
};
type Props = {
  friend: FriendSelection;
  trump: ITrump;
  num_decks: number;
  onChange: (input: FriendSelection) => void;
};

const FriendSelect = (props: Props) => {
  const handleChange = (
    transform: (e: HTMLSelectElement) => Partial<FriendSelection>,
  ) => (event: React.ChangeEvent<HTMLSelectElement>) => {
    event.preventDefault();
    props.onChange({
      card: props.friend.card,
      skip: props.friend.skip,
      ...transform(event.target),
    });
  };

  const handleCardChange = handleChange((select) => ({
    card: select.value,
  }));
  const handleOrdinalChange = handleChange((select) => ({
    skip: parseInt(select.value, 10),
  }));

  const rank = props.trump.Standard
    ? props.trump.Standard.number
    : props.trump.NoTrump.number;
  return (
    <div className="friend-select">
      <select value={props.friend.card} onChange={handleCardChange}>
        <option value=""> </option>
        {(window as any).CARDS.map((c: any) => {
          return c.number !== null && c.number !== rank ? (
            <option
              key={c.value}
              value={c.value}
            >{`${c.number}${c.typ}`}</option>
          ) : null;
        })}
      </select>
      <select value={props.friend.skip} onChange={handleOrdinalChange}>
        {ArrayUtils.range(props.num_decks, (idx) => (
          <option key={idx} value={idx}>
            {idx + 1}
          </option>
        ))}
      </select>
    </div>
  );
};

export default FriendSelect;
