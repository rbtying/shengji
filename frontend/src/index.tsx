/* tslint:disable:max-classes-per-file variable-name forin */
import * as React from 'react';
import * as ReactDOM from 'react-dom';
import Errors from './Errors';
import Trump from './Trump';
import FriendSelect from './FriendSelect';
import LabeledPlay from './LabeledPlay';
import Card from './Card';
import Header from './Header';
import Friends from './Friends';
import Players from './Players';
import AppStateProvider, {AppState, AppStateConsumer} from './AppStateProvider';
import WebsocketProvider from './WebsocketProvider';
import TimerProvider from './TimerProvider';
import {TimerConsumer} from './TimerProvider';
import LandlordSelector from './LandlordSelector';
import NumDecksSelector from './NumDecksSelector';
import RankSelector from './RankSelector';
import Kicker from './Kicker';
import Credits from './Credits';
import Chat from './Chat';
import Cards from './Cards';
import Play from './Play';
import ArrayUtils from './util/array';
import {
  IDrawPhase,
  IExchangePhase,
  IFriend,
  IInitializePhase,
  IPlayer,
} from './types';
import * as ReactModal from 'react-modal';
ReactModal.setAppElement(document.getElementById('root'));

type IInitializeProps = {
  state: IInitializePhase;
  cards: string[];
  name: string;
};
class Initialize extends React.Component<IInitializeProps, {}> {
  constructor(props: IInitializeProps) {
    super(props);
    this.setGameMode = this.setGameMode.bind(this);
    this.startGame = this.startGame.bind(this);
    this.setKittySize = this.setKittySize.bind(this);
    this.setNumFriends = this.setNumFriends.bind(this);
    this.setHideLandlordsPoints = this.setHideLandlordsPoints.bind(this);
    this.setHidePlayedCards = this.setHidePlayedCards.bind(this);
    this.setThrowPenalty = this.setThrowPenalty.bind(this);
    this.setKittyPenalty = this.setKittyPenalty.bind(this);
  }

  setGameMode(evt: any) {
    evt.preventDefault();
    if (evt.target.value === 'Tractor') {
      send({Action: {SetGameMode: 'Tractor'}});
    } else {
      send({
        Action: {
          SetGameMode: {
            FindingFriends: {
              num_friends: null,
            },
          },
        },
      });
    }
  }

  setNumFriends(evt: any) {
    evt.preventDefault();
    if (evt.target.value === '') {
      send({
        Action: {
          SetGameMode: {
            FindingFriends: {
              num_friends: null,
            },
          },
        },
      });
    } else {
      const num = parseInt(evt.target.value, 10);
      send({
        Action: {
          SetGameMode: {
            FindingFriends: {
              num_friends: num,
            },
          },
        },
      });
    }
  }

  setKittySize(evt: any) {
    evt.preventDefault();
    if (evt.target.value !== '') {
      const size = parseInt(evt.target.value, 10);
      send({
        Action: {
          SetKittySize: size,
        },
      });
    } else {
      send({
        Action: {
          SetKittySize: null,
        },
      });
    }
  }

  setKittyPenalty(evt: any) {
    evt.preventDefault();
    if (evt.target.value !== '') {
      send({
        Action: {
          SetKittyPenalty: evt.target.value,
        },
      });
    } else {
      send({
        Action: {
          SetKittyPenalty: null,
        },
      });
    }
  }

  setThrowPenalty(evt: any) {
    evt.preventDefault();
    if (evt.target.value !== '') {
      send({
        Action: {
          SetThrowPenalty: evt.target.value,
        },
      });
    } else {
      send({
        Action: {
          SetThrowPenalty: null,
        },
      });
    }
  }

  setHideLandlordsPoints(evt: any) {
    evt.preventDefault();
    send({Action: {SetHideLandlordsPoints: evt.target.value === 'hide'}});
  }

  setHidePlayedCards(evt: any) {
    evt.preventDefault();
    send({Action: {SetHidePlayedCards: evt.target.value === 'hide'}});
  }

