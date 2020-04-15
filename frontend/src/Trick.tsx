import * as React from 'react';
import {IPlayer, ITrick, IPlayedCards} from './types';
import LabeledPlay from './LabeledPlay';
import ArrayUtils from './util/array';

type Props = {
  players: IPlayer[];
  trick: ITrick;
  showTrickInPlayerOrder: boolean;
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

  const playedByID: {[id: number]: IPlayedCards} = {};
  let playOrder: number[] = [];

  props.trick.played_cards.forEach((played) => {
    playOrder.push(played.id);
    playedByID[played.id] = played;
  });

  if (props.showTrickInPlayerOrder) {
    playOrder = props.players.map((p) => p.id);
  } else {
    props.trick.player_queue.forEach((id) => playOrder.push(id));
  }

  return (
    <div className="trick">
      {playOrder.map((id) => {
        const winning = props.trick.current_winner === id;
        const better = betterPlayer === id;
        const cards = playedByID[id]?.cards || blankCards;
        const suffix = winning ? ' (!)' : better ? ' (-)' : '';

        return (
          <LabeledPlay
            key={id}
            label={namesById[id] + suffix}
            className={
              winning
                ? 'winning'
                : props.trick.player_queue[0] === id
                ? 'notify'
                : ''
            }
            cards={cards}
            moreCards={playedByID[id]?.bad_throw_cards}
          />
        );
      })}
    </div>
  );
};

export default Trick;
