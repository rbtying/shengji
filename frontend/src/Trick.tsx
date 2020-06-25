import * as React from "react";

import classNames from "classnames";

import LabeledPlay from "./LabeledPlay";
import { IPlayedCards, IPlayer, ITrick } from "./types";
import ArrayUtils from "./util/array";

interface IProps {
  players: IPlayer[];
  landlord?: number | null;
  landlord_suffix: string;
  landlords_team?: number[];
  trick: ITrick;
  next?: number | null;
  name: string;
  showTrickInPlayerOrder: boolean;
}
const Trick = (props: IProps): JSX.Element => {
  const namesById = ArrayUtils.mapObject(props.players, (p: IPlayer) => [
    String(p.id),
    p.name,
  ]);
  const blankCards =
    props.trick.played_cards.length > 0
      ? Array(props.trick.played_cards[0].cards.length).fill("🂠")
      : ["🂠"];
  const betterPlayer =
    props.trick.played_cards.length > 0
      ? props.trick.played_cards[0].better_player
      : null;

  const playedByID: { [id: number]: IPlayedCards } = {};
  const cardsFromMappingByID: { [id: number]: string[][] } = {};
  let playOrder: number[] = [];

  props.trick.played_cards.forEach((played, idx) => {
    playOrder.push(played.id);
    playedByID[played.id] = played;
    const m = props.trick.played_card_mappings[idx];
    if (m !== undefined && m !== null && m.length > 0) {
      // We should coalesce blocks of `Repeated` of count 1 together, since
      // that displays more nicely.
      const mapping: string[][] = [];
      const singles: string[] = [];

      m.forEach((mm) => {
        if (mm.Repeated !== undefined && mm.Repeated.count === 1) {
          singles.push(mm.Repeated.card.card);
        } else if (mm.Repeated !== undefined) {
          mapping.push(
            ArrayUtils.range(mm.Repeated.count, (_) => mm.Repeated.card.card)
          );
        } else if (mm.Tractor !== undefined) {
          mapping.push(
            mm.Tractor.members.flatMap((mmm) =>
              ArrayUtils.range(mm.Tractor.count, (_) => mmm.card)
            )
          );
        }
      });
      mapping.push(singles);

      cardsFromMappingByID[played.id] = mapping;
    }
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
        const cards = id in playedByID ? playedByID[id].cards : blankCards;
        const suffix = winning ? " (!)" : better ? " (-)" : "";

        const className = classNames(
          winning
            ? "winning"
            : props.trick.player_queue[0] === id
            ? "notify"
            : "",
          {
            landlord:
              id === props.landlord || props.landlords_team?.includes(id),
          }
        );

        return (
          <LabeledPlay
            key={id}
            id={id}
            label={
              namesById[id] +
              (id === props.landlord ? " " + props.landlord_suffix : "") +
              suffix
            }
            className={className}
            groupedCards={cardsFromMappingByID[id]}
            cards={cards}
            next={props.next}
            moreCards={playedByID[id]?.bad_throw_cards}
          />
        );
      })}
    </div>
  );
};

export default Trick;
