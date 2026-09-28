import * as React from "react";
import MiniCard from "./MiniCard";
import { useCardSize } from "./cardStyle";

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

const SUIT_GAP = 8;

/** Card width and the least of each card that must show (its corner index), by card size. */
const SIZES = {
  normal: { cardW: 50, minStep: 19, maxStep: 34 },
  large: { cardW: 64, minStep: 25, maxStep: 42 },
};

/** Split groups into rows that fit, trump first, never splitting a suit across rows. */
const layoutRows = (
  groups: CardGroup[],
  width: number,
  cardW: number,
  minStep: number,
): CardGroup[][] => {
  const fits = (row: CardGroup[]): boolean => {
    const n = row.reduce((s, g) => s + g.cards.length, 0);
    if (n <= 1) {
      return true;
    }
    const step = (width - cardW - SUIT_GAP * (row.length - 1)) / (n - 1);
    return step >= minStep;
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

  const { cardW, minStep, maxStep } = SIZES[useCardSize()];
  // Two rows always fit a full hand; a third would only take room from the felt.
  const maxRows = 2;
  const groups = props.groups.filter((g) => g.cards.length > 0);
  const rows = layoutRows(groups, width, cardW, minStep);
  // Short screens keep the hand to two rows (or fewer): re-flow the suits, in order, into
  // that many rows so the biggest row is as small as possible. Cards overlap a little more
  // instead, and the felt keeps its room.
  if (rows.length > maxRows) {
    const flat = rows.flat();
    const count = (gs: CardGroup[]) =>
      gs.reduce((n, g) => n + g.cards.length, 0);
    let best: CardGroup[][] | null = null;
    const search = (start: number, left: number, acc: CardGroup[][]): void => {
      if (left === 1) {
        const candidate = [...acc, flat.slice(start)];
        const worst = Math.max(...candidate.map(count));
        if (best === null || worst < Math.max(...best.map(count))) {
          best = candidate;
        }
        return;
      }
      for (let end = start + 1; end <= flat.length - left + 1; end++) {
        search(end, left - 1, [...acc, flat.slice(start, end)]);
      }
    };
    search(0, Math.min(maxRows, flat.length), []);
    if (best !== null) {
      rows.splice(0, rows.length, ...(best as CardGroup[][]));
    }
  }
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
            ? cardW
            : Math.min(
                maxStep,
                (width - cardW - SUIT_GAP * (row.length - 1)) / (n - 1),
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
                    ? step - cardW + SUIT_GAP
                    : step - cardW;
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
