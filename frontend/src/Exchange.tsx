/* tslint:disable:max-classes-per-file variable-name forin */
import * as React from "react";
import BeepButton from "./BeepButton";
import BidArea from "./BidArea";
import Trump from "./Trump";
import FriendSelect from "./FriendSelect";
import InlineCard from "./InlineCard";
import Card from "./Card";
import Header from "./Header";
import Friends from "./Friends";
import Players from "./Players";
import LabeledPlay from "./LabeledPlay";
import { IExchangePhase, IFriend } from "./types";
import Cards from "./Cards";

interface IExchangeProps {
  state: IExchangePhase;
  name: string;
}
interface IExchangeState {
  friends: IFriend[];
}
class Exchange extends React.Component<IExchangeProps, IExchangeState> {
  constructor(props: IExchangeProps) {
    super(props);
    this.moveCardToKitty = this.moveCardToKitty.bind(this);
    this.moveCardToHand = this.moveCardToHand.bind(this);
    this.startGame = this.startGame.bind(this);
    this.pickUpKitty = this.pickUpKitty.bind(this);
    this.putDownKitty = this.putDownKitty.bind(this);
    this.pickFriends = this.pickFriends.bind(this);
    this.state = {
      friends: [],
    };

    this.fixFriends = this.fixFriends.bind(this);
  }

  fixFriends(): void {
    if (this.props.state.game_mode !== "Tractor") {
      const gameMode = this.props.state.game_mode.FindingFriends;
      const numFriends = gameMode.num_friends;
      const propFriends = gameMode.friends;
      if (numFriends !== this.state.friends.length) {
        if (propFriends.length !== numFriends) {
          const friends = [...this.state.friends];
          while (friends.length < numFriends) {
            friends.push({
              card: "",
              skip: 0,
              initial_skip: 0,
              player_id: null,
            });
          }
          while (friends.length > numFriends) {
            friends.pop();
          }
          this.setState({ friends });
        } else {
          this.setState({ friends: propFriends });
        }
      }
    } else {
      if (this.state.friends.length !== 0) {
        this.setState({ friends: [] });
      }
    }
  }

  componentDidMount(): void {
    this.fixFriends();
  }

  componentDidUpdate(): void {
    this.fixFriends();
  }

  moveCardToKitty(card: string): void {
    (window as any).send({ Action: { MoveCardToKitty: card } });
  }

  moveCardToHand(card: string): void {
    (window as any).send({ Action: { MoveCardToHand: card } });
  }

  startGame(evt: React.SyntheticEvent): void {
    evt.preventDefault();
    (window as any).send({ Action: "BeginPlay" });
  }

  pickUpKitty(evt: React.SyntheticEvent): void {
    evt.preventDefault();
    (window as any).send({ Action: "PickUpKitty" });
  }

  putDownKitty(evt: React.SyntheticEvent): void {
    evt.preventDefault();
    (window as any).send({ Action: "PutDownKitty" });
  }

  pickFriends(evt: React.SyntheticEvent): void {
    evt.preventDefault();
    if (
      this.props.state.game_mode !== "Tractor" &&
      this.props.state.game_mode.FindingFriends.num_friends ===
        this.state.friends.length
    ) {
      (window as any).send({
        Action: {
          SetFriends: this.state.friends,
        },
      });
    } else {
      this.fixFriends();
    }
  }

