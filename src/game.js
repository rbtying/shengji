'use strict';

const e = React.createElement;

const uri = 'ws://' + location.host + '/api';
const ws = new WebSocket(uri);

const RED_CARDS = [
  '🃁',
  '🃎',
  '🃍',
  '🃋',
  '🃊',
  '🃉',
  '🃈',
  '🃇',
  '🃆',
  '🃅',
  '🃄',
  '🃃',
  '🃂',
  '🂱',
  '🂾',
  '🂽',
  '🂻',
  '🂺',
  '🂹',
  '🂸',
  '🂷',
  '🂶',
  '🂵',
  '🂴',
  '🂳',
  '🂲',
  '🃏'
];
const ALL_STANDARD_CARDS = [
  { suit: '♧', number: 'A', value: '🃑' },
  { suit: '♧', number: 'K', value: '🃞' },
  { suit: '♧', number: 'Q', value: '🃝' },
  { suit: '♧', number: 'J', value: '🃛' },
  { suit: '♧', number: '10', value: '🃚' },
  { suit: '♧', number: '9', value: '🃙' },
  { suit: '♧', number: '8', value: '🃘' },
  { suit: '♧', number: '7', value: '🃗' },
  { suit: '♧', number: '6', value: '🃖' },
  { suit: '♧', number: '5', value: '🃕' },
  { suit: '♧', number: '4', value: '🃔' },
  { suit: '♧', number: '3', value: '🃓' },
  { suit: '♧', number: '2', value: '🃒' },
  { suit: '♢', number: 'A', value: '🃁' },
  { suit: '♢', number: 'K', value: '🃎' },
  { suit: '♢', number: 'Q', value: '🃍' },
  { suit: '♢', number: 'J', value: '🃋' },
  { suit: '♢', number: '10', value: '🃊' },
  { suit: '♢', number: '9', value: '🃉' },
  { suit: '♢', number: '8', value: '🃈' },
  { suit: '♢', number: '7', value: '🃇' },
  { suit: '♢', number: '6', value: '🃆' },
  { suit: '♢', number: '5', value: '🃅' },
  { suit: '♢', number: '4', value: '🃄' },
  { suit: '♢', number: '3', value: '🃃' },
  { suit: '♢', number: '2', value: '🃂' },
  { suit: '♡', number: 'A', value: '🂱' },
  { suit: '♡', number: 'K', value: '🂾' },
  { suit: '♡', number: 'Q', value: '🂽' },
  { suit: '♡', number: 'J', value: '🂻' },
  { suit: '♡', number: '10', value: '🂺' },
  { suit: '♡', number: '9', value: '🂹' },
  { suit: '♡', number: '8', value: '🂸' },
  { suit: '♡', number: '7', value: '🂷' },
  { suit: '♡', number: '6', value: '🂶' },
  { suit: '♡', number: '5', value: '🂵' },
  { suit: '♡', number: '4', value: '🂴' },
  { suit: '♡', number: '3', value: '🂳' },
  { suit: '♡', number: '2', value: '🂲' },
  { suit: '♤', number: 'A', value: '🂡' },
  { suit: '♤', number: 'K', value: '🂮' },
  { suit: '♤', number: 'Q', value: '🂭' },
  { suit: '♤', number: 'J', value: '🂫' },
  { suit: '♤', number: '10', value: '🂪' },
  { suit: '♤', number: '9', value: '🂩' },
  { suit: '♤', number: '8', value: '🂨' },
  { suit: '♤', number: '7', value: '🂧' },
  { suit: '♤', number: '6', value: '🂦' },
  { suit: '♤', number: '5', value: '🂥' },
  { suit: '♤', number: '4', value: '🂤' },
  { suit: '♤', number: '3', value: '🂣' },
  { suit: '♤', number: '2', value: '🂢' }
];

class Initialize extends React.Component {
  constructor(props) {
    super(props);
    this.setGameMode = this.setGameMode.bind(this);
    this.startGame = this.startGame.bind(this);
  }

  setGameMode(evt) {
    evt.preventDefault();
    if (evt.target.value == 'Tractor') {
      send({Action: { SetGameMode: 'Tractor' }});
    } else {
      send({Action: {
        SetGameMode: {
          FindingFriends: {
            num_friends: 0,
            friends: [],
          }
        }
      }});
    }
  }

  startGame(evt) {
    evt.preventDefault();
    send({Action: 'StartGame'});
  }

