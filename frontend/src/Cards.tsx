import * as React from "react";
import Card from "./Card";
import classNames from "classnames";
import ArrayUtils from "./util/array";

interface IProps {
  cardsInHand: string[];
  selectedCards?: string[];
  onSelect?: (selected: string[]) => void;
  notifyEmpty?: boolean;
}

const Cards = (props: IProps): JSX.Element => {
  const { cardsInHand, selectedCards, notifyEmpty } = props;
  const handleSelect = (card: string) => () => {
    if (selectedCards !== undefined) {
      props.onSelect([...selectedCards, card]);
    }
  };

  const handleUnselect = (card: string) => () => {
    if (selectedCards !== undefined) {
      const index = selectedCards.indexOf(card);
      if (index >= 0) {
        props.onSelect(ArrayUtils.minus(selectedCards, [card]));
      }
    }
  };

  const unselected =
    selectedCards === undefined
      ? cardsInHand
      : ArrayUtils.minus(cardsInHand, selectedCards);
  return (
    <div className="hand">
      {props.selectedCards !== undefined ? (
        <div className="selected-cards">
          {selectedCards.map((c, idx) => (
            <Card key={idx} onClick={handleUnselect(c)} card={c} />
          ))}
          {selectedCards.length === 0 && (
            <Card card="🂠" className={classNames({ notify: notifyEmpty })} />
          )}
        </div>
      ) : null}
      <div
        className={classNames("unselected-cards", {
          unclickable: props.onSelect === undefined,
        })}
      >
        {unselected.map((c, idx) => (
          <Card
            key={idx}
            onClick={props.onSelect !== undefined ? handleSelect(c) : null}
            card={c}
          />
        ))}
        {unselected.length === 0 && <Card card="🂠" />}
      </div>
    </div>
  );
};

export default Cards;
