import * as React from 'react';
import {ITrump} from './types';
import InlineCard from './InlineCard';
import preloadedCards from './preloadedCards';

type Props = {trump: ITrump};
const Trump = (props: Props) => {
  const {trump} = props;
  if (trump.Standard) {
    const {suit, number: rank} = trump.Standard;
    const card = preloadedCards.filter(
      (v) => v.typ === suit && v.number === rank,
    )[0].value;
    return (
      <div className="trump">
        The trump suit is <InlineCard card={card} /> (rank {rank})
      </div>
    );
  } else {
    return <div className="trump">No trump, rank {trump.NoTrump.number}</div>;
  }
};

export default Trump;