  startGame(evt: any) {
    evt.preventDefault();
    send({Action: 'StartGame'});
  }

  render() {
    const mode_as_string =
      this.props.state.propagated.game_mode === 'Tractor'
        ? 'Tractor'
        : 'FindingFriends';
    const num_friends =
      this.props.state.propagated.game_mode === 'Tractor' ||
      this.props.state.propagated.game_mode.FindingFriends.num_friends === null
        ? ''
        : this.props.state.propagated.game_mode.FindingFriends.num_friends;
    const decks_effective =
      this.props.state.propagated.num_decks ||
      Math.floor(this.props.state.propagated.players.length / 2);
    let kitty_offset =
      (decks_effective * 54) % this.props.state.propagated.players.length;
    if (kitty_offset === 0) {
      kitty_offset += this.props.state.propagated.players.length;
    }

    let currentPlayer = this.props.state.propagated.players.find(
      (p: IPlayer) => p.name === this.props.name,
    );
    if (!currentPlayer) {
      currentPlayer = this.props.state.propagated.observers.find(
        (p) => p.name === this.props.name,
      );
    }

    return (
      <div>
        <Header
          gameMode={this.props.state.propagated.game_mode}
          chatLink={this.props.state.propagated.chat_link}
        />
        <Players
          players={this.props.state.propagated.players}
          landlord={this.props.state.propagated.landlord}
          next={null}
          movable={true}
          name={this.props.name}
        />
        <p>
          Send this link to other players to allow them to join the game:{' '}
          <a href={window.location.href} target="_blank">
            <code>{window.location.href}</code>
          </a>
        </p>
        {this.props.state.propagated.players.length >= 4 ? (
          <button onClick={this.startGame}>Start game</button>
        ) : (
          <h2>Waiting for players...</h2>
        )}
        <Kicker
          players={this.props.state.propagated.players}
          onKick={(playerId: number) => send({Kick: playerId})}
        />
        <div className="game-settings">
          <h3>Game settings</h3>
          <div>
            <label>
              Game mode:{' '}
              <select value={mode_as_string} onChange={this.setGameMode}>
                <option value="Tractor">升级 / Tractor</option>
                <option value="FindingFriends">找朋友 / Finding Friends</option>
              </select>
            </label>
          </div>
          <div>
            {this.props.state.propagated.game_mode !== 'Tractor' ? (
              <label>
                Number of friends:{' '}
                <select value={num_friends} onChange={this.setNumFriends}>
                  <option value="">default</option>
                  {ArrayUtils.range(
                    Math.max(
                      Math.floor(
                        this.props.state.propagated.players.length / 2,
                      ) - 1,
                      0,
                    ),
                    (idx) => (
                      <option value={idx + 1} key={idx}>
                        {idx + 1}
                      </option>
                    ),
                  )}
                </select>
              </label>
            ) : null}
          </div>
          <NumDecksSelector
            numPlayers={this.props.state.propagated.players.length}
            numDecks={this.props.state.propagated.num_decks}
            onChange={(newNumDecks: number | null) =>
              send({Action: {SetNumDecks: newNumDecks}})
            }
          />
          <div>
            <label>
              Number of cards in the bottom:{' '}
              <select
                value={this.props.state.propagated.kitty_size || ''}
                onChange={this.setKittySize}
              >
                <option value="">default</option>
                <option value={kitty_offset}>{kitty_offset} cards</option>
                <option
                  value={
                    kitty_offset + this.props.state.propagated.players.length
                  }
                >
                  {kitty_offset + this.props.state.propagated.players.length}{' '}
                  cards
                </option>
                <option
                  value={
                    kitty_offset +
                    2 * this.props.state.propagated.players.length
                  }
                >
                  {kitty_offset +
                    2 * this.props.state.propagated.players.length}{' '}
                  cards
                </option>
                <option
                  value={
                    kitty_offset +
                    3 * this.props.state.propagated.players.length
                  }
                >
                  {kitty_offset +
                    3 * this.props.state.propagated.players.length}{' '}
                  cards
                </option>
              </select>
            </label>
          </div>
          <div>
            <label>
              Point visibility:{' '}
              <select
                value={
                  this.props.state.propagated.hide_landlord_points
                    ? 'hide'
                    : 'show'
                }
                onChange={this.setHideLandlordsPoints}
              >
                <option value="show">Show all players' points</option>
                <option value="hide">Hide defending team's points</option>
              </select>
            </label>
          </div>
          <div>
            <label>
              Played card visibility (in chat):{' '}
              <select
                value={
                  this.props.state.propagated.hide_played_cards
                    ? 'hide'
                    : 'show'
                }
                onChange={this.setHidePlayedCards}
              >
                <option value="show">Show played cards in chat</option>
                <option value="hide">Hide played cards in chat</option>
              </select>
            </label>
          </div>
          <div>
            <label>
              Penalty for points left in the bottom:{' '}
              <select
                value={this.props.state.propagated.kitty_penalty}
                onChange={this.setKittyPenalty}
              >
                <option value="Times">Twice the size of the last trick</option>
                <option value="Power">
                  Two to the power of the size of the last trick
                </option>
              </select>
            </label>
          </div>
          <div>
            <label>
              Penalty for incorrect throws:{' '}
              <select
                value={this.props.state.propagated.throw_penalty}
                onChange={this.setThrowPenalty}
              >
                <option value="None">No penalty</option>
                <option value="TenPointsPerAttempt">
                  Ten points per bad throw
                </option>
              </select>
            </label>
          </div>
          <h3>Continuation settings</h3>
          <LandlordSelector
            players={this.props.state.propagated.players}
            landlordId={this.props.state.propagated.landlord}
            onChange={(newLandlord: number | null) =>
              send({Action: {SetLandlord: newLandlord}})
            }
          />
          <RankSelector
            rank={currentPlayer.level}
            onChangeRank={(newRank: string) =>
              send({Action: {SetRank: newRank}})
            }
          />
        </div>
      </div>
    );
  }
}

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
    this.drawCard = this.drawCard.bind(this);
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

      send({Action: {Bid: [c, counts[c] + already_bid]}});
      this.setSelected([]);
    }
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
      send({Action: 'DrawCard'});
    }
  }

  pickUpKitty(evt: any) {
    evt.preventDefault();
    send({Action: 'PickUpKitty'});
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
          landlord={this.props.state.propagated.landlord}
          next={next}
          name={this.props.name}
        />
        <div>
          <h2>
            Bids ({this.props.state.deck.length} cards remaining in the deck)
          </h2>
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
          onClick={this.pickUpKitty}
          disabled={
            this.props.state.deck.length > 0 ||
            this.props.state.bids.length === 0 ||
            (this.props.state.propagated.landlord !== null &&
              this.props.state.propagated.landlord !== player_id) ||
            (this.props.state.propagated.landlord === null &&
              this.props.state.bids[this.props.state.bids.length - 1].id !==
                player_id)
          }
        >
          Pick up cards from the bottom
        </button>
        <Cards
          cardsInHand={cards_not_bid}
          selectedCards={this.state.selected}
          onSelect={this.setSelected}
        />
      </div>
    );
  }
}