  render() {
    const mode_as_string = this.props.state.game_mode == 'Tractor' ? 'Tractor' : 'FindingFriends';
    return e('div', null,
      e(GameMode, { game_mode: this.props.state.game_mode }),
      e('h1', null, 'Initialize'),
      e(Players, { players: this.props.state.players, landlord: this.props.state.landlord, next: null, name: this.props.name }),
      e('select', { value: mode_as_string, onChange: this.setGameMode },
        e('option', { value: 'Tractor' }, '升级 / Tractor'),
        e('option', { value: 'FindingFriends' }, '找朋友 / Finding Friends'),
      ),
      this.props.state.players.length >= 4 ? 
        e('button', { onClick: this.startGame }, 'Start game') : e('p', null, 'Waiting for players...'),
    );
  }
}

class Draw extends React.Component {
  constructor(props) {
    super(props);
    this.state = {
      selected: [],
      autodraw: true,
    };
    this.could_draw = false;
    this.timeout = null;
    this.setSelected = ((new_selected) => this.setState({selected: new_selected})).bind(this);
    this.makeBid = this.makeBid.bind(this);
    this.drawCard = this.drawCard.bind(this);
    this.onAutodrawClicked = this.onAutodrawClicked.bind(this);
  }

  makeBid(evt) {
    evt.preventDefault();
    const counts = {};
    this.state.selected.forEach((c) => counts[c] = (counts[c] || 0) + 1);

    if (Object.keys(counts).length != 1) {
      return;
    }

    for (const c in counts) {
      send({ Action: { Bid: [c, counts[c]] } });
      this.setSelected([]);
    }
  }

  drawCard() {
    const can_draw = this.props.state.players[this.props.state.position].name == this.props.name;
    if (this.timeout) {
      clearTimeout(this.timeout);
      this.timeout = null;
    }
    if (can_draw) {
      send({Action: 'DrawCard'});
    }
  }

  pickUpKitty(evt) {
    evt.preventDefault();
    send({Action: 'PickUpKitty'});
  }

  onAutodrawClicked(evt) {
    this.setState({
      autodraw: evt.target.checked
    });
  }

  render() {
    const can_draw = this.props.state.players[this.props.state.position].name == this.props.name && this.props.state.deck.length > 0;
    if (can_draw && !this.could_draw && !this.timeout && this.state.autodraw) {
      this.timeout = setTimeout(() => {
        this.drawCard();
      }, 100);
    }
    this.could_draw = can_draw;

    let next = this.props.state.players[this.props.state.position].id;
    let next_idx = this.props.state.position;
    if (this.props.state.deck.length == 0 && this.props.state.bids.length > 0) {
      next = this.props.state.bids[this.props.state.bids.length - 1].id;

      this.props.state.players.forEach((player, idx) => {
        if (player.id == next) {
          next_idx = idx;
        }
      });
    }

    return e('div', null,
      e(GameMode, { game_mode: this.props.state.game_mode }),
      e('h1', null, `Draw (rank ${this.props.state.level})`),
      e(Players, {
        players: this.props.state.players,
        landlord: this.props.state.landlord,
        next: next,
        name: this.props.name,
      }),
      e('div', null, 
        e('h2', null, 'bids'),
        this.props.state.bids.map((bid, idx) => {
          let name = 'unknown';
          this.props.state.players.forEach((player) => {
            if (player.id == bid.id) {
              name = player.name;
            }
          });
          return e(LabeledPlay, { label: name, key: idx, cards: Array(bid.count).fill(bid.card) });
        }),
      ),
      e(Cards, { cards: this.props.cards, selected: this.state.selected, setSelected: this.setSelected }),
      e('button', { onClick: (evt) => { evt.preventDefault(); this.drawCard() }, disabled: !can_draw }, 'Draw card'),
      e('label', null, 'autodraw',
        e('input', { name: 'autodraw', type: 'checkbox', checked: this.state.autodraw, onChange: this.onAutodrawClicked })
      ),
      e('button', { onClick: this.makeBid, disabled: this.state.selected.length == 0 }, 'Make bid'),
      e('button', {
        onClick: this.pickUpKitty,
        disabled: this.props.state.deck.length > 0 || this.props.state.bids.length == 0 || this.props.state.players[next_idx].name != this.props.name
      }, 'Pick up cards'),
    );
  }
}

