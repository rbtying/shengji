import * as React from 'react';
import classNames from 'classnames';
import { IPlayer, ITrick, IPlayedCards } from './types';
import LabeledPlay from './LabeledPlay';
import ArrayUtils from './util/array';

type Props = {
  players: IPlayer[];
  landlord?: number | null;
  landlords_team?: number[];
  trick: ITrick;
  next?: number | null;
  name: string;
  showTrickInPlayerOrder: boolean;
};
const Trick = (props: Props) => {
  const namesById = ArrayUtils.mapObject(props.players, (p: IPlayer) => [
    String(p.id),
    (p.id === props.landlord) ? (p.name + ' (landlord)') : (p.name),
  ]);
  const blankCards =
    props.trick.played_cards.length > 0
      ? Array(props.trick.played_cards[0].cards.length).fill('🂠')
      : ['🂠'];
  const betterPlayer =
    props.trick.played_cards.length > 0
      ? props.trick.played_cards[0].better_player
      : null;

  const playedByID: { [id: number]: IPlayedCards } = {};
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

        const className = classNames(winning
          ? 'winning'
          : props.trick.player_queue[0] === id
            ? 'notify'
            : '', {
          landlord:
            id === props.landlord || props.landlords_team?.includes(id),
          next: id === props.next,
          you: props.name === props.players[id].name,
        });

        return (
          <LabeledPlay
            key={id}
            label={namesById[id] + suffix}
            className={className}
            cards={cards}
            moreCards={playedByID[id]?.bad_throw_cards}
          />
        );
      })}
    </div>
  );
};

export default Trick;
