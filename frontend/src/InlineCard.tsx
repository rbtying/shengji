import * as React from 'react';
import {unicodeToCard, cardToUnicodeSuit, SuitCard} from './util/cardHelpers';
import styled from 'styled-components';

const InlineCardBase = styled.span`
  padding-left: 0.1em;
  padding-right: 0.1em;
`;

const Suit = (className: string): React.FunctionComponent<{}> => (props) => (
  <InlineCardBase className={className} {...props} />
);
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
