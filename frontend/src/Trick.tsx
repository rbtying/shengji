import * as React from 'react';
import {IPlayer, ITrick} from './types';
import LabeledPlay from './LabeledPlay';
import ArrayUtils from './util/array';

type Props = {
  players: IPlayer[];
  trick: ITrick;
};
const Trick = (props: Props) => {
  const namesById = ArrayUtils.mapObject(props.players, (p: IPlayer) => [
    String(p.id),
    p.name,
  ]);
  const blankCards =
    props.trick.played_cards.length > 0
      ? Array(props.trick.played_cards[0].cards.length).fill('🂠')
      : ['🂠'];
  const betterPlayer =
    props.trick.played_cards.length > 0
      ? props.trick.played_cards[0].better_player
      : null;

  return (
    <div className="trick">
      {props.trick.played_cards.map((played, idx) => {
        const winning = props.trick.current_winner === played.id;
        return (
          <LabeledPlay
            key={idx}
            label={
              winning ? `${namesById[played.id]} (!)` : namesById[played.id]
            }
            className={winning ? 'winning' : ''}
            cards={played.cards}
            moreCards={played.bad_throw_cards}
          />
        );
      })}
      {props.trick.player_queue.map((id, idx) => {
        const better = betterPlayer === id;
        return (
          <LabeledPlay
            key={idx + props.trick.played_cards.length}
            label={better ? `${namesById[id]} (-)` : namesById[id]}
            cards={blankCards}
          />
        );
      })}
    </div>
  );
};

export default Trick;
