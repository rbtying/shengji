/* tslint:disable:max-classes-per-file variable-name forin */
import * as React from "react";
import Cards from "./Cards";
import { IDrawPhase, IPlayer } from "./types";
import Header from "./Header";
import Players from "./Players";
import LabeledPlay from "./LabeledPlay";
import BeepButton from "./BeepButton";

interface IDrawProps {
  state: IDrawPhase;
  name: string;
  cards: string[];
  separateBidCards: boolean;
  setTimeout: (fn: () => void, timeout: number) => number;
  clearTimeout: (id: number) => void;
}
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

  setSelected(newSelected: string[]): void {
    this.setState({ selected: newSelected });
  }

  makeBid(evt: any): void {
    evt.preventDefault();
    const counts: { [card: string]: number } = {};
    this.state.selected.forEach(
      (c) => (counts[c] = (counts[c] !== undefined ? counts[c] : 0) + 1)
    );
    if (Object.keys(counts).length !== 1) {
      return;
    }

    const players: { [playerId: number]: IPlayer } = {};
    this.props.state.propagated.players.forEach((p) => {
      players[p.id] = p;
    });

    for (const c in counts) {
      let alreadyBid = 0;
      this.props.state.bids.forEach((bid) => {
        if (players[bid.id].name === this.props.name && bid.card === c) {
          alreadyBid = alreadyBid < bid.count ? bid.count : alreadyBid;
        }
      });

      (window as any).send({ Action: { Bid: [c, counts[c] + alreadyBid] } });
      this.setSelected([]);
    }
  }

  takeBackBid(evt: any): void {
    evt.preventDefault();
    (window as any).send({ Action: "TakeBackBid" });
  }

  drawCard(): void {
    const canDraw =
      this.props.state.propagated.players[this.props.state.position].name ===
      this.props.name;
    if (this.timeout !== null) {
      this.props.clearTimeout(this.timeout);
      this.timeout = null;
    }
    if (canDraw) {
      (window as any).send({ Action: "DrawCard" });
    }
  }

  pickUpKitty(evt: any): void {
    evt.preventDefault();
    (window as any).send({ Action: "PickUpKitty" });
  }

  revealCard(evt: any): void {
    evt.preventDefault();
    (window as any).send({ Action: "RevealCard" });
  }

  onAutodrawClicked(evt: React.ChangeEvent<HTMLInputElement>): void {
    this.setState({
      autodraw: evt.target.checked,
    });
    if (evt.target.checked) {
      this.drawCard();
    } else {
      if (this.timeout !== null) {
        clearTimeout(this.timeout);
        this.timeout = null;
      }
    }
  }

  render(): JSX.Element {
    const canDraw =
      this.props.state.propagated.players[this.props.state.position].name ===
        this.props.name && this.props.state.deck.length > 0;
    if (
      canDraw &&
      !this.could_draw &&
      this.timeout === null &&
      this.state.autodraw
    ) {
      this.timeout = this.props.setTimeout(() => {
        this.drawCard();
      }, 250);
    }
    this.could_draw = canDraw;

    let next = this.props.state.propagated.players[this.props.state.position]
      .id;
    if (
      this.props.state.deck.length === 0 &&
      this.props.state.bids.length > 0
    ) {
      next = this.props.state.bids[this.props.state.bids.length - 1].id;
    }

    const players: { [playerId: number]: IPlayer } = {};
    let playerId = -1;
    this.props.state.propagated.players.forEach((p) => {
      players[p.id] = p;
      if (p.name === this.props.name) {
        playerId = p.id;
      }
    });

    const myBids: { [card: string]: number } = {};
    this.props.state.bids.forEach((bid) => {
      if (playerId === bid.id) {
        const existingBid = bid.card in myBids ? myBids[bid.card] : 0;
        myBids[bid.card] = existingBid < bid.count ? bid.count : existingBid;
      }
    });
    const cardsNotBid = [...this.props.cards];

    Object.keys(myBids).forEach((card) => {
      const count = card in myBids ? myBids[card] : 0;
      for (let i = 0; i < count; i = i + 1) {
        const cardIdx = cardsNotBid.indexOf(card);
        if (cardIdx >= 0) {
          cardsNotBid.splice(cardIdx, 1);
        }
      }
    });

    const landlord = this.props.state.propagated.landlord;
    const level =
      landlord == null ? players[playerId].level : players[landlord].level;
    return (
      <div>
        <Header
          gameMode={this.props.state.game_mode}
          chatLink={this.props.state.propagated.chat_link}
        />
        <Players
          players={this.props.state.propagated.players}
          observers={this.props.state.propagated.observers}
          landlord={landlord}
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
          disabled={!canDraw}
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
        {this.props.state.propagated.bid_takeback_policy ===
          "AllowBidTakeback" && (
          <button
            onClick={this.takeBackBid}
            disabled={
              this.props.state.bids.length === 0 ||
              this.props.state.bids[this.props.state.bids.length - 1].id !==
                playerId
            }
          >
            Take back bid
          </button>
        )}
        <button
          onClick={this.pickUpKitty}
          disabled={
            this.props.state.deck.length > 0 ||
            (this.props.state.bids.length === 0 &&
              this.props.state.autobid === null) ||
            (landlord !== null && landlord !== playerId) ||
            (landlord === null &&
              ((this.props.state.propagated.first_landlord_selection_policy ===
                "ByWinningBid" &&
                this.props.state.bids[this.props.state.bids.length - 1].id !==
                  playerId) ||
                (this.props.state.propagated.first_landlord_selection_policy ===
                  "ByFirstBid" &&
                  this.props.state.bids[0].id !== playerId)))
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
            Bid using {players[this.props.state.propagated.landlord].level}
            &apos;s in the same suit, or jokers
          </p>
        ) : players[playerId] !== undefined ? (
          <p>
            Bid using {players[playerId].level}&apos;s in the same suit, or
            jokers
          </p>
        ) : (
          <div />
        )}
        <Cards
          cardsInHand={cardsNotBid}
          selectedCards={this.state.selected}
          onSelect={this.setSelected}
          separateBidCards={this.props.separateBidCards}
          level={level}
        />
        <LabeledPlay cards={this.props.state.kitty} label="底牌" />
      </div>
    );
  }
}

export default Draw;