  render(): JSX.Element {
    const exchanger =
      this.props.state.exchanger === null
        ? this.props.state.landlord
        : this.props.state.exchanger;

    let landlordIdx = -1;
    let exchangerIdx = -1;
    let playerId = -1;
    this.props.state.propagated.players.forEach((player, idx) => {
      if (player.id === this.props.state.landlord) {
        landlordIdx = idx;
      }
      if (player.id === exchanger) {
        exchangerIdx = idx;
      }
      if (player.name === this.props.name) {
        playerId = player.id;
      }
    });

    const isLandlord =
      this.props.state.propagated.players[landlordIdx].name === this.props.name;
    const isExchanger =
      this.props.state.propagated.players[exchangerIdx].name ===
      this.props.name;
    const kittyTheftEnabled =
      this.props.state.propagated.kitty_theft_policy === "AllowKittyTheft";

    const nextPlayer =
      kittyTheftEnabled &&
      !this.props.state.finalized &&
      this.props.state.exchanger !== null
        ? this.props.state.exchanger
        : this.props.state.landlord;

    const exchangeUI =
      isExchanger && !this.props.state.finalized ? (
        <>
          <h2>Your hand</h2>
          <Cards
            hands={this.props.state.hands}
            playerId={playerId}
            onCardClick={(c) => this.moveCardToKitty(c)}
            trump={this.props.state.trump}
          />
          <h2>
            Discarded cards {this.props.state.kitty.length} /{" "}
            {this.props.state.kitty_size}
          </h2>
          <div className="kitty">
            {this.props.state.kitty.map((c, idx) => (
              <Card key={idx} onClick={() => this.moveCardToHand(c)} card={c} />
            ))}
          </div>
          {kittyTheftEnabled ? (
            <button
              onClick={this.putDownKitty}
              disabled={
                this.props.state.kitty.length !== this.props.state.kitty_size
              }
            >
              Finalize exchanged cards
            </button>
          ) : null}
        </>
      ) : null;

    const lastBid = this.props.state.bids[this.props.state.bids.length - 1];
    const startGame = (
      <button
        onClick={this.startGame}
        disabled={
          this.props.state.kitty.length !== this.props.state.kitty_size ||
          (kittyTheftEnabled &&
            !this.props.state.finalized &&
            this.props.state.autobid === null)
        }
      >
        Start game
      </button>
    );
    const bidUI =
      kittyTheftEnabled &&
      this.props.state.finalized &&
      this.props.state.autobid === null &&
      (!isExchanger || lastBid.epoch + 1 !== this.props.state.epoch) ? (
        <>
          <BidArea
            bids={this.props.state.bids}
            autobid={this.props.state.autobid}
            hands={this.props.state.hands}
            epoch={this.props.state.epoch}
            name={this.props.name}
            landlord={this.props.state.propagated.landlord}
            players={this.props.state.propagated.players}
            bidPolicy={this.props.state.propagated.bid_policy}
            bidReinforcementPolicy={
              this.props.state.propagated.bid_reinforcement_policy
            }
            jokerBidPolicy={this.props.state.propagated.joker_bid_policy}
            numDecks={this.props.state.num_decks}
            header={
              <h2>Bids (round {this.props.state.epoch + 1} of bidding)</h2>
            }
            suffixButtons={
              <>
                <button
                  onClick={this.pickUpKitty}
                  disabled={
                    lastBid.id !== playerId ||
                    lastBid.epoch !== this.props.state.epoch
                  }
                >
                  Pick up cards from the bottom
                </button>
                {isLandlord ? startGame : null}
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
        </>
      ) : null;
    const friendUI =
      this.props.state.game_mode !== "Tractor" && isLandlord ? (
        <div>
          <Friends gameMode={this.props.state.game_mode} showPlayed={false} />
          {this.state.friends.map((friend, idx) => {
            const onChange = (x: IFriend): void => {
              const newFriends = [...this.state.friends];
              newFriends[idx] = x;
              this.setState({ friends: newFriends });
              this.fixFriends();
            };
            return (
              <FriendSelect
                onChange={onChange}
                key={idx}
                friend={friend}
                trump={this.props.state.trump}
                friend_selection_policy={
                  this.props.state.propagated.friend_selection_policy
                }
                num_decks={this.props.state.num_decks}
              />
            );
          })}
          <button onClick={this.pickFriends}>Pick friends</button>
        </div>
      ) : null;

    return (
      <div>
        <Header
          gameMode={this.props.state.game_mode}
          chatLink={this.props.state.propagated.chat_link}
        />
        <Players
          players={this.props.state.propagated.players}
          observers={this.props.state.propagated.observers}
          landlord={this.props.state.landlord}
          next={this.props.state.landlord}
          name={this.props.name}
        />
        <Trump trump={this.props.state.trump} />
        {this.props.state.removed_cards.length > 0 ? (
          <p>
            Note:{" "}
            {this.props.state.removed_cards.map((c) => (
              <InlineCard key={c} card={c} />
            ))}{" "}
            have been removed from the deck
          </p>
        ) : null}
        {friendUI}
        {exchangeUI}
        {exchangeUI === null && bidUI === null && playerId >= 0 ? (
          <>
            <Cards
              hands={this.props.state.hands}
              playerId={playerId}
              trump={this.props.state.trump}
            />
            <p>Waiting...</p>
          </>
        ) : null}
        {playerId !== nextPlayer && <BeepButton />}
        {isLandlord && bidUI === null ? startGame : null}
        {bidUI}
      </div>
    );
  }
}

export default Exchange;
