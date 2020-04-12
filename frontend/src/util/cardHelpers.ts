// prettier-ignore
type Rank = (
  | 'A' | '2' | '3' | '4' | '5' | '6' | '7'
  | '8' | '9' | 'T' | 'J' | 'Q' | 'K'
);
type Suit = 'diamonds' | 'clubs' | 'hearts' | 'spades';

const suitToUnicode: {[key in Suit]: string} = {
  clubs: '♧',
  diamonds: '♢',
  hearts: '♡',
  spades: '♤',
};
const suitToFilledUnicode: {[key in Suit]: string} = {
  clubs: '♣',
  diamonds: '♦',
  hearts: '♥',
  spades: '♠',
};

export type SuitCard = {
  type: 'suit_card';
  rank: Rank;
  suit: Suit;
};

type Card = SuitCard | {type: 'big_joker'} | {type: 'little_joker'};

const cardInfoToSuit = (cardInfo: any): Suit => {
  switch (cardInfo.typ) {
    case '♢':
      return 'diamonds';
    case '♧':
      return 'clubs';
    case '♡':
      return 'hearts';
    case '♤':
      return 'spades';
    default:
      throw new Error('Invalid cardInfo');
  }
};

export const unicodeToCard = (unicode: string): Card => {
  const cardInfo = (window as any).CARD_LUT[unicode];
  if (!cardInfo) {
    throw new Error(`Invalid card string: ${unicode}`);
  }

  if (unicode === '🃟') {
    return {type: 'little_joker'};
  } else if (unicode === '🃏') {
    return {type: 'big_joker'};
  } else {
    return {
      type: 'suit_card',
      suit: cardInfoToSuit(cardInfo),
      rank: cardInfo.number,
    };
  }
};

export const cardToUnicodeSuit = (
  card: SuitCard,
  fill: boolean = true,
): string => {
  const table = fill ? suitToFilledUnicode : suitToUnicode;
  return table[card.suit];
};
