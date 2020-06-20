import * as React from "react";

import classNames from "classnames";
import Card from "./Card";

interface IProps {
  id?: number | null;
  className?: string;
  cards?: string[];
  groupedCards?: string[][];
  moreCards?: string[];
  label: string;
  next?: number | null;
}
const LabeledPlay = (props: IProps) => {
  const className = classNames("label", {
    next:
      props.next !== undefined &&
      props.next !== null &&
      props.id === props.next,
  });

  const cards = props.cards.map((card, idx) => <Card card={card} key={idx} />);

  const groupedCards =
    props.groupedCards?.map((c, gidx) => (
      <div className="card-group" key={gidx}>
        {c.map((card, idx) => (
          <Card card={card} key={gidx + "-" + idx} />
        ))}
      </div>
    )) || cards;

  return (
    <div className={classNames("labeled-play", props.className)}>
      <div className="play">{groupedCards}</div>
      {props.moreCards && props.moreCards.length > 0 ? (
        <div className="play more">
          {props.moreCards.map((card, idx) => (
            <Card card={card} key={idx} />
          ))}
        </div>
      ) : null}
      <div className={className}>{props.label}</div>
    </div>
  );
};

export default LabeledPlay;