class Exchange extends React.Component {
  constructor(props) {
    super(props)
    this.moveCardToKitty = this.moveCardToKitty.bind(this);
    this.moveCardToHand = this.moveCardToHand.bind(this);
    this.startGame = this.startGame.bind(this);
    this.pickFriends = this.pickFriends.bind(this);
    this.state = {
      friends: []
    };

    this.fixFriends = (() => {
      if (this.props.state.game_mode.FindingFriends) {
        const num_friends = this.props.state.game_mode.FindingFriends.num_friends;
        const prop_friends = this.props.state.game_mode.FindingFriends.friends;
        if (num_friends != this.state.friends.length) {
          if (prop_friends.length != num_friends) {
            const friends = [...this.state.friends];
            while (friends.length < num_friends) {
              friends.push({
                card: '',
                skip: 0,
              });
            }
            while (friends.length > num_friends) {
              friends.pop();
            }
            this.setState({ friends: friends });
          } else {
            this.setState({ friends: prop_friends });
          }
        }
      } else {
        if (this.state.friends.length != 0) {
          this.setState({ friends: [] });
        }
      }
    }).bind(this);
  }

  componentDidMount() {
    this.fixFriends();
  }

  componentDidUpdate() {
    this.fixFriends();
  }

  moveCardToKitty(card) {
    send({ Action: { MoveCardToKitty: card } });
  }

  moveCardToHand(card) {
    send({ Action: { MoveCardToHand: card } });
  }

  startGame(evt) {
    evt.preventDefault();
    send({ Action: 'BeginPlay' });
  }

  pickFriends(evt) {
    evt.preventDefault();
    if (this.props.state.game_mode.FindingFriends && this.props.state.game_mode.FindingFriends.num_friends == this.state.friends.length) {
      send({ Action: {
        SetFriends: this.state.friends
      }});
    }
  }

  render() {
    let landlord_idx = 0;
    this.props.state.players.forEach((player, idx) => {
      if (player.id == this.props.state.landlord) {
        landlord_idx = idx;
      }
    });
    if (this.props.state.players[landlord_idx].name == this.props.name) {
      return e('div', null,
        e(GameMode, { game_mode: this.props.state.game_mode }),
        e('h1', null, 'Exchange'),
        e(Players, {players: this.props.state.players, landlord: this.props.state.landlord, next: this.props.state.landlord, name: this.props.name }),
        e(Trump, {trump: this.props.state.trump}),
        this.props.state.game_mode.FindingFriends ? e('div', null,
          e(Friends, { game_mode: this.props.state.game_mode }),
          this.state.friends.map((friend, idx) => {
            const onChange = (x) => {
              const new_friends = [...this.state.friends];
              new_friends[idx] = x;
              this.setState({ friends: new_friends });
            };
            return e(FriendSelect, {
              onChange: onChange,
              key: idx,
              friend: friend,
              trump: this.props.state.trump,
              num_decks: this.props.state.num_decks
            });
          }),
          e('button', { onClick: this.pickFriends }, 'Pick friends'),
        ) : null,
        e('h2', null, 'Your hand'),
        e('div', { className: 'hand' }, this.props.cards.map((c, idx) => 
          e(Card, { key: idx, onClick: () => this.moveCardToKitty(c), card: c })
        )),
        e('h2', null, `Discarded cards (${this.props.state.kitty.length} / ${this.props.state.kitty_size})`),
        e('div', { className: 'kitty' }, this.props.state.kitty.map((c, idx) => 
          e(Card, { key: idx, onClick: () => this.moveCardToHand(c), card: c })
        )),
        e('button', { onClick: this.startGame, disabled: this.props.state.kitty.length != this.props.state.kitty_size }, 'Start game'),
      );
    } else {
      return e('div', null,
        e(GameMode, { game_mode: this.props.state.game_mode }),
        e('h1', null, 'Exchange'),
        e(Players, { players: this.props.state.players, landlord: this.props.state.landlord, next: this.props.state.landlord, name: this.props.name }),
        e(Trump, {trump: this.props.state.trump}),
        e('p', null, 'Waiting...'),
      );
    }
  }
}

class Play extends React.Component {
  constructor(props) {
    super(props);
    this.state = {
      selected: [],
    };
    this.setSelected = ((new_selected) => this.setState({selected: new_selected})).bind(this);
    this.playCards = this.playCards.bind(this);
    this.takeBackCards = this.takeBackCards.bind(this);
    this.endTrick = this.endTrick.bind(this);
  }

  playCards(evt) {
    evt.preventDefault();
    send({ Action: { PlayCards: this.state.selected } });
    this.setSelected([]);
  }

