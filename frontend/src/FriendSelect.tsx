import * as React from 'react';
import Select from 'react-select';
import {ITrump} from './types';
import ArrayUtils from './util/array';
import preloadedCards from './preloadedCards';
import InlineCard from './InlineCard';
import {cardLookup} from './util/cardHelpers';

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
type Option = {
  value: string;
  label: string;
};

const FriendSelect = (props: Props) => {
  const handleChange = (transform: (e: Option) => Partial<FriendSelection>) => (
    value: Option,
  ) => {
    props.onChange({
      card: props.friend.card,
      skip: props.friend.skip,
      ...transform(value),
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

  const cardOptions: Option[] = [];
  const currentValue: {[s: string]: any} = {};
  if (props.friend.card !== '') {
    const c = cardLookup[props.friend.card];
    currentValue.label = `${c.number}${c.typ}`;
    currentValue.value = c.value;
  }

  preloadedCards.forEach((c) => {
    if (c.number !== null && c.number !== rank) {
      cardOptions.push({
        label: `${c.number}${c.typ}`,
        value: c.value,
      });
    }
  });

  return (
    <div className="friend-select">
      <div style={{width: '100px', display: 'inline-block'}}>
        <Select
          value={currentValue}
          onChange={handleCardChange}
          options={cardOptions}
          formatOptionLabel={({value}) =>
            value ? <InlineCard card={value} /> : value
          }
        />
      </div>
      <div
        style={{width: '100px', display: 'inline-block', marginLeft: '10px'}}
      >
        <Select
          value={
            props.friend.skip !== null
              ? {
                  value: `${props.friend.skip}`,
                  label: `#${props.friend.skip + 1}`,
                }
              : {}
          }
          onChange={handleOrdinalChange}
          options={ArrayUtils.range(props.num_decks, (idx) => {
            return {value: `${idx}`, label: `#${idx + 1}`};
          })}
        />
      </div>
    </div>
  );
};

export default FriendSelect;
