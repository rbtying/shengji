/* tslint:disable:max-classes-per-file variable-name forin */
import * as React from "react";
import Trump from "./Trump";
import FriendSelect from "./FriendSelect";
import Card from "./Card";
import Header from "./Header";
import Friends from "./Friends";
import Players from "./Players";
import { IExchangePhase, IFriend } from "./types";

interface IExchangeProps {
  state: IExchangePhase;
  name: string;
  cards: string[];
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

  startGame(evt: any): void {
    evt.preventDefault();
    (window as any).send({ Action: "BeginPlay" });
  }

  pickFriends(evt: any): void {
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
    let landlordIdx = 0;
    this.props.state.propagated.players.forEach((player, idx) => {
      if (player.id === this.props.state.landlord) {
        landlordIdx = idx;
      }
    });
    if (
      this.props.state.propagated.players[landlordIdx].name === this.props.name
    ) {
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
          {this.props.state.game_mode !== "Tractor" ? (
            <div>
              <Friends
                gameMode={this.props.state.game_mode}
                showPlayed={false}
              />
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
          ) : null}
          <h2>Your hand</h2>
          <div className="hand">
            {this.props.cards.map((c, idx) => (
              <Card
                key={idx}
                onClick={() => this.moveCardToKitty(c)}
                card={c}
              />
            ))}
          </div>
          <h2>
            Discarded cards {this.props.state.kitty.length} /{" "}
            {this.props.state.kitty_size}
          </h2>
          <div className="kitty">
            {this.props.state.kitty.map((c, idx) => (
              <Card key={idx} onClick={() => this.moveCardToHand(c)} card={c} />
            ))}
          </div>
          <button
            onClick={this.startGame}
            disabled={
              this.props.state.kitty.length !== this.props.state.kitty_size
            }
          >
            Start game
          </button>
        </div>
      );
    } else {
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
          <div className="hand">
            {this.props.cards.map((c, idx) => (
              <Card key={idx} card={c} />
            ))}
          </div>
          <p>Waiting...</p>
        </div>
      );
    }
  }
}

export default Exchange;