  takeBackCards(evt) {
    evt.preventDefault();
    send({ Action: 'TakeBackCards' });
  }

  endTrick(evt) {
    evt.preventDefault();
    send({ Action: 'EndTrick' });
  }

  startNewGame(evt) {
    evt.preventDefault();
    send({ Action: 'StartNewGame' });
  }

  render() {
    const next = this.props.state.trick.player_queue[0];
    let can_take_back = false;
    let can_play = false;
    this.props.state.players.forEach((p) => {
      if (p.name == this.props.name) {
        const last_play = this.props.state.trick.played_cards[this.props.state.trick.played_cards.length - 1];
        if (p.id == next) {
          if (last_play) {
            can_play = this.state.selected.length == last_play.cards.length;
          } else {
            can_play = this.state.selected.length > 0;
          }
        }
        if (last_play && p.id == last_play.id) {
          can_take_back = true;
        }
      }
    });
    return e('div', null,
      e(GameMode, { game_mode: this.props.state.game_mode }),
      e('h1', null, 'Play tricks'),
      e(Players, {
        players: this.props.state.players,
        landlord: this.props.state.landlord,
        landlords_team: this.props.state.landlords_team,
        name: this.props.name,
        next: next
      }),
      e(Trump, { trump: this.props.state.trump }),
      e(Friends, { game_mode: this.props.state.game_mode }),
      e(Trick, { trick: this.props.state.trick, players: this.props.state.players }),
      e(Cards, { cards: this.props.cards, selected: this.state.selected, setSelected: this.setSelected }),
      e('button', { onClick: this.playCards, disabled: !can_play }, 'Play selected cards'),
      e('button', { onClick: this.takeBackCards, disabled: !can_take_back }, 'Take back last play'),
      e('button', { onClick: this.endTrick, disabled: this.props.state.trick.player_queue.length > 0 }, 'Finish trick'),
      this.props.cards.length == 0 ? e('button', { onClick: this.startNewGame }, 'Finish game') : null,
      e(Points, {
        points: this.props.state.points,
        players: this.props.state.players,
        landlords_team: this.props.state.landlords_team
      }),
      e(LabeledPlay, { cards: this.props.state.kitty, label: '底牌'}),
    );
  }
}

class Trick extends React.Component {
  render() {
    const names_by_id = {};
    this.props.players.forEach((p) => {
      names_by_id[p.id] = p.name;
    });
    const blank_cards = this.props.trick.played_cards.length > 0 ? Array(this.props.trick.played_cards[0].cards.length).fill('🂠') : ['🂠'];

    return e('div', { className: 'trick' },
      this.props.trick.played_cards.map((played, idx) => {
        return e(LabeledPlay, { key: idx, label: names_by_id[played.id], cards: played.cards });
      }),
      this.props.trick.player_queue.map((id, idx) => {
        return e(LabeledPlay, { key: idx + this.props.trick.played_cards.length, label: names_by_id[id], cards: blank_cards });
      }),
    );
  }
}

class Points extends React.Component {
  render() {
    console.log(this.props);
    return e('div', { className: 'points' },
      this.props.players.map((player) => {
        const className = this.props.landlords_team.includes(player.id) ? 'landlord' : '';
        const cards = this.props.points[player.id].length > 0 ? this.props.points[player.id] : ['🂠'];
        return e(LabeledPlay, { key: player.id, className: className, label: `${player.name} 分`, cards: cards });
      }),
    );
  }
}

class Cards extends React.Component {
  constructor(props) {
    super(props);
    this.selectCard = this.selectCard.bind(this);
    this.unselectCard = this.unselectCard.bind(this);
  }

  selectCard(card) {
    const new_selected = [...this.props.selected];
    new_selected.push(card);
    this.props.setSelected(new_selected);
  }

  unselectCard(card) {
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

    return e('div', { className: 'hand' },
      this.props.selected ? e('div', { className: 'selected-cards' }, this.props.selected.map((c, idx) => 
        e(Card, { key: idx, onClick: () => this.unselectCard(c), card: c })
      )) : null,
      e('p', null, 'Your hand'),
      e('div', { className: 'unselected-cards' }, unselected.map((c, idx) => 
        e(Card, { key: idx, onClick: () => this.selectCard(c), card: c })
      )),
    );
  }
}

