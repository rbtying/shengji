/* tslint:disable:max-classes-per-file variable-name forin */
import * as React from 'react';
import Cards from './Cards';
import {IDrawPhase, IPlayer} from './types';
import Header from './Header';
import Players from './Players';
import LabeledPlay from './LabeledPlay';
import BeepButton from './BeepButton';

type IDrawProps = {
  state: IDrawPhase;
  name: string;
  cards: string[];
  setTimeout: (fn: () => void, timeout: number) => number;
  clearTimeout: (id: number) => void;
};
interface IDrawState {
  selected: string[];
  autodraw: boolean;
}
class Draw extends React.Component<IDrawProps, IDrawState> {
  private could_draw: boolean = false;
  private timeout: number | null = null;

  constructor(props: IDrawProps) {
    super(props);
    this.state = {
      selected: [],
      autodraw: true,
    };
    this.setSelected = this.setSelected.bind(this);
    this.makeBid = this.makeBid.bind(this);
    this.takeBackBid = this.takeBackBid.bind(this);
    this.drawCard = this.drawCard.bind(this);
    this.pickUpKitty = this.pickUpKitty.bind(this);
    this.revealCard = this.revealCard.bind(this);
    this.onAutodrawClicked = this.onAutodrawClicked.bind(this);
  }

  setSelected(new_selected: string[]) {
    this.setState({selected: new_selected});
  }

  makeBid(evt: any) {
    evt.preventDefault();
    const counts: {[card: string]: number} = {};
    this.state.selected.forEach((c) => (counts[c] = (counts[c] || 0) + 1));
    if (Object.keys(counts).length !== 1) {
      return;
    }

    const players: {[player_id: number]: IPlayer} = {};
    this.props.state.propagated.players.forEach((p) => {
      players[p.id] = p;
    });

    for (const c in counts) {
      let already_bid = 0;
      this.props.state.bids.forEach((bid) => {
        if (players[bid.id].name === this.props.name && bid.card === c) {
          already_bid = already_bid < bid.count ? bid.count : already_bid;
        }
      });

      (window as any).send({Action: {Bid: [c, counts[c] + already_bid]}});
      this.setSelected([]);
    }
  }

  takeBackBid(evt: any) {
    evt.preventDefault();
    (window as any).send({Action: 'TakeBackBid'});
  }

  drawCard() {
    const can_draw =
      this.props.state.propagated.players[this.props.state.position].name ===
      this.props.name;
    if (this.timeout) {
      this.props.clearTimeout(this.timeout);
      this.timeout = null;
    }
    if (can_draw) {
      (window as any).send({Action: 'DrawCard'});
    }
  }

  pickUpKitty(evt: any) {
    evt.preventDefault();
    (window as any).send({Action: 'PickUpKitty'});
  }

  revealCard(evt: any) {
    evt.preventDefault();
    (window as any).send({Action: 'RevealCard'});
  }

  onAutodrawClicked(evt: any) {
    this.setState({
      autodraw: evt.target.checked,
    });
    if (evt.target.checked) {
      this.drawCard();
    } else {
      if (this.timeout) {
        clearTimeout(this.timeout);
        this.timeout = null;
      }
    }
  }

