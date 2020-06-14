import * as React from 'react';
import classNames from 'classnames';
import Card from './Card';

type Props = {
  id: number;
  className?: string;
  cards: string[];
  moreCards?: string[];
  label: string;
  next?: number | null;
};
const LabeledPlay = (props: Props) => {
  const className = classNames('label', {
    next: props.id === props.next,
  });

  return (
    <div className={classNames('labeled-play', props.className)}>
      <div className="play">
        {props.cards.map((card, idx) => (
          <Card card={card} key={idx} />
        ))}
      </div>
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
