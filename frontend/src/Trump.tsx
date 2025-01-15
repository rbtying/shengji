import * as React from "react";
import { Trump } from "./gen-types";
import InlineCard from "./InlineCard";
import preloadedCards from "./preloadedCards";

interface IProps {
  trump: Trump;
}
const TrumpE = (props: IProps): JSX.Element => {
  const { trump } = props;
  if ("Standard" in trump) {
    const { suit, number: rank } = trump.Standard;
    const card = preloadedCards.filter(
      (v) => v.typ === suit && v.number === rank,
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

export default TrumpE;