  render() {
    const can_draw =
      this.props.state.propagated.players[this.props.state.position].name ===
        this.props.name && this.props.state.deck.length > 0;
    if (
      can_draw &&
      !this.could_draw &&
      this.timeout === null &&
      this.state.autodraw
    ) {
      this.timeout = this.props.setTimeout(() => {
        this.drawCard();
      }, 250);
    }
    this.could_draw = can_draw;

    let next = this.props.state.propagated.players[this.props.state.position]
      .id;
    if (
      this.props.state.deck.length === 0 &&
      this.props.state.bids.length > 0
    ) {
      next = this.props.state.bids[this.props.state.bids.length - 1].id;
    }

    const players: {[player_id: number]: IPlayer} = {};
    let player_id = -1;
    this.props.state.propagated.players.forEach((p) => {
      players[p.id] = p;
      if (p.name === this.props.name) {
        player_id = p.id;
      }
    });

    const my_bids: {[card: string]: number} = {};
    this.props.state.bids.forEach((bid) => {
      if (player_id === bid.id) {
        const existing_bid = my_bids[bid.card] || 0;
        my_bids[bid.card] = existing_bid < bid.count ? bid.count : existing_bid;
      }
    });
    const cards_not_bid = [...this.props.cards];

    Object.keys(my_bids).forEach((card) => {
      const count = my_bids[card] || 0;
      for (let i = 0; i < count; i = i + 1) {
        const card_idx = cards_not_bid.indexOf(card);
        if (card_idx >= 0) {
          cards_not_bid.splice(card_idx, 1);
        }
      }
    });

    return (
      <div>
        <Header
          gameMode={this.props.state.game_mode}
          chatLink={this.props.state.propagated.chat_link}
        />
        <Players
          players={this.props.state.propagated.players}
          observers={this.props.state.propagated.observers}
          landlord={this.props.state.propagated.landlord}
          next={next}
          name={this.props.name}
        />
        <div>
          <h2>
            Bids ({this.props.state.deck.length} cards remaining in the deck)
          </h2>
          {this.props.state.autobid !== null ? (
            <LabeledPlay
              label={`${
                players[this.props.state.autobid.id].name
              } (from bottom)`}
              cards={[this.props.state.autobid.card]}
            />
          ) : null}
          {this.props.state.bids.map((bid, idx) => {
            const name = players[bid.id].name;
            return (
              <LabeledPlay
                label={name}
                key={idx}
                cards={Array(bid.count).fill(bid.card)}
              />
            );
          })}
        </div>
        <button
          onClick={(evt: any) => {
            evt.preventDefault();
            this.drawCard();
          }}
          disabled={!can_draw}
        >
          Draw card
        </button>
        <label>
          auto-draw
          <input
            type="checkbox"
            name="autodraw"
            checked={this.state.autodraw}
            onChange={this.onAutodrawClicked}
          />
        </label>
        <button
          onClick={this.makeBid}
          disabled={this.state.selected.length === 0}
        >
          Make bid
        </button>
        <button
          onClick={this.takeBackBid}
          disabled={
            this.props.state.bids.length === 0 ||
            this.props.state.bids[this.props.state.bids.length - 1].id !==
              player_id
          }
        >
          Take back bid
        </button>
        <button
          onClick={this.pickUpKitty}
          disabled={
            this.props.state.deck.length > 0 ||
            (this.props.state.bids.length === 0 &&
              this.props.state.autobid === null) ||
            (this.props.state.propagated.landlord !== null &&
              this.props.state.propagated.landlord !== player_id) ||
            (this.props.state.propagated.landlord === null &&
              this.props.state.bids[this.props.state.bids.length - 1].id !==
                player_id)
          }
        >
          Pick up cards from the bottom
        </button>
        <button
          onClick={this.revealCard}
          disabled={
            this.props.state.deck.length > 0 ||
            this.props.state.bids.length > 0 ||
            this.props.state.autobid !== null ||
            this.props.state.revealed_cards >= this.props.state.kitty.length
          }
        >
          Reveal card from the bottom
        </button>
        <BeepButton />
        {this.props.state.propagated.landlord !== null ? (
          <p>
            Bid using {players[this.props.state.propagated.landlord].level}'s in
            the same suit, or jokers
          </p>
        ) : players[player_id] ? (
          <p>
            Bid using {players[player_id].level}'s in the same suit, or jokers
          </p>
        ) : (
          <div />
        )}
        <Cards
          cardsInHand={cards_not_bid}
          selectedCards={this.state.selected}
          onSelect={this.setSelected}
        />
        <LabeledPlay cards={this.props.state.kitty} label="底牌" />
      </div>
    );
  }
}

export default Draw;
