import * as React from 'react';
import classNames from 'classnames';
import Card from './Card';

type Props = {
  className?: string;
  cards: string[];
  label: string;
};
const LabeledPlay = (props: Props) => {
  return (
    <div className={classNames('labeled-play', props.className)}>
      <div className="play">
        {props.cards.map((card, idx) => (
          <Card card={card} key={idx} />
        ))}
      </div>
      <div className="label">{props.label}</div>
    </div>
  );
};

export default LabeledPlay;
