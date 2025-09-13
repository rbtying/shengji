import * as React from "react";
import classNames from "classnames";
import Card from "./Card";
import { Trump, Hands, SuitGroup } from "./gen-types";
import ArrayUtils from "./util/array";
import { useEngine } from "./useEngine";
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
  const [selectedCardGroups, setSelectedCardGroups] = React.useState<any[][]>(
    [],
  );
  const [unselectedCardGroups, setUnselectedCardGroups] = React.useState<
    any[][]
  >([]);
  const [isLoading, setIsLoading] = React.useState<boolean>(true);

  const { hands, selectedCards, notifyEmpty } = props;
  const engine = useEngine();
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

  // Create stable string representation of the player's hand for dependency checking
  // This prevents re-running the effect when hands.hands object reference changes
  // but the actual cards remain the same
  const handKey = React.useMemo(() => {
    if (!(props.playerId in hands.hands)) {
      return "";
    }
    // Create a stable key from the hand object (card -> count mapping)
    return Object.entries(hands.hands[props.playerId])
      .sort(([a], [b]) => a.localeCompare(b))
      .map(([card, count]) => `${card}:${count}`)
      .join(",");
  }, [hands.hands, props.playerId]);

  // Load sorted cards when they change
  React.useEffect(() => {
    setIsLoading(true);

    const loadSortedCards = async () => {
      try {
        // Load selected cards groups if needed
        let selectedGroups: any[][] = [];
        if (
          props.selectedCards !== undefined &&
          props.selectedCards.length > 0
        ) {
          const sorted = await engine.sortAndGroupCards({
            cards: props.selectedCards,
            trump: props.trump,
          });
          selectedGroups = sorted.map((g: SuitGroup) =>
            g.cards.map((c) => ({
              card: c,
              suit: g.suit,
            })),
          );
        }

        // Load unselected cards groups
        let unselectedGroups: any[][] = [];
        if (unselected.length > 0) {
          const sorted = await engine.sortAndGroupCards({
            cards: unselected,
            trump: props.trump,
          });
          unselectedGroups = sorted.map((g: SuitGroup) =>
            g.cards.map((c) => ({
              card: c,
              suit: g.suit,
            })),
          );
        }

        // Apply grouping settings
        if (!separateCardsBySuit) {
          selectedGroups =
            selectedGroups.length > 0 ? [selectedGroups.flatMap((g) => g)] : [];
          unselectedGroups =
            unselectedGroups.length > 0
              ? [unselectedGroups.flatMap((g) => g)]
              : [];
        }

        if (reverseCardOrder) {
          unselectedGroups.reverse();
          unselectedGroups.forEach((g) => g.reverse());
        }

        setSelectedCardGroups(selectedGroups);
        setUnselectedCardGroups(unselectedGroups);
        setIsLoading(false);
      } catch (error) {
        console.error("Error sorting cards:", error);
        // Fallback to unsorted display
        const fallbackSelected = props.selectedCards
          ? [props.selectedCards.map((c) => ({ card: c, suit: null }))]
          : [];
        const fallbackUnselected = [
          unselected.map((c) => ({ card: c, suit: null })),
        ];

        setSelectedCardGroups(fallbackSelected);
        setUnselectedCardGroups(fallbackUnselected);
        setIsLoading(false);
      }
    };

    loadSortedCards();
  }, [
    props.selectedCards,
    props.trump,
    props.playerId,
    handKey, // Use the stable key instead of hands.hands
    separateCardsBySuit,
    reverseCardOrder,
    engine,
  ]);

  if (isLoading) {
    return (
      <div className="hand">
        <div>Loading cards...</div>
      </div>
    );
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
