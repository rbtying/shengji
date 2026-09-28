import * as React from "react";
import { useEngine } from "../useEngine";
import { Trump } from "../gen-types";
import { CardGroup } from "./HandTray";

const LABELS: { [k: string]: string } = {
  Trump: "Trump",
  Spades: "Spades",
  Hearts: "Hearts",
  Clubs: "Clubs",
  Diamonds: "Diamonds",
};

/** Sorts cards and groups them by effective suit, trump first. */
export const useHandGroups = (cards: string[], trump: Trump): CardGroup[] => {
  const engine = useEngine();
  const [groups, setGroups] = React.useState<CardGroup[]>([]);
  const key = JSON.stringify([cards.slice().sort(), trump]);
  React.useEffect(() => {
    let live = true;
    engine
      .sortAndGroupCards({ cards, trump })
      .then((result) => {
        if (!live) {
          return;
        }
        const g = result.map((sg) => ({
          label: LABELS[sg.suit] ?? sg.suit,
          cards: sg.cards as string[],
        }));
        g.sort((a, b) =>
          a.label === "Trump" ? -1 : b.label === "Trump" ? 1 : 0,
        );
        setGroups(g);
      })
      .catch(() => {
        if (live) {
          setGroups([{ label: "Hand", cards }]);
        }
      });
    return () => {
      live = false;
    };
  }, [key]);
  return groups;
};
