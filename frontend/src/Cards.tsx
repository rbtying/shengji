import * as React from "react";
import classNames from "classnames";
import Card from "./Card";
import { ITrump, IHands } from "./types";
import ArrayUtils from "./util/array";
import WasmContext from "./WasmContext";
import { SettingsContext } from "./AppStateProvider";

interface IProps {
  hands: IHands;
  trump: ITrump;
  playerId: number;
  selectedCards?: string[];
  onSelect?: (selected: string[]) => void;
  onCardClick?: (card: string) => void;
  notifyEmpty?: boolean;
}

const Cards = (props: IProps): JSX.Element => {
  const [highlightedSuit, setHighlightedSuit] = React.useState<string | null>(
    null
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
      if (index >= 0) {
        props.onSelect(ArrayUtils.minus(selectedCards, [card]));
      }
    }
  };

  const cardsInHand =
    props.playerId in hands.hands
      ? Object.entries(hands.hands[props.playerId]).flatMap(([c, ct]) =>
          Array(ct).fill(c)
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
          }))
        )
      : [];

  let unselectedCardGroups = sortAndGroupCards({
    cards: unselected,
    trump: props.trump,
  }).map((g) =>
    g.cards.map((c) => ({
      card: c,
      suit: g.suit,
    }))
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
                  card={c.card}
                />
              ))}
            </div>
          ))}
          {props.selectedCards.length === 0 && (
            <Card card="🂠" className={classNames({ notify: notifyEmpty })} />
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
                    : null
                )}
                onClick={handleSelect(c.card)}
                card={c.card}
                onMouseEnter={(_) => setHighlightedSuit(c.suit)}
                onMouseLeave={(_) => setHighlightedSuit(null)}
              />
            ))}
          </div>
        ))}
        {unselectedCardGroups.length === 0 && <Card card="🂠" />}
      </div>
    </div>
  );
};

export default Cards;
