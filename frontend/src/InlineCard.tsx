import * as React from 'react';
import {AppStateConsumer} from './AppStateProvider';
import {unicodeToCard, cardToUnicodeSuit, SuitCard} from './util/cardHelpers';
import ElementWithProps from './ElementWithProps';

const ColoredDiv = (color: string) =>
  ElementWithProps('span', {
    style: {color, paddingLeft: '0.1em', paddingRight: '0.1em'},
  });
const Black = ColoredDiv('#000000');
const Red = ColoredDiv('#BB0313');
const Blue = ColoredDiv('#1933F9');
const Green = ColoredDiv('#477E1B');

const suitColor = (suitCard: SuitCard, fourColor: boolean) => {
  switch (suitCard.suit) {
    case 'diamonds':
      return fourColor ? Blue : Red;
    case 'hearts':
      return Red;
    case 'clubs':
      return fourColor ? Green : Black;
    case 'spades':
      return Black;
  }
};

type Props = {
  card: string;
};

const InlineCard = (props: Props) => {
  const card = unicodeToCard(props.card);
  switch (card.type) {
    case 'big_joker':
      return <Red>HJ</Red>;
    case 'little_joker':
      return <Black>LJ</Black>;
    case 'suit_card':
      return (
        <AppStateConsumer>
          {({state}) => {
            const Color = suitColor(card, state.settings.fourColor);
            return (
              <Color>
                {card.rank}
                {cardToUnicodeSuit(card)}
              </Color>
            );
          }}
        </AppStateConsumer>
      );
  }
};

export default InlineCard;
