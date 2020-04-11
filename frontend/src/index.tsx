/* tslint:disable:max-classes-per-file variable-name forin */
import * as React from 'react';
import * as ReactDOM from 'react-dom';
import Beeper from './Beeper';
import Errors from './Errors';
import Trump from './Trump';
import FriendSelect from './FriendSelect';
import LabeledPlay from './LabeledPlay';
import Card from './Card';
import Trick from './Trick';
import Header from './Header';
import Friends from './Friends';
import Players from './Players';
import AppStateProvider, {AppState, AppStateConsumer} from './AppStateProvider';
import WebsocketProvider from './WebsocketProvider';
import Credits from './Credits';
import Chat from './Chat';
import mapObject from './util/mapObject';
import {
  ICardInfo,
  IDrawPhase,
  IExchangePhase,
  IFriend,
  IInitializePhase,
  IPlayPhase,
  IPlayer,
} from './types';
import * as ReactModal from 'react-modal';
ReactModal.setAppElement(document.getElementById('root'));

const CARD_LUT = mapObject(CARDS, (c: ICardInfo) => [c.value, c]);
(window as any).CARD_LUT = CARD_LUT;

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
    this.setHideLandlordsPoints = this.setHideLandlordsPoints.bind(this);
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
              num_friends: 0,
              friends: [],
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
    }
  }

  setHideLandlordsPoints(evt: any) {
    evt.preventDefault();
    send({Action: {SetHideLandlordsPoints: evt.target.value === 'hide'}});
  }

  startGame(evt: any) {
    evt.preventDefault();
    send({Action: 'StartGame'});
  }

  render() {
    const mode_as_string =
      this.props.state.game_mode === 'Tractor' ? 'Tractor' : 'FindingFriends';
    return (
      <div>
        <Header gameMode={this.props.state.game_mode} />
        <Players
          players={this.props.state.players}
          landlord={this.props.state.landlord}
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
        {this.props.state.players.length >= 4 ? (
          <button onClick={this.startGame}>Start game</button>
        ) : (
          <h2>Waiting for players...</h2>
        )}
        <Kicker players={this.props.state.players} />
        <div className="game-settings">
          <h3>Game settings</h3>
          <label>
            Game mode:{' '}
            <select value={mode_as_string} onChange={this.setGameMode}>
              <option value="Tractor">升级 / Tractor</option>
              <option value="FindingFriends">找朋友 / Finding Friends</option>
            </select>
          </label>
          <NumDecksSelector
            num_decks={this.props.state.num_decks}
            players={this.props.state.players}
          />
          <label>
            Point visibility
            <select
              value={this.props.state.hide_landlord_points ? 'hide' : 'show'}
              onChange={this.setHideLandlordsPoints}
            >
              <option value="show">Show all players' points</option>
              <option value="hide">Hide defending team's points</option>
            </select>
          </label>
          <LandlordSelector
            players={this.props.state.players}
            landlord={this.props.state.landlord}
          />
          <RankSelector
            players={this.props.state.players}
            name={this.props.name}
            num_decks={this.props.state.num_decks}
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
    this.props.state.players.forEach((p) => {
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
      this.props.state.players[this.props.state.position].name ===
      this.props.name;
    if (this.timeout) {
      clearTimeout(this.timeout);
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
      this.props.state.players[this.props.state.position].name ===
        this.props.name && this.props.state.deck.length > 0;
    if (
      can_draw &&
      !this.could_draw &&
      this.timeout === null &&
      this.state.autodraw
    ) {
      this.timeout = setTimeout(() => {
        this.drawCard();
      }, 250);
    }
    this.could_draw = can_draw;

    let next = this.props.state.players[this.props.state.position].id;
    if (
      this.props.state.deck.length === 0 &&
      this.props.state.bids.length > 0
    ) {
      next = this.props.state.bids[this.props.state.bids.length - 1].id;
    }

    const players: {[player_id: number]: IPlayer} = {};
    let player_id = -1;
    this.props.state.players.forEach((p) => {
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
        <Header gameMode={this.props.state.game_mode} />
        <Players
          players={this.props.state.players}
          landlord={this.props.state.landlord}
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
            (this.props.state.landlord !== null &&
              this.props.state.landlord !== player_id) ||
            (this.props.state.landlord === null &&
              this.props.state.bids[this.props.state.bids.length - 1].id !==
                player_id)
          }
        >
          Pick up cards from the bottom
        </button>
        <Cards
          cards={cards_not_bid}
          selected={this.state.selected}
          setSelected={this.setSelected}
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
    this.props.state.players.forEach((player, idx) => {
      if (player.id === this.props.state.landlord) {
        landlord_idx = idx;
      }
    });
    if (this.props.state.players[landlord_idx].name === this.props.name) {
      return (
        <div>
          <Header gameMode={this.props.state.game_mode} />
          <Players
            players={this.props.state.players}
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
          <Header gameMode={this.props.state.game_mode} />
          <Players
            players={this.props.state.players}
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

type IPlayProps = {
  state: IPlayPhase;
  name: string;
  cards: string[];
  beep_on_turn: boolean;
  show_last_trick: boolean;
};
interface IPlayState {
  selected: string[];
}
class Play extends React.Component<IPlayProps, IPlayState> {
  private was_my_turn: boolean = false;

  constructor(props: IPlayProps) {
    super(props);
    this.state = {
      selected: [],
    };
    this.setSelected = this.setSelected.bind(this);
    this.playCards = this.playCards.bind(this);
    this.takeBackCards = this.takeBackCards.bind(this);
    this.endTrick = this.endTrick.bind(this);
  }

  setSelected(new_selected: string[]) {
    this.setState({selected: new_selected});
  }

  playCards(evt: any) {
    evt.preventDefault();
    send({Action: {PlayCards: this.state.selected}});
    this.setSelected([]);
  }

  takeBackCards(evt: any) {
    evt.preventDefault();
    send({Action: 'TakeBackCards'});
  }

  endTrick(evt: any) {
    evt.preventDefault();
    send({Action: 'EndTrick'});
  }

  startNewGame(evt: any) {
    evt.preventDefault();
    send({Action: 'StartNewGame'});
  }

  render() {
    const next = this.props.state.trick.player_queue[0];
    let can_take_back = false;
    let can_play = false;
    let is_my_turn = false;
    this.props.state.players.forEach((p) => {
      if (p.name === this.props.name) {
        const last_play = this.props.state.trick.played_cards[
          this.props.state.trick.played_cards.length - 1
        ];
        if (p.id === next) {
          is_my_turn = true;
          if (last_play) {
            can_play = this.state.selected.length === last_play.cards.length;
          } else {
            can_play = this.state.selected.length > 0;
          }
        }
        if (last_play && p.id === last_play.id) {
          can_take_back = true;
        }
      }
    });
    const shouldBeBeeping =
      this.props.beep_on_turn && is_my_turn && !this.was_my_turn;
    this.was_my_turn = is_my_turn;

    let remaining_cards_to_play = 0;
    Object.values(this.props.state.hands.hands).forEach((h) => {
      Object.values(h).forEach((c) => {
        remaining_cards_to_play += c;
      });
    });

    return (
      <div>
        {shouldBeBeeping ? <Beeper /> : null}
        <Header gameMode={this.props.state.game_mode} />
        <Players
          players={this.props.state.players}
          landlord={this.props.state.landlord}
          landlords_team={this.props.state.landlords_team}
          name={this.props.name}
          next={next}
        />
        <Trump trump={this.props.state.trump} />
        <Friends gameMode={this.props.state.game_mode} />
        <Trick
          trick={this.props.state.trick}
          players={this.props.state.players}
        />
        <button onClick={this.playCards} disabled={!can_play}>
          Play selected cards
        </button>
        <button onClick={this.takeBackCards} disabled={!can_take_back}>
          Take back last play
        </button>
        <button
          onClick={this.endTrick}
          disabled={this.props.state.trick.player_queue.length > 0}
        >
          Finish trick
        </button>
        {remaining_cards_to_play === 0 &&
        this.props.state.trick.played_cards.length === 0 ? (
          <button onClick={this.startNewGame}>Finish game</button>
        ) : null}
        <Cards
          cards={this.props.cards}
          notify_empty={is_my_turn}
          selected={this.state.selected}
          setSelected={this.setSelected}
        />
        {this.props.state.last_trick && this.props.show_last_trick ? (
          <div>
            <p>Previous trick</p>
            <Trick
              trick={this.props.state.last_trick}
              players={this.props.state.players}
            />
          </div>
        ) : null}
        <Points
          points={this.props.state.points}
          num_decks={this.props.state.num_decks}
          players={this.props.state.players}
          landlords_team={this.props.state.landlords_team}
          landlord={this.props.state.landlord}
          hide_landlord_points={this.props.state.hide_landlord_points}
        />
        <LabeledPlay cards={this.props.state.kitty} label="底牌" />
      </div>
    );
  }
}

interface IPointsProps {
  players: IPlayer[];
  num_decks: number;
  points: {[player_id: number]: string[]};
  landlords_team: number[];
  landlord: number;
  hide_landlord_points: boolean | null;
}
class Points extends React.Component<IPointsProps, {}> {
  render() {
    let total_points_played = 0;
    let non_landlords_points = 0;
    let landlord = '';

    const player_point_elements = this.props.players.map((player) => {
      if (player.id === this.props.landlord) {
        landlord = player.name;
      }

      let player_points = 0;
      this.props.points[player.id].forEach((c) => {
        player_points += CARD_LUT[c].points;
      });
      total_points_played += player_points;

      const on_landlords_team = this.props.landlords_team.includes(player.id);
      const className = on_landlords_team ? 'landlord' : '';
      if (!on_landlords_team) {
        non_landlords_points += player_points;
      }
      const cards =
        this.props.points[player.id].length > 0
          ? this.props.points[player.id]
          : ['🂠'];

      if (this.props.hide_landlord_points && on_landlords_team) {
        return null;
      }

      return (
        <LabeledPlay
          key={player.id}
          className={className}
          label={`${player.name}: ${player_points}分`}
          cards={cards}
        />
      );
    });

    const segment = this.props.num_decks * 20;
    let threshold_str = '';

    if (non_landlords_points === 0) {
      threshold_str = `${landlord}'s team will go up 3 levels (next threshold: 5分)`;
    } else if (non_landlords_points < segment) {
      threshold_str = `${landlord}'s team will go up 2 levels (next threshold: ${segment}分)`;
    } else if (non_landlords_points < 2 * segment) {
      threshold_str = `${landlord}'s team will go up 1 level (next threshold: ${
        2 * segment
      }分)`;
    } else if (non_landlords_points < 3 * segment) {
      threshold_str = `Neither team will go up a level (next threshold: ${
        3 * segment
      }分)`;
    } else if (non_landlords_points < 4 * segment) {
      threshold_str = `The attacking team will go up 1 level (next threshold: ${
        4 * segment
      }分)`;
    } else if (non_landlords_points < 5 * segment) {
      threshold_str = `The attacking team will go up 2 levels (next threshold: ${
        5 * segment
      }分)`;
    } else {
      threshold_str = 'The attacking team will go up 3 levels.';
    }

    return (
      <div className="points">
        <h2>Points</h2>
        <p>
          {non_landlords_points}分
          {this.props.hide_landlord_points
            ? null
            : ` / ${total_points_played}分`}{' '}
          stolen from {landlord}'s team. {threshold_str}
        </p>
        {player_point_elements}
      </div>
    );
  }
}

interface ICardsProps {
  selected: string[];
  cards: string[];
  notify_empty?: boolean;
  setSelected(new_selected: string[]): void;
}
class Cards extends React.Component<ICardsProps, {}> {
  constructor(props: ICardsProps) {
    super(props);
    this.selectCard = this.selectCard.bind(this);
    this.unselectCard = this.unselectCard.bind(this);
  }

  selectCard(card: string) {
    const new_selected = [...this.props.selected];
    new_selected.push(card);
    this.props.setSelected(new_selected);
  }

  unselectCard(card: string) {
    const pos = this.props.selected.indexOf(card);
    if (pos >= 0) {
      const new_selected = [...this.props.selected];
      new_selected.splice(pos, 1);
      this.props.setSelected(new_selected);
    }
  }

  render() {
    const unselected = [...this.props.cards];
    this.props.selected.forEach((card) => {
      unselected.splice(unselected.indexOf(card), 1);
    });

    return (
      <div className="hand">
        <div className="selected-cards">
          {this.props.selected.map((c, idx) => (
            <Card key={idx} onClick={() => this.unselectCard(c)} card={c} />
          ))}
          {this.props.selected.length === 0 ? (
            <Card
              card="🂠"
              className={this.props.notify_empty ? 'notify' : ''}
            />
          ) : null}
        </div>
        <div className="unselected-cards">
          {unselected.map((c, idx) => (
            <Card key={idx} onClick={() => this.selectCard(c)} card={c} />
          ))}
          {unselected.length === 0 ? <Card card="🂠" /> : null}
        </div>
      </div>
    );
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

interface IKickerProps {
  players: IPlayer[];
}
class Kicker extends React.Component<IKickerProps, {to_kick: string}> {
  constructor(props: IKickerProps) {
    super(props);
    this.state = {
      to_kick: '',
    };
    this.onChange = this.onChange.bind(this);
    this.kick = this.kick.bind(this);
  }

  onChange(evt: any) {
    evt.preventDefault();
    this.setState({to_kick: evt.target.value});
  }
  kick(evt: any) {
    evt.preventDefault();
    send({Kick: parseInt(this.state.to_kick, 10)});
  }

  render() {
    return (
      <div className="kicker">
        <label>
          Kick player:{' '}
          <select value={this.state.to_kick} onChange={this.onChange}>
            <option value="" />
            {this.props.players.map((player) => (
              <option value={player.id} key={player.id}>
                {player.name}
              </option>
            ))}
          </select>
          <button onClick={this.kick} disabled={this.state.to_kick === ''}>
            Kick
          </button>
        </label>
      </div>
    );
  }
}

interface ILandlordSelectorProps {
  landlord: number | null;
  players: IPlayer[];
}
class LandlordSelector extends React.Component<ILandlordSelectorProps, {}> {
  constructor(props: ILandlordSelectorProps) {
    super(props);
    this.onChange = this.onChange.bind(this);
  }

  onChange(evt: any) {
    evt.preventDefault();

    if (evt.target.value !== '') {
      send({Action: {SetLandlord: parseInt(evt.target.value, 10)}});
    } else {
      send({Action: {SetLandlord: null}});
    }
  }

  render() {
    return (
      <div className="landlord-picker">
        <label>
          Current leader:{' '}
          <select
            value={this.props.landlord !== null ? this.props.landlord : ''}
            onChange={this.onChange}
          >
            <option value="">winner of the bid</option>
            {this.props.players.map((player) => (
              <option value={player.id} key={player.id}>
                {player.name}
              </option>
            ))}
          </select>
        </label>
      </div>
    );
  }
}

interface INumDecksSelectorProps {
  num_decks: number | null;
  players: IPlayer[];
}
class NumDecksSelector extends React.Component<INumDecksSelectorProps, {}> {
  constructor(props: INumDecksSelectorProps) {
    super(props);
    this.onChange = this.onChange.bind(this);
  }

  onChange(evt: any) {
    evt.preventDefault();

    if (evt.target.value !== '') {
      send({Action: {SetNumDecks: parseInt(evt.target.value, 10)}});
    } else {
      send({Action: {SetNumDecks: null}});
    }
  }

  render() {
    return (
      <div className="num-decks-picker">
        <label>
          Number of decks:{' '}
          <select
            value={this.props.num_decks !== null ? this.props.num_decks : ''}
            onChange={this.onChange}
          >
            <option value="">default</option>
            {Array(this.props.players.length)
              .fill(0)
              .map((_, idx) => {
                const val = idx + 1;
                return (
                  <option value={val} key={idx}>
                    {val}
                  </option>
                );
              })}
          </select>
        </label>
      </div>
    );
  }
}

interface IRankSelectorProps {
  num_decks: number | null;
  players: IPlayer[];
  name: string;
}
class RankSelector extends React.Component<IRankSelectorProps, {}> {
  constructor(props: IRankSelectorProps) {
    super(props);
    this.onChange = this.onChange.bind(this);
  }

  onChange(evt: any) {
    evt.preventDefault();

    if (evt.target.value !== '') {
      send({Action: {SetRank: evt.target.value}});
    }
  }

  render() {
    let selectedRank = '';
    this.props.players.forEach((p) => {
      if (p.name === this.props.name) {
        selectedRank = p.level;
      }
    });
    return (
      <div className="rank-picker">
        <label>
          Your rank:{' '}
          <select value={selectedRank} onChange={this.onChange}>
            {[
              '2',
              '3',
              '4',
              '5',
              '6',
              '7',
              '8',
              '9',
              '10',
              'J',
              'K',
              'Q',
              'A',
            ].map((rank) => (
              <option value={rank} key={rank}>
                {rank}
              </option>
            ))}
          </select>
        </label>
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
                cards={state.cards}
                name={state.name}
              />
            ) : null}
            {state.game_state.Draw ? (
              <Draw
                state={state.game_state.Draw}
                cards={state.cards}
                name={state.name}
              />
            ) : null}
            {state.game_state.Exchange ? (
              <Exchange
                state={state.game_state.Exchange}
                cards={state.cards}
                name={state.name}
              />
            ) : null}
            {state.game_state.Play ? (
              <Play
                state={state.game_state.Play}
                cards={state.cards}
                name={state.name}
                show_last_trick={state.settings.showLastTrick}
                beep_on_turn={state.settings.beepOnTurn}
              />
            ) : null}
            {state.game_state.Done ? <p>Game Over</p> : null}
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
        <AppStateConsumer>{renderUI}</AppStateConsumer>
      </WebsocketProvider>
    </AppStateProvider>,
    document.getElementById('root'),
  );
};

bootstrap();

declare var CARDS: ICardInfo[];
declare var send: (value: any) => void;