class Card extends React.Component {
  render() {
    const card_color = (c) => {
      if (RED_CARDS.includes(c)) {
        return ' red';
      } else {
        return '';
      }
    };
    const props = { className: 'card' + card_color(this.props.card) };
    if (this.props.onClick) {
      props.onClick = this.props.onClick;
    }
    return e('span', props, this.props.card == '🃏' ? '🃟' : this.props.card)
  }
}

class LabeledPlay extends React.Component {
  render() {
    return e('div', { className: 'labeled-play' },
      e('div', { className: 'play' }, this.props.cards.map((card, idx) => e(Card, { card: card, key: idx }))),
      e('div', { className: 'label' }, this.props.label),
    );
  }
}

class JoinRoom extends React.Component {
  constructor(props) {
    super(props);
    this.handleChange = this.handleChange.bind(this);
    this.handleSubmit = this.handleSubmit.bind(this);
  }

  handleChange(event) {
    this.props.setName(event.target.value);
  }

  handleSubmit(event) {
    event.preventDefault();
    if (this.props.name.length > 0) {
      send({
        room_name: this.props.room_name,
        name: this.props.name,
      });
    }
  }

  render() {
    return e('div', null,
      e('form', {onSubmit: this.handleSubmit}, 
        e('input', { type: 'text', placeholder: 'name', value: this.props.name, onChange: this.handleChange, autoFocus: true }),
        e('input', { type: 'submit', value: 'join' }),
      )
    );
  }
}

class Trump extends React.Component {
  render() {
    if (this.props.trump.Standard) {
      return e('div', { className: 'trump' },
        `The trump suit is ${this.props.trump.Standard.suit}, rank ${this.props.trump.Standard.number}`
      );
    } else {
      return e('div', { className: 'trump' }, `No trump, rank ${this.props.trump.NoTrump.number}`);
    }
  }
}

class Players extends React.Component {
  render() {
    return e('table', { className: 'players' },
      e('tbody', null, 
        e('tr', null, 
          this.props.players.map((player) => {
            let className = 'player';
            let descriptor = `${player.name} (rank ${player.level})`;

            if (player.id == this.props.landlord) {
              descriptor = descriptor + ' (当庄)';
            }
            if (player.name == this.props.name) {
              descriptor = descriptor + ' (You!)';
            }
            if (player.id == this.props.landlord || (this.props.landlords_team && this.props.landlords_team.includes(player.id))) {
              className = className + ' landlord';
            }
            if (player.id == this.props.next) {
              className = className + ' next';
            }

            return e('td', { key: player.id, className: className }, descriptor);
          })
        )
      )
    );
  }
}

class Chat extends React.Component {
  constructor(props) {
    super(props);
    this.state = { message: '' };
    this.handleChange = this.handleChange.bind(this);
    this.handleSubmit = this.handleSubmit.bind(this);
    this.anchor = null;
  }

  componentDidMount() {
    if (this.anchor) {
      this.anchor.scrollIntoView({ block: 'nearest', inline: 'start', });
    }
  }

  componentDidUpdate() {
    if (this.anchor) {
      this.anchor.scrollIntoView({ block: 'nearest', inline: 'start', });
    }
  }

  handleChange(event) {
    this.setState({ message: event.target.value });
  }

  handleSubmit(event) {
    event.preventDefault();
    if (this.state.message.length > 0) {
      send({
        Message: this.state.message,
      });
    }
    this.setState({message: ''});
  }

  render() {
    return e('div', { className: 'chat' },
      e('div', { className: 'messages' },
        this.props.messages.map(
          (m, idx) => e('p', { key: idx, className: 'message' }, `${m.from}: ${m.message}`)
        ),
        e('div', { className: 'chat-anchor', ref: (el) => { this.anchor = el; } }),
      ),
      e('form', {onSubmit: this.handleSubmit}, 
        e('input', { type: 'text', placeholder: 'type message here', value: this.state.message, onChange: this.handleChange }),
        e('input', { type: 'submit', value: 'submit' }),
      )
    );
  }
}

class GameMode extends React.Component {
  render() {
    if (this.props.game_mode == 'Tractor') {
      return e('h1', null, '升级 / Tractor');
    } else {
      return e('h1', null, '找朋友 / Finding Friends');
    }
  }
}

class Friends extends React.Component {
  render() {
    if (this.props.game_mode.FindingFriends) {
      return e('div', { className: 'pending-friends' },
        this.props.game_mode.FindingFriends.friends.map((friend, idx) => {
          let card = friend.card;
          ALL_STANDARD_CARDS.forEach((c) => {
            if (c.value == friend.card) {
              card = `${c.number}${c.suit}`;
            }
          });
          if (friend.player_id != null) {
            return null;
          }
          if (friend.skip == 0) {
              return e('p', { key: idx }, `The next person to play ${card} is a friend`);
          } else {
              return e('p', { key: idx }, `${friend.skip} ${card} can be played before the next person to play ${card} is a friend`);
          }
        }),
      );
    } else {
      return null;
    }
  }
}

