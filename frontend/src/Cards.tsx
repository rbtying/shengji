import * as React from "react";
import classNames from "classnames";
import Card from "./Card";
import { Trump, Hands } from "./gen-types";
import ArrayUtils from "./util/array";
import WasmContext from "./WasmContext";
import { SettingsContext } from "./AppStateProvider";

import type { JSX } from "react";

interface IProps {
  hands: Hands;
  trump: Trump;
  playerId: number;
  selectedCards?: string[];
  onSelect?: (selected: string[]) => void;
  onCardClick?: (card: string) => void;
  notifyEmpty?: boolean;
}

const Cards = (props: IProps): JSX.Element => {
  const [highlightedSuit, setHighlightedSuit] = React.useState<string | null>(
    null,
  );

  const { hands, selectedCards, notifyEmpty } = props;
  const { sortAndGroupCards } = React.useContext(WasmContext);
  const { separateCardsBySuit, disableSuitHighlights, reverseCardOrder } =
    React.useContext(SettingsContext);
  const handleSelect = (card: string) => () => {
    if (props.onCardClick !== undefined) {
      props.onCardClick(card);
    }
    if (selectedCards !== undefined && props.onSelect !== undefined) {
      props.onSelect([...selectedCards, card]);
    }
  };

  const handleUnselect = (card: string) => () => {
    if (selectedCards !== undefined) {
      const index = selectedCards.indexOf(card);
      if (index >= 0 && props.onSelect) {
        props.onSelect(ArrayUtils.minus(selectedCards, [card]));
      }
    }
  };

  const cardsInHand =
    props.playerId in hands.hands
      ? Object.entries(hands.hands[props.playerId]).flatMap(([c, ct]) =>
          Array(ct).fill(c),
        )
      : [];

  const unselected =
    selectedCards === undefined
      ? cardsInHand
      : ArrayUtils.minus(cardsInHand, selectedCards);

  let selectedCardGroups =
    props.selectedCards !== undefined
      ? sortAndGroupCards({
          cards: props.selectedCards,
          trump: props.trump,
        }).map((g) =>
          g.cards.map((c) => ({
            card: c,
            suit: g.suit,
          })),
        )
      : [];

  let unselectedCardGroups = sortAndGroupCards({
    cards: unselected,
    trump: props.trump,
  }).map((g) =>
    g.cards.map((c) => ({
      card: c,
      suit: g.suit,
    })),
  );

  if (!separateCardsBySuit) {
    selectedCardGroups = [selectedCardGroups.flatMap((g) => g)];
    unselectedCardGroups = [unselectedCardGroups.flatMap((g) => g)];
  }

  if (reverseCardOrder) {
    unselectedCardGroups.reverse();
    unselectedCardGroups.forEach((g) => g.reverse());
  }

  return (
    <div className="hand">
      {props.selectedCards !== undefined ? (
        <div className="selected-cards">
          {selectedCardGroups.map((g, gidx) => (
            <div style={{ display: "inline-block" }} key={gidx}>
              {g.map((c, idx) => (
                <Card
                  key={`${gidx}-${idx}`}
                  onClick={handleUnselect(c.card)}
                  trump={props.trump}
                  card={c.card}
                  collapseRight={idx !== g.length - 1}
                />
              ))}
            </div>
          ))}
          {props.selectedCards.length === 0 && (
            <Card
              card="🂠"
              trump={props.trump}
              className={classNames({ notify: notifyEmpty })}
            />
          )}
        </div>
      ) : null}
      <div
        className={classNames("unselected-cards", {
          unclickable:
            props.onSelect === undefined && props.onCardClick === undefined,
        })}
      >
        {unselectedCardGroups.map((g, gidx) => (
          <div style={{ display: "inline-block" }} key={gidx}>
            {g.map((c, idx) => (
              <Card
                key={`${gidx}-${idx}`}
                className={classNames(
                  !disableSuitHighlights && highlightedSuit === c.suit
                    ? "highlighted"
                    : null,
                )}
                onClick={handleSelect(c.card)}
                card={c.card}
                collapseRight={idx !== g.length - 1}
                trump={props.trump}
                onMouseEnter={(_) => setHighlightedSuit(c.suit)}
                onMouseLeave={(_) => setHighlightedSuit(null)}
              />
            ))}
          </div>
        ))}
        {unselectedCardGroups.length === 0 && (
          <Card trump={props.trump} card="🂠" />
        )}
      </div>
    </div>
  );
};

export default Cards;
