import * as React from 'react';
import {unicodeToCard, cardToUnicodeSuit, SuitCard} from './util/cardHelpers';
import ElementWithProps from './ElementWithProps';

const Suit = (className: string) =>
  ElementWithProps('span', {
    className,
    style: {
      paddingLeft: '0.1em',
      paddingRight: '0.1em',
    },
  });
const Diamonds = Suit('♢');
const Hearts = Suit('♡');
const Spades = Suit('♤');
const Clubs = Suit('♧');
const LittleJoker = Suit('🃟');
const BigJoker = Suit('🃏');
const Unknown = Suit('🂠');

const suitComponent = (suitCard: SuitCard) => {
  switch (suitCard.suit) {
    case 'diamonds':
      return Diamonds;
    case 'hearts':
      return Hearts;
    case 'clubs':
      return Clubs;
    case 'spades':
      return Spades;
  }
};

type Props = {
  card: string;
};

const InlineCard = (props: Props) => {
  const card = unicodeToCard(props.card);
  switch (card.type) {
    case 'unknown':
      return <Unknown>🂠</Unknown>;
    case 'big_joker':
      return <BigJoker>HJ</BigJoker>;
    case 'little_joker':
      return <LittleJoker>LJ</LittleJoker>;
    case 'suit_card':
      const Component = suitComponent(card);
      return (
        <Component>
          {card.rank}
          {cardToUnicodeSuit(card)}
        </Component>
      );
  }
};

export default InlineCard;
