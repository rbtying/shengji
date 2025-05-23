import * as React from "react";

import { Tooltip } from "react-tooltip";
import classNames from "classnames";

import LabeledPlay from "./LabeledPlay";
import { PlayedCards, Player, Trick } from "./gen-types";
import ArrayUtils from "./util/array";

import type { JSX } from "react";

interface IProps {
  players: Player[];
  landlord?: number | null;
  landlord_suffix: string;
  landlords_team?: number[];
  trick: Trick;
  next?: number | null;
  name: string;
  showTrickInPlayerOrder: boolean;
}
const TrickE = (props: IProps): JSX.Element => {
  const namesById = ArrayUtils.mapObject(props.players, (p: Player) => [
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

  const playedByID: { [id: number]: PlayedCards } = {};
  const cardsFromMappingByID: { [id: number]: string[][] } = {};
  let playOrder: number[] = [];

  props.trick.played_cards.forEach((played, idx) => {
    playOrder.push(played.id);
    playedByID[played.id] = played;
    const m = props.trick.played_card_mappings
      ? props.trick.played_card_mappings[idx]
      : undefined;
    if (m !== undefined && m !== null && m.length > 0) {
      // We should coalesce blocks of `Repeated` of count 1 together, since
      // that displays more nicely.
      const mapping: string[][] = [];
      const singles: string[] = [];

      m.forEach((mm) => {
        if ("Repeated" in mm && mm.Repeated.count === 1) {
          singles.push(mm.Repeated.card.card);
        } else if ("Repeated" in mm) {
          mapping.push(
            ArrayUtils.range(mm.Repeated.count, (_) => mm.Repeated.card.card),
          );
        } else if ("Tractor" in mm) {
          mapping.push(
            mm.Tractor.members.flatMap((mmm) =>
              ArrayUtils.range(mm.Tractor.count, (_) => mmm.card),
            ),
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
        const suffix = winning ? (
          <>
            {" "}
            <Tooltip id="winningTip" place="bottom" />
            <span
              data-tooltip-id="winningTip"
              data-tooltip-content="Current winner of trick"
            >
              (<code>!</code>)
            </span>
          </>
        ) : better ? (
          <>
            {" "}
            <Tooltip id="betterTip" place="bottom" />
            <span
              data-tooltip-id="betterTip"
              data-tooltip-content="First player who can prevent the attempted throw"
            >
              (<code>-</code>)
            </span>
          </>
        ) : (
          <></>
        );

        const className = classNames(
          winning
            ? "winning"
            : props.trick.player_queue[0] === id
              ? "notify"
              : "",
          {
            landlord:
              id === props.landlord || props.landlords_team?.includes(id),
          },
        );

        return (
          <LabeledPlay
            key={id}
            id={id}
            label={
              <>
                {namesById[id] +
                  (id === props.landlord ? " " + props.landlord_suffix : "")}
                {suffix}
              </>
            }
            className={className}
            groupedCards={cardsFromMappingByID[id]}
            cards={cards}
            trump={props.trick.trump}
            next={props.next}
            moreCards={playedByID[id]?.bad_throw_cards}
          />
        );
      })}
    </div>
  );
};

export default TrickE;
