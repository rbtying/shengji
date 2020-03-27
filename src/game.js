'use strict';

const e = React.createElement;

const uri = 'ws://' + location.host + '/api';
const ws = new WebSocket(uri);

const red_cards = ['🃁', '🃍', '🃌', '🃋', '🃊', '🃉', '🃈', '🃇', '🃆', '🃅', '🃄', '🃃', '🃂', '🂱', '🂽', '🂼', '🂻', '🂺', '🂹', '🂸', '🂷', '🂶', '🂵', '🂴', '🂳', '🂲'];

class Initialize extends React.Component {
  render() {
    return e('div', null,
      e(Players, {players: this.props.state.players, landlord: this.props.state.landlord}),
      e('h1', null, 'Initialize'),
      this.props.state.players.length >= 4 ? e('form', {onSubmit: (evt) => { evt.preventDefault(); send({Action: 'StartGame'}); }}, 
        e('input', { type: 'submit', value: 'start game' }),
      ) : e('p', null, 'waiting for players...'),
    );
  }
}

class Draw extends React.Component {
  constructor(props) {
    super(props);
    this.state = {
      selected: [],
      timeout: null,
    };
    this.setSelected = ((new_selected) => this.setState({selected: new_selected})).bind(this);
    this.makeBid = this.makeBid.bind(this);
    this.drawCard = this.drawCard.bind(this);
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
    if (this.state.timeout) {
      clearTimeout(this.state.timeout);
    }
    this.setState({timeout: null});
    if (can_draw) {
      send({Action: 'DrawCard'});
    }
  }

  pickUpKitty(evt) {
    evt.preventDefault();
    send({Action: 'PickUpKitty'});
  }

  render() {
    const can_draw = this.props.state.players[this.props.state.position].name == this.props.name;
    if (can_draw && !this.state.timeout) {
      const t = setTimeout(() => {
        this.drawCard();
      }, 250);
    }
    return e('div', null,
      e(Players, { players: this.props.state.players, landlord: this.props.state.landlord }),
      e('h1', null, `Draw (level ${this.props.state.level})`),
      e('div', null, 
        e('h2', null, 'bids'),
        this.props.state.bids.map((bid, idx) => e('div', null,
          e('h3', null, `${this.props.state.players[bid.id].name}`),
          Array(bid.count).fill(bid.card).map((c, idx2) => e(Card, { card: c, key: `${idx}-${idx2}` })),
        ))
      ),
      e(Cards, { cards: this.props.cards, selected: this.state.selected, setSelected: this.setSelected }),
      e('button', { onClick: (evt) => { evt.preventDefault(); this.drawCard() }, disabled: !can_draw }, 'draw card'),
      e('button', { onClick: this.makeBid, disabled: this.state.selected.length == 0 }, 'make bid'),
      e('button', { onClick: this.pickUpKitty, disabled: this.props.state.deck.length > 0 }, 'pick up cards'),
    );
  }
}

class Exchange extends React.Component {
  moveCardToKitty(card) {
    send({ Action: { MoveCardToKitty: card } });
  }
  moveCardToHand(card) {
    send({ Action: { MoveCardToHand: card } });
  }

  render() {
    if (this.props.state.players[this.props.state.landlord].name == this.props.name) {
      return e('div', null,
        e(Players, {players: this.props.state.players, landlord: this.props.state.landlord}),
        e('h1', null, 'Exchange'),
        e('h2', null, 'your hand'),
        e('div', { className: 'hand' }, this.props.cards.map((c, idx) => 
          e(Card, { key: idx, onClick: () => this.moveCardToKitty(c), card: c })
        )),
        e('h2', null, 'discarded cards'),
        e('div', { className: 'kitty' }, this.props.state.kitty.map((c, idx) => 
          e(Card, { key: idx, onClick: () => this.moveCardToHand(c), card: c })
        )),
      );
    } else {
      return e('div', null,
        e(Players, { players: this.props.state.players, landlord: this.props.state.landlord }),
        e('h1', null, 'Exchange'),
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
  }

  render() {
    return e('div', null,
      e(Players, { players: this.props.state.players, landlord: this.props.state.landlord }),
      e('h1', null, 'Play'),
      e(Cards, { cards: this.props.cards, selected: this.state.selected, setSelected: this.setSelected }),
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

    return e('div', null,
      e('p', null, 'your selected cards'),
      e('div', {className: 'selected-cards'}, this.props.selected.map((c, idx) => 
        e(Card, { key: idx, onClick: () => this.unselectCard(c), card: c })
      )),
      e('p', null, 'your unselected cards'),
      e('div', { className: 'unselected-cards' }, unselected.map((c, idx) => 
        e(Card, { key: idx, onClick: () => this.selectCard(c), card: c })
      )),
    );
  }
}

class Card extends React.Component {
  render() {
    const card_color = (c) => {
      if (red_cards.indexOf(c) >= 0) {
        return ' red';
      } else {
        return '';
      }
    };
    const props = { className: 'card' + card_color(this.props.card) };
    if (this.props.onClick) {
      props.onClick = this.props.onClick;
    }
    return e('span', props, this.props.card)
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

class Players extends React.Component {
  render() {
    return e('div', { className: 'players' },
      this.props.players.map((player) => {
        if (player.id == this.props.landlord) {
          return e('div', { key: player.id, className: 'landlord' }, `${player.name}: ${player.level} (当庄)`)
        } else if (this.props.landlords_team && this.props.landlords_team.indexOf(player.id) >= 0) {
          return e('div', { key: player.id, className: 'landlord' }, `${player.name}: ${player.level}`)
        } else {
          return e('div', { key: player.id }, `${player.name}: ${player.level}`)
        }
      })
    );
  }
}

class Chat extends React.Component {
  constructor(props) {
    super(props);
    this.state = { message: '' };
    this.handleChange = this.handleChange.bind(this);
    this.handleSubmit = this.handleSubmit.bind(this);
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
      this.props.messages.map(
        (m, idx) => e('p', { key: idx, className: 'message' }, `${m.from}: ${m.message}`)
      ),
      e('form', {onSubmit: this.handleSubmit}, 
        e('input', { type: 'text', placeholder: 'type message here', value: this.state.message, onChange: this.handleChange }),
        e('input', { type: 'submit', value: 'submit' }),
      )
    );
  }
}

class Errors extends React.Component {
  render() {
    return e('div', null, this.props.errors.map(
      (err, idx) => e('p', {key: idx}, err)
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
  name: '',
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
            renderUI();
          }}),
          e('pre', null, JSON.stringify(state, null, 2)),
        ),
        document.getElementById('root')
      );
    } else {
      ReactDOM.render(
        e('div', null,
          e('h2', null, `Room Name: ${state.room_name}`),
          e(Errors, {errors: state.errors}),
          e(Chat, {messages: state.messages}),
          state.game_state.Initialize ? e(Initialize, {state: state.game_state.Initialize, cards: state.cards}) : null,
          state.game_state.Draw ? e(Draw, {state: state.game_state.Draw, cards: state.cards, name: state.name}) : null,
          state.game_state.Exchange ? e(Exchange, {state: state.game_state.Exchange, cards: state.cards, name: state.name}) : null,
          state.game_state.Play ? e(Play, {state: state.game_state.Play, cards: state.cards, name: state.name}) : null,
          state.game_state.Done ? e('p', null, 'Game over') : null,
          e('pre', null, JSON.stringify(state, null, 2)),
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
