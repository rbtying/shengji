import * as React from 'react';
import {ITrump} from './types';

type Props = {trump: ITrump};
const Trump = (props: Props) => {
  const {trump} = props;
  if (trump.Standard) {
    const {suit, number: rank} = trump.Standard;
    return (
      <div className="trump">
        The trump suit is <span className={suit}>{suit}</span>, rank {rank}
      </div>
    );
  } else {
    return <div className="trump">No trump, rank {trump.NoTrump.number}</div>;
  }
};

export default Trump;
