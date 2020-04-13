import * as React from 'react';
import classNames from 'classnames';
import {cardLookup} from './util/cardHelpers';

type Props = {
  card: string;
  className?: string;
  onClick?: (event: React.MouseEvent) => void;
};

const Card = (props: Props) => {
  const cardInfo = cardLookup[props.card];
  if (!cardInfo) {
    return (
      <span className={classNames('card', 'unknown', props.className)}>
        {props.card}
      </span>
    );
  } else {
    return (
      <span
        className={classNames('card', cardInfo.typ, props.className)}
        onClick={props.onClick}
      >
        {cardInfo.display_value}
      </span>
    );
  }
};

export default Card;
