import * as React from "react";
import { ITrump } from "./types";
import InlineCard from "./InlineCard";
import preloadedCards from "./preloadedCards";

interface IProps {
  trump: ITrump;
}
const Trump = (props: IProps): JSX.Element => {
  const { trump } = props;
  if (trump.Standard !== undefined) {
    const { suit, number: rank } = trump.Standard;
    const card = preloadedCards.filter(
      (v) => v.typ === suit && v.number === rank
    )[0].value;
    return (
      <div className="trump">
        The trump suit is <InlineCard card={card} /> (rank {rank})
      </div>
    );
  } else if (
    trump.NoTrump.number !== undefined &&
    trump.NoTrump.number !== null
  ) {
    return <div className="trump">No trump, rank {trump.NoTrump.number}</div>;
  } else {
    return <div className="trump">No trump</div>;
  }
};

export default Trump;
