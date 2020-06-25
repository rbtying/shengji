import * as React from "react";
import Card from "./Card";
import classNames from "classnames";
import ArrayUtils from "./util/array";

interface IProps {
  cardsInHand: string[];
  selectedCards: string[];
  onSelect: (selected: string[]) => void;
  notifyEmpty?: boolean;
}

const Cards = (props: IProps): JSX.Element => {
  const { cardsInHand, selectedCards, notifyEmpty } = props;
  const handleSelect = (card: string) => () => {
    props.onSelect([...selectedCards, card]);
  };

  const handleUnselect = (card: string) => () => {
    const index = selectedCards.indexOf(card);
    if (index >= 0) {
      props.onSelect(ArrayUtils.minus(selectedCards, [card]));
    }
  };

  const unselected = ArrayUtils.minus(cardsInHand, selectedCards);

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
