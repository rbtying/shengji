import * as React from "react";
import MiniCard from "./MiniCard";

import type { JSX } from "react";

export interface CardGroup {
  label: string;
  cards: string[];
}

interface IProps {
  groups: CardGroup[];
  /** Keys of selected cards, "<card>#<copy>". */
  selected: string[];
  suggested?: string[];
  onTap?: (card: string, key: string) => void;
  title?: string;
}

const CARD_W = 44;
const SUIT_GAP = 8;

/** Split groups into rows that fit, trump first, never splitting a suit across rows. */
const layoutRows = (groups: CardGroup[], width: number): CardGroup[][] => {
  const fits = (row: CardGroup[]): boolean => {
    const n = row.reduce((s, g) => s + g.cards.length, 0);
    if (n <= 1) {
      return true;
    }
    const step = (width - CARD_W - SUIT_GAP * (row.length - 1)) / (n - 1);
    return step >= 17;
  };
  const rows: CardGroup[][] = [];
  groups.forEach((g) => {
    const last = rows[rows.length - 1];
    // Trump always gets its own row when it's big; side suits share rows while they fit.
    if (last !== undefined && last[0].label !== "Trump" && fits([...last, g])) {
      last.push(g);
    } else {
      rows.push([g]);
    }
  });
  return rows;
};

/** Your hand, grouped by suit (trump first) in overlapping rows sized to the screen. */
const HandTray = (props: IProps): JSX.Element => {
  const ref = React.useRef<HTMLDivElement>(null);
  const [width, setWidth] = React.useState<number>(360);
  React.useEffect(() => {
    const el = ref.current;
    if (el === null) {
      return;
    }
    // Room for cards: the tray's width minus its side padding.
    const measure = (): void => {
      const cs = window.getComputedStyle(el);
      setWidth(
        el.clientWidth -
          parseFloat(cs.paddingLeft || "0") -
          parseFloat(cs.paddingRight || "0"),
      );
    };
    measure();
    const ro = new ResizeObserver(measure);
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  const groups = props.groups.filter((g) => g.cards.length > 0);
  const rows = layoutRows(groups, width);
  const total = groups.reduce((s, g) => s + g.cards.length, 0);
  const seen: { [card: string]: number } = {};

  return (
    <div className="tta-tray" ref={ref}>
      <div className="tta-tray-head">
        <span>{props.title ?? "Your hand"}</span>
        <span>{total} cards</span>
      </div>
      {rows.map((row, ri) => {
        const n = row.reduce((s, g) => s + g.cards.length, 0);
        const step =
          n <= 1
            ? CARD_W
            : Math.min(
                30,
                (width - CARD_W - SUIT_GAP * (row.length - 1)) / (n - 1),
              );
        return (
          <div className="tta-tray-row" key={ri}>
            {row.map((g, gi) =>
              g.cards.map((card, ci) => {
                seen[card] = (seen[card] ?? 0) + 1;
                const key = `${card}#${seen[card]}`;
                const first = gi === 0 && ci === 0;
                const marginLeft = first
                  ? 0
                  : ci === 0
                    ? step - CARD_W + SUIT_GAP
                    : step - CARD_W;
                return (
                  <MiniCard
                    key={key}
                    card={card}
                    pressed={props.selected.includes(key)}
                    suggested={props.suggested?.includes(key)}
                    onClick={
                      props.onTap !== undefined
                        ? () => props.onTap!(card, key)
                        : undefined
                    }
                    style={{ marginLeft }}
                  />
                );
              }),
            )}
          </div>
        );
      })}
    </div>
  );
};

export default HandTray;
