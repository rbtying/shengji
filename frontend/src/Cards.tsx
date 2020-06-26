import * as React from "react";
import Card from "./Card";
import classNames from "classnames";
import ArrayUtils from "./util/array";
import { cardLookup, unicodeToCard } from "./util/cardHelpers";

interface IProps {
  cardsInHand: string[];
  selectedCards: string[];
  onSelect: (selected: string[]) => void;
  notifyEmpty?: boolean;
  separateBidCards?: boolean;
  level?: string;
}

const Cards = (props: IProps): JSX.Element => {
  const {
    cardsInHand,
    selectedCards,
    notifyEmpty,
    separateBidCards,
    level,
  } = props;
  const handleSelect = (card: string) => () => {
    props.onSelect([...selectedCards, card]);
  };

  const handleUnselect = (card: string) => () => {
    const index = selectedCards.indexOf(card);
    if (index >= 0) {
      props.onSelect(ArrayUtils.minus(selectedCards, [card]));
    }
  };

  let unselected = ArrayUtils.minus(cardsInHand, selectedCards);
  const bidCards = separateBidCards
    ? unselected.filter(
        (card) =>
          unicodeToCard(card).type === "big_joker" ||
          unicodeToCard(card).type === "little_joker" ||
          (unicodeToCard(card).type === "suit_card" &&
            cardLookup[card].number === level)
      )
    : [];

  let bidCardComponent;
  if (separateBidCards) {
    unselected = ArrayUtils.minus(unselected, bidCards);
    if (bidCards.length === 0) {
      bidCardComponent = null;
    } else {
      bidCardComponent = (
        <div className="unselected-cards">
          <div>Available Bid Cards</div>
          {bidCards.map((c, idx) => (
            <Card key={idx} onClick={handleSelect(c)} card={c} />
          ))}
        </div>
      );
    }
  } else {
    bidCardComponent = null;
  }

  console.log("bidCards: " + JSON.stringify(bidCards));
  return (
    <div className="hand">
      <div className="selected-cards">
        {selectedCards.map((c, idx) => (
          <Card key={idx} onClick={handleUnselect(c)} card={c} />
        ))}
        {selectedCards.length === 0 && (
          <Card card="🂠" className={classNames({ notify: notifyEmpty })} />
        )}
      </div>
      {bidCardComponent}
      <div className="unselected-cards">
        {unselected.map((c, idx) => (
          <Card key={idx} onClick={handleSelect(c)} card={c} />
        ))}
        {unselected.length === 0 && <Card card="🂠" />}
      </div>
    </div>
  );
};

export default Cards;
