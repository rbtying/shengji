import preloadedCards from "../preloadedCards";
import { ICardInfo } from "../types";
import ArrayUtils from "../util/array";

export const cardLookup = ArrayUtils.mapObject(
  preloadedCards,
  (c: ICardInfo) => [c.value, c]
);

// prettier-ignore
type Rank = (
  | "A" | "2" | "3" | "4" | "5" | "6" | "7"
  | "8" | "9" | "10" | "J" | "Q" | "K"
);
type Suit = "diamonds" | "clubs" | "hearts" | "spades";

const suitToUnicode: { [key in Suit]: string } = {
  clubs: "♧",
  diamonds: "♢",
  hearts: "♡",
  spades: "♤",
};
const suitToFilledUnicode: { [key in Suit]: string } = {
  clubs: "♣",
  diamonds: "♦",
  hearts: "♥",
  spades: "♠",
};

export interface ISuitCard {
  type: "suit_card";
  rank: Rank;
  suit: Suit;
}

type Card =
  | ISuitCard
  | { type: "big_joker" }
  | { type: "little_joker" }
  | { type: "unknown" };

const cardInfoToSuit = (cardInfo: any): Suit => {
  switch (cardInfo.typ) {
    case "♢":
      return "diamonds";
    case "♧":
      return "clubs";
    case "♡":
      return "hearts";
    case "♤":
      return "spades";
    default:
      throw new Error("Invalid cardInfo");
  }
};

export const unicodeToCard = (unicode: string): Card => {
  if (unicode === "🂠") {
    return { type: "unknown" };
  }
  if (!(unicode in cardLookup)) {
    throw new Error(`Invalid card string: ${unicode}`);
  }
  const cardInfo = cardLookup[unicode];

  if (unicode === "🃟") {
    return { type: "little_joker" };
  } else if (unicode === "🃏") {
    return { type: "big_joker" };
  } else {
    return {
      rank: cardInfo.number as Rank,
      suit: cardInfoToSuit(cardInfo),
      type: "suit_card",
    };
  }
};

export const cardToUnicodeSuit = (
  card: ISuitCard,
  fill: boolean = true
): string => {
  const table = fill ? suitToFilledUnicode : suitToUnicode;
  return table[card.suit];
};
