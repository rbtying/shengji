// Small helpers shared by the app screens.
import { cardLookup } from "../util/cardHelpers";
import { Hands, Player, Trump } from "../gen-types";

export interface CardFace {
  card: string;
  rank: string;
  suit: string;
  red: boolean;
  label: string;
  points: number;
}

const SUIT_SYMBOL: { [k: string]: string } = {
  "♡": "♥",
  "♢": "♦",
  "♤": "♠",
  "♧": "♣",
};
const SUIT_NAME: { [k: string]: string } = {
  "♡": "hearts",
  "♢": "diamonds",
  "♤": "spades",
  "♧": "clubs",
};
const RANK_NAME: { [k: string]: string } = {
  A: "Ace",
  K: "King",
  Q: "Queen",
  J: "Jack",
};

/** How a card (its unicode code point, as the engine sends it) is drawn and read aloud. */
export const face = (card: string): CardFace => {
  if (card === "🃏") {
    return {
      card,
      rank: "★",
      suit: "BIG",
      red: true,
      label: "Big joker",
      points: 0,
    };
  }
  if (card === "🃟") {
    return {
      card,
      rank: "★",
      suit: "SM",
      red: false,
      label: "Small joker",
      points: 0,
    };
  }
  const info = cardLookup[card];
  if (info === undefined) {
    return {
      card,
      rank: "",
      suit: "",
      red: false,
      label: "Hidden card",
      points: 0,
    };
  }
  const number = info.number ?? "";
  return {
    card,
    rank: number,
    suit: SUIT_SYMBOL[info.typ] ?? info.typ,
    red: info.typ === "♡" || info.typ === "♢",
    label: `${RANK_NAME[number] ?? number} of ${SUIT_NAME[info.typ] ?? ""}`,
    points: info.points,
  };
};

export const suitSymbol = (engineSuit: string): string =>
  SUIT_SYMBOL[engineSuit] ?? engineSuit;

/** "2♥", or "2 · no trump suit". */
export const trumpLabel = (trump: Trump): { text: string; red: boolean } => {
  if ("Standard" in trump) {
    const s = trump.Standard.suit;
    return {
      text: `${trump.Standard.number}${suitSymbol(s)}`,
      red: s === "♡" || s === "♢",
    };
  }
  return {
    text: `${trump.NoTrump.number ?? ""} no trump`.trim(),
    red: false,
  };
};

/** Bots join as "Ann (hard)"; show just "Ann". */
export const displayName = (name: string): string =>
  name.replace(/ \((beginner|easy|intermediate|hard|expert)\)$/, "");

export const botLevel = (name: string): string | null => {
  const m = name.match(/ \((beginner|easy|intermediate|hard|expert)\)$/);
  return m === null ? null : m[1];
};

/** The cards in a player's hand, one entry per copy. */
export const handCards = (hands: Hands, playerId: number): string[] => {
  const h = hands.hands[playerId] ?? {};
  const out: string[] = [];
  Object.entries(h).forEach(([card, count]) => {
    for (let i = 0; i < count; i++) {
      out.push(card);
    }
  });
  return out;
};

export const pointsIn = (cards: string[]): number =>
  cards.reduce((sum, c) => sum + face(c).points, 0);

/** Players listed starting from `me`, in turn order. */
export const fromMe = (players: Player[], me: number): Player[] => {
  const i = players.findIndex((p) => p.id === me);
  if (i < 0) {
    return players;
  }
  return players.slice(i).concat(players.slice(0, i));
};

/**
 * Where each seat sits around the table, as percentages of the felt, with "you" at the
 * bottom and play moving around the table in turn order.
 */
export const seatPositions = (n: number): Array<{ x: number; y: number }> => {
  if (n === 4) {
    return [
      { x: 50, y: 83 },
      { x: 86, y: 48 },
      { x: 50, y: 16 },
      { x: 14, y: 48 },
    ];
  }
  const out = [];
  for (let i = 0; i < n; i++) {
    // Spread seats around an ellipse, starting at the bottom.
    const angle = Math.PI / 2 - (2 * Math.PI * i) / n;
    // A little flatter than a circle, so the top and bottom seats stay on the felt.
    out.push({ x: 50 + 38 * Math.cos(angle), y: 49 + 33 * Math.sin(angle) });
  }
  return out;
};

export const DIFFICULTIES = [
  {
    value: "Beginner",
    label: "Beginner",
    note: "Plays simple rules and makes deliberate mistakes. Good for your first games.",
  },
  { value: "Easy", label: "Easy", note: "Simple rules, no look-ahead." },
  {
    value: "Intermediate",
    label: "Intermediate",
    note: "Thinks ahead by simulating possible deals.",
  },
  {
    value: "Hard",
    label: "Hard",
    note: "Simulates dozens of deals per move and counts cards carefully.",
  },
  {
    value: "Expert",
    label: "Expert",
    note: "The deepest simulation. Plays to win every point.",
  },
] as const;

/**
 * Which cell of a 3x3 grid on the felt each seat uses, starting with you at the bottom centre
 * and going around the table in playing order. The centre cell is left for the table note.
 */
export const seatCells = (n: number): Array<{ row: number; col: number }> => {
  const layouts: { [n: number]: Array<[number, number]> } = {
    4: [
      [2, 1],
      [1, 2],
      [0, 1],
      [1, 0],
    ],
    5: [
      [2, 1],
      [1, 2],
      [0, 2],
      [0, 0],
      [1, 0],
    ],
    6: [
      [2, 1],
      [1, 2],
      [0, 2],
      [0, 1],
      [0, 0],
      [1, 0],
    ],
    7: [
      [2, 1],
      [2, 2],
      [1, 2],
      [0, 2],
      [0, 1],
      [0, 0],
      [1, 0],
    ],
    8: [
      [2, 1],
      [2, 2],
      [1, 2],
      [0, 2],
      [0, 1],
      [0, 0],
      [1, 0],
      [2, 0],
    ],
  };
  const cells = layouts[Math.min(8, Math.max(4, n))] ?? layouts[4];
  return cells.slice(0, n).map(([row, col]) => ({ row, col }));
};
