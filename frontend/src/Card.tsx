import * as React from "react";

import classNames from "classnames";
import InlineCard from "./InlineCard";
import { cardLookup } from "./util/cardHelpers";

interface IProps {
  card: string;
  className?: string;
  onClick?: (event: React.MouseEvent) => void;
}

const Card = (props: IProps): JSX.Element => {
  if (!(props.card in cardLookup)) {
    return (
      <span className={classNames("card", "unknown", props.className)}>
        {props.card}
      </span>
    );
  } else {
    const cardInfo = cardLookup[props.card];
    return (
      <span
        className={classNames("card", cardInfo.typ, props.className)}
        onClick={props.onClick}
      >
        <div className="card-label">
          <InlineCard card={props.card} />
        </div>
        {cardInfo.display_value}
      </span>
    );
  }
};

export default Card;
