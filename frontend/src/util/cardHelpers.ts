import preloadedCards from '../preloadedCards';
import ArrayUtils from '../util/array';
import {ICardInfo} from '../types';

export const cardLookup = ArrayUtils.mapObject(
  preloadedCards,
  (c: ICardInfo) => [c.value, c],
);

// prettier-ignore
type Rank = (
  | 'A' | '2' | '3' | '4' | '5' | '6' | '7'
  | '8' | '9' | '10' | 'J' | 'Q' | 'K'
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

type Card =
  | SuitCard
  | {type: 'big_joker'}
  | {type: 'little_joker'}
  | {type: 'unknown'};

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
  if (unicode === '🂠') {
    return {type: 'unknown'};
  }
  const cardInfo = cardLookup[unicode];
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
      rank: cardInfo.number as Rank,
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
