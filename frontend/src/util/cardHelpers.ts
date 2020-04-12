// prettier-ignore
type Rank = (
  | 'A' | '2' | '3' | '4' | '5' | '6' | '7'
  | '8' | '9' | 'T' | 'J' | 'Q' | 'K'
);
type Suit = 'diamonds' | 'clubs' | 'hearts' | 'spades';

export type SuitCard = {
  type: 'suit_card';
  rank: Rank;
  suit: Suit;
};

type Card = SuitCard | {type: 'big_joker'} | {type: 'little_joker'};

// prettier-ignore
const orderedRanks: (Rank | null)[] = [
  'A', '2', '3', '4', '5', '6', '7',
  '8', '9', 'T', 'J', null, 'Q', 'K',
];
const suitsToUnicodeOffsets: {suit: Suit; offset: number}[] = [
  {suit: 'spades', offset: 56481},
  {suit: 'hearts', offset: 56497},
  {suit: 'diamonds', offset: 56513},
  {suit: 'clubs', offset: 56529},
];

export const unicodeToCard = (unicode: string): Card => {
  if (unicode === '🃟') {
    return {type: 'little_joker'};
  }
  if (unicode === '🃏') {
    return {type: 'big_joker'};
  }
  const first = unicode.charCodeAt(0);
  const second = unicode.charCodeAt(1);
  if (first === 55356) {
    const suitAndOffset = suitsToUnicodeOffsets.find(
      (entry) => second >= entry.offset && second < entry.offset + 14,
    );
    if (suitAndOffset && unicode.length === 2) {
      const rank = orderedRanks[second - suitAndOffset.offset];
      if (rank) {
        return {
          type: 'suit_card',
          rank,
          suit: suitAndOffset.suit,
        };
      }
    }
  }
  throw new Error(`Invalid card string: ${unicode}`);
};

export const cardToUnicodeSuit = (card: SuitCard): string => {
  switch (card.suit) {
    case 'diamonds':
      return '♢';
    case 'clubs':
      return '♧';
    case 'hearts':
      return '♥';
    case 'spades':
      return '♤';
  }
};