class FriendSelect extends React.Component {
  constructor(props) {
    super(props);
    this.onCardChange = this.onCardChange.bind(this);
    this.onOrdinalChange = this.onOrdinalChange.bind(this);
  }

  onCardChange(evt) {
    evt.preventDefault();
    this.props.onChange({
      card: evt.target.value,
      skip: this.props.friend.skip,
    });
  }
  onOrdinalChange(evt) {
    evt.preventDefault();
    this.props.onChange({
      card: this.props.friend.card,
      skip: parseInt(evt.target.value, 10),
    });
  }

  render() {
    const number = this.props.trump.Standard ? this.props.trump.Standard.number : this.props.trump.NoTrump.number;
    return e('div', { className: 'friend-select' },
      e('select', { value: this.props.friend.card, onChange: this.onCardChange },
        e('option', { value: '' }, ' '),
        ALL_STANDARD_CARDS.map((c) => {
          return c.number != number ? e('option', { key: c.value, value: c.value }, `${c.number}${c.suit}`) : null;
        })
      ),
      e('select', { value: this.props.friend.skip, onChange: this.onOrdinalChange },
        Array(this.props.num_decks).fill(1).map((_, idx) => {
          return e('option', { key: idx, value: idx }, idx + 1);
        })
      )
    );
  }
}

class Errors extends React.Component {
  render() {
    return e('div', { className: 'errors' }, this.props.errors.map(
      (err, idx) => e('p', {key: idx}, e('code', null, err))
    ));
  }
}

if (window.location.hash.length != 17) {
  var arr = new Uint8Array(8);
  window.crypto.getRandomValues(arr);
  const r = Array.from(arr, (d) => ('0' + d.toString(16)).substr(-2)).join('');
  window.location.hash = r;
}

let state = {
  connected: false,
  room_name: window.location.hash.slice(1),
  name: window.localStorage.getItem('name') || '',
  game_state: null,
  cards: [],
  errors: [],
  messages: [],
};

function send(value) {
  ws.send(JSON.stringify(value));
}

function renderUI() {
  if (state.connected) {
    if (state.game_state == null) {
      ReactDOM.render(
        e('div', null,
          e('h2', null, `Room Name: ${state.room_name}`),
          e(Errors, {errors: state.errors}),
          e(JoinRoom, {name: state.name, room_name: state.room_name, setName: (name) => {
            state.name = name;
            window.localStorage.setItem('name', name);
            renderUI();
          }}),
        ),
        document.getElementById('root')
      );
    } else {
      ReactDOM.render(
        e('div', null,
          e(Errors, {errors: state.errors}),
          e(Chat, {messages: state.messages}),
          e('div', {className: 'game'}, 
            state.game_state.Initialize ? e(Initialize, {state: state.game_state.Initialize, cards: state.cards}) : null,
            state.game_state.Draw ? e(Draw, {state: state.game_state.Draw, cards: state.cards, name: state.name}) : null,
            state.game_state.Exchange ? e(Exchange, {state: state.game_state.Exchange, cards: state.cards, name: state.name}) : null,
            state.game_state.Play ? e(Play, {state: state.game_state.Play, cards: state.cards, name: state.name}) : null,
            state.game_state.Done ? e('p', null, 'Game over') : null,
          ),
        ),
        document.getElementById('root')
      );
    }
  } else {
    ReactDOM.render(
      e('p', null, 'disconnected from server, please refresh'),
      document.getElementById('root')
    );
  }
}

ws.onopen = () => {
  state.connected = true;
  renderUI();
};
ws.onclose = (evt) => {
  state.connected = false;
  renderUI();
};
ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);
  if (msg.Message) {
    state.messages.push(msg.Message);
    if (state.messages.length >= 100) {
      state.message.shift();
    }
  }

  if (msg.Error) {
    state.errors.push(msg.Error);
    setTimeout(() => {
      state.errors = state.errors.filter((v) => v != msg.Error);
      renderUI();
    }, 5000);
  }

  if (msg.State) {
    state.game_state = msg.State.state;
    state.cards = msg.State.cards;
  }

  renderUI()
};
