import * as React from 'react';
import {ITrump} from './types';

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

  const number = props.trump.Standard
    ? props.trump.Standard.number
    : props.trump.NoTrump.number;
  return (
    <div className="friend-select">
      <select value={props.friend.card} onChange={handleCardChange}>
        <option value=""> </option>
        {(window as any).CARDS.map((c: any) => {
          return c.number != null && c.number != number ? (
            <option
              key={c.value}
              value={c.value}
            >{`${c.number}${c.typ}`}</option>
          ) : null;
        })}
      </select>
      <select value={props.friend.skip} onChange={handleOrdinalChange}>
        {Array(props.num_decks)
          .fill(1)
          .map((_, idx) => (
            <option key={idx} value={idx}>
              {idx + 1}
            </option>
          ))}
      </select>
    </div>
  );
};

export default FriendSelect;