type IExchangeProps = {
  state: IExchangePhase;
  name: string;
  cards: string[];
};
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

  fixFriends() {
    if (this.props.state.game_mode !== 'Tractor') {
      const game_mode = this.props.state.game_mode.FindingFriends;
      const num_friends = game_mode.num_friends;
      const prop_friends = game_mode.friends;
      if (num_friends !== this.state.friends.length) {
        if (prop_friends.length !== num_friends) {
          const friends = [...this.state.friends];
          while (friends.length < num_friends) {
            friends.push({
              card: '',
              skip: 0,
              player_id: null,
            });
          }
          while (friends.length > num_friends) {
            friends.pop();
          }
          this.setState({friends});
        } else {
          this.setState({friends: prop_friends});
        }
      }
    } else {
      if (this.state.friends.length !== 0) {
        this.setState({friends: []});
      }
    }
  }

  componentDidMount() {
    this.fixFriends();
  }

  componentDidUpdate() {
    this.fixFriends();
  }

  moveCardToKitty(card: string) {
    send({Action: {MoveCardToKitty: card}});
  }

  moveCardToHand(card: string) {
    send({Action: {MoveCardToHand: card}});
  }

  startGame(evt: any) {
    evt.preventDefault();
    send({Action: 'BeginPlay'});
  }

  pickFriends(evt: any) {
    evt.preventDefault();
    if (
      this.props.state.game_mode !== 'Tractor' &&
      this.props.state.game_mode.FindingFriends.num_friends ===
        this.state.friends.length
    ) {
      send({
        Action: {
          SetFriends: this.state.friends,
        },
      });
    } else {
      this.fixFriends();
    }
  }

  render() {
    let landlord_idx = 0;
    this.props.state.propagated.players.forEach((player, idx) => {
      if (player.id === this.props.state.landlord) {
        landlord_idx = idx;
      }
    });
    if (
      this.props.state.propagated.players[landlord_idx].name === this.props.name
    ) {
      return (
        <div>
          <Header
            gameMode={this.props.state.game_mode}
            chatLink={this.props.state.propagated.chat_link}
          />
          <Players
            players={this.props.state.propagated.players}
            landlord={this.props.state.landlord}
            next={this.props.state.landlord}
            name={this.props.name}
          />
          <Trump trump={this.props.state.trump} />
          {this.props.state.game_mode !== 'Tractor' ? (
            <div>
              <Friends gameMode={this.props.state.game_mode} />
              {this.state.friends.map((friend, idx) => {
                const onChange = (x: IFriend) => {
                  const new_friends = [...this.state.friends];
                  new_friends[idx] = x;
                  this.setState({friends: new_friends});
                  this.fixFriends();
                };
                return (
                  <FriendSelect
                    onChange={onChange}
                    key={idx}
                    friend={friend}
                    trump={this.props.state.trump}
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
            Discarded cards {this.props.state.kitty.length} /{' '}
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

interface IJoinRoomProps {
  name: string;
  room_name: string;
  setName(name: string): void;
  setRoomName(name: string): void;
}
class JoinRoom extends React.Component<IJoinRoomProps, {editable: boolean}> {
  constructor(props: IJoinRoomProps) {
    super(props);
    this.state = {
      editable: false,
    };
    this.handleChange = this.handleChange.bind(this);
    this.handleSubmit = this.handleSubmit.bind(this);
    this.handleRoomChange = this.handleRoomChange.bind(this);
  }

  handleChange(event: any) {
    this.props.setName(event.target.value.trim());
  }

  handleRoomChange(event: any) {
    this.props.setRoomName(event.target.value.trim());
  }

  handleSubmit(event: any) {
    event.preventDefault();
    if (this.props.name.length > 0 && this.props.room_name.length === 16) {
      send({
        room_name: this.props.room_name,
        name: this.props.name,
      });
    }
  }

  render() {
    const editableRoomName = (
      <input
        type="text"
        placeholder="Enter a room code"
        value={this.props.room_name}
        onChange={this.handleRoomChange}
        maxLength={16}
      />
    );
    const nonEditableRoomName = (
      <span
        onClick={(evt) => {
          evt.preventDefault();
          this.setState({editable: true});
        }}
      >
        {this.props.room_name}
      </span>
    );

    return (
      <div>
        <LabeledPlay cards={['🃟', '🃟', '🃏', '🃏']} label={null}></LabeledPlay>
        <form className="join-room" onSubmit={this.handleSubmit}>
          <div>
            <h2>
              <label>
                <strong>Room Name:</strong>{' '}
                {this.state.editable ? editableRoomName : nonEditableRoomName} (
                <a href="rules" target="_blank">
                  rules
                </a>
                )
              </label>
            </h2>
          </div>
          <div>
            <label>
              <strong>Player Name:</strong>{' '}
              <input
                type="text"
                placeholder="Enter your name here"
                value={this.props.name}
                onChange={this.handleChange}
                autoFocus={true}
              />
            </label>
            <input
              type="submit"
              value="Join the game!"
              disabled={
                this.props.room_name.length !== 16 ||
                this.props.name.length === 0 ||
                this.props.name.length > 32
              }
            />
          </div>
          <div></div>
        </form>
      </div>
    );
  }
}

if (window.location.hash.length !== 17) {
  const arr = new Uint8Array(8);
  window.crypto.getRandomValues(arr);
  const r = Array.from(arr, (d) => ('0' + d.toString(16)).substr(-2)).join('');
  window.location.hash = r;
}

const renderUI = (props: {
  state: AppState;
  updateState: (state: Partial<AppState>) => void;
}) => {
  const {state, updateState} = props;
  if (state.connected) {
    if (state.game_state === null) {
      return (
        <div>
          <Errors errors={state.errors} />
          <div className="game">
            <h1>
              升级 / <span className="red">Tractor</span> / 找朋友 /{' '}
              <span className="red">Finding Friends</span>
            </h1>
            <JoinRoom
              name={state.name}
              room_name={state.roomName}
              setName={(name: string) => updateState({name})}
              setRoomName={(roomName: string) => {
                updateState({roomName});
                window.location.hash = roomName;
              }}
            />
          </div>
          <hr />
          <Credits />
        </div>
      );
    } else {
      const cards = [...state.cards];
      if (state.settings.reverseCardOrder) {
        cards.reverse();
      }
      return (
        <div className={state.settings.fourColor ? 'four-color' : ''}>
          <Errors errors={state.errors} />
          <div className="game">
            {state.game_state.Initialize ? null : (
              <a
                href={window.location.href}
                className="reset-link"
                onClick={(evt) => {
                  evt.preventDefault();
                  if (window.confirm('Do you really want to reset the game?')) {
                    send({Action: 'ResetGame'});
                  }
                }}
              >
                Reset game
              </a>
            )}
            {state.game_state.Initialize ? (
              <Initialize
                state={state.game_state.Initialize}
                cards={cards}
                name={state.name}
              />
            ) : null}
            {state.game_state.Draw ? (
              <TimerConsumer>
                {({setTimeout, clearTimeout}) => (
                  <Draw
                    state={state.game_state.Draw}
                    cards={cards}
                    name={state.name}
                    setTimeout={setTimeout}
                    clearTimeout={clearTimeout}
                  />
                )}
              </TimerConsumer>
            ) : null}
            {state.game_state.Exchange ? (
              <Exchange
                state={state.game_state.Exchange}
                cards={cards}
                name={state.name}
              />
            ) : null}
            {state.game_state.Play ? (
              <Play
                playPhase={state.game_state.Play}
                cards={cards}
                name={state.name}
                showLastTrick={state.settings.showLastTrick}
                unsetAutoPlayWhenWinnerChanges={
                  state.settings.unsetAutoPlayWhenWinnerChanges
                }
                beepOnTurn={state.settings.beepOnTurn}
              />
            ) : null}
          </div>
          <Chat messages={state.messages} />
          <hr />
          <Credits />
        </div>
      );
    }
  } else {
    return <p>disconnected from server, please refresh</p>;
  }
};

const bootstrap = () => {
  ReactDOM.render(
    <AppStateProvider>
      <WebsocketProvider>
        <TimerProvider>
          <AppStateConsumer>{renderUI}</AppStateConsumer>
        </TimerProvider>
      </WebsocketProvider>
    </AppStateProvider>,
    document.getElementById('root'),
  );
};

bootstrap();

declare var send: (value: any) => void;
