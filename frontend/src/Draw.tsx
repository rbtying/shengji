/* tslint:disable:max-classes-per-file variable-name forin */
import * as React from "react";
import { IDrawPhase, IPlayer } from "./types";
import Header from "./Header";
import Players from "./Players";
import LabeledPlay from "./LabeledPlay";
import BeepButton from "./BeepButton";
import BidArea from "./BidArea";

interface IDrawProps {
  state: IDrawPhase;
  name: string;
  setTimeout: (fn: () => void, timeout: number) => number;
  clearTimeout: (id: number) => void;
}
interface IDrawState {
  autodraw: boolean;
}
class Draw extends React.Component<IDrawProps, IDrawState> {
  private could_draw: boolean = false;
  private timeout: number | null = null;

  constructor(props: IDrawProps) {
    super(props);
    this.state = {
      autodraw: true,
    };
    this.drawCard = this.drawCard.bind(this);
    this.pickUpKitty = this.pickUpKitty.bind(this);
    this.revealCard = this.revealCard.bind(this);
    this.onAutodrawClicked = this.onAutodrawClicked.bind(this);
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

  pickUpKitty(evt: React.SyntheticEvent): void {
    evt.preventDefault();
    (window as any).send({ Action: "PickUpKitty" });
  }

  revealCard(evt: React.SyntheticEvent): void {
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

    const landlord = this.props.state.propagated.landlord;
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
        <BidArea
          bids={this.props.state.bids}
          autobid={this.props.state.autobid}
          hands={this.props.state.hands}
          epoch={0}
          name={this.props.name}
          landlord={this.props.state.propagated.landlord}
          players={this.props.state.propagated.players}
          bidPolicy={this.props.state.propagated.bid_policy}
          header={
            <h2>
              Bids ({this.props.state.deck.length} cards remaining in the deck)
            </h2>
          }
          prefixButtons={
            <>
              <button
                onClick={(evt: React.SyntheticEvent) => {
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
            </>
          }
          suffixButtons={
            <>
              <button
                onClick={this.pickUpKitty}
                disabled={
                  this.props.state.deck.length > 0 ||
                  (this.props.state.bids.length === 0 &&
                    this.props.state.autobid === null) ||
                  (landlord !== null && landlord !== playerId) ||
                  (landlord === null &&
                    ((this.props.state.propagated
                      .first_landlord_selection_policy === "ByWinningBid" &&
                      this.props.state.bids[this.props.state.bids.length - 1]
                        .id !== playerId) ||
                      (this.props.state.propagated
                        .first_landlord_selection_policy === "ByFirstBid" &&
                        this.props.state.bids[0].id !== playerId)))
                }
              >
                Pick up cards from the bottom
              </button>
              <button
                onClick={this.revealCard}
                disabled={
                  this.props.state.propagated.landlord == null ||
                  this.props.state.deck.length > 0 ||
                  this.props.state.bids.length > 0 ||
                  this.props.state.autobid !== null ||
                  this.props.state.revealed_cards >=
                    this.props.state.kitty.length
                }
              >
                Reveal card from the bottom
              </button>
              <BeepButton />
            </>
          }
          bidTakeBacksEnabled={
            this.props.state.propagated.bid_takeback_policy ===
            "AllowBidTakeback"
          }
        />
        <LabeledPlay
          className="kitty"
          cards={this.props.state.kitty}
          label="底牌"
        />
      </div>
    );
  }
}

export default Draw;
