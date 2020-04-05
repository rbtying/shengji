import * as React from 'react';
import {ICardInfo} from './types';
import classNames from 'classnames';

type Props = {
  card: string;
  className?: string;
  onClick?: (event: React.MouseEvent) => void;
};
const Card = (props: Props) => {
  const cardInfo = (window as any).CARD_LUT[props.card] as ICardInfo;
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
