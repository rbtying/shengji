"use strict";

const e = React.createElement;
const CARD_LUT: { [details: string]: ICardInfo } = {};
CARDS.forEach((c) => {
  CARD_LUT[c.value] = c;
});

interface IInitializeProps {
  state: IInitializePhase;
  cards: string[];
  name: string;
}
class Initialize extends React.Component<IInitializeProps, {}> {
  constructor(props: IInitializeProps) {
    super(props);
    this.setGameMode = this.setGameMode.bind(this);
    this.startGame = this.startGame.bind(this);
    this.setKittySize = this.setKittySize.bind(this);
  }

  setGameMode(evt: any) {
    evt.preventDefault();
    if (evt.target.value == "Tractor") {
      send({ Action: { SetGameMode: "Tractor" } });
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
    if (evt.target.value != "") {
      const size = parseInt(evt.target.value, 10);
      send({
        Action: {
          SetKittySize: size,
        },
      });
    }
  }

  startGame(evt: any) {
    evt.preventDefault();
    send({ Action: "StartGame" });
  }

  render() {
    const mode_as_string =
      this.props.state.game_mode == "Tractor" ? "Tractor" : "FindingFriends";
    return (
      <div>
        <GameMode game_mode={this.props.state.game_mode} />
        <Players
          players={this.props.state.players}
          landlord={this.props.state.landlord}
          next={null}
          movable={true}
          name={this.props.name}
        />
        <p>
          Send the link to other players to allow them to join the game:{" "}
          <a href={window.location.href} target="_blank">
            <code>{window.location.href}</code>
          </a>
        </p>
        <select value={mode_as_string} onChange={this.setGameMode}>
          <option value="Tractor">升级 / Tractor</option>
          <option value="FindingFriends">找朋友 / Finding Friends</option>
        </select>
        <NumDecksSelector
          num_decks={this.props.state.num_decks}
          players={this.props.state.players}
        />
        {this.props.state.players.length >= 4 ? (
          <button onClick={this.startGame}>Start game</button>
        ) : (
          <p>Waiting for players...</p>
        )}
        <Kicker players={this.props.state.players} />
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
    );
  }
}

interface IDrawProps {
  state: IDrawPhase;
  name: string;
  cards: string[];
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
    this.drawCard = this.drawCard.bind(this);
    this.onAutodrawClicked = this.onAutodrawClicked.bind(this);
  }

  setSelected(new_selected: string[]) {
    this.setState({ selected: new_selected });
  }

  makeBid(evt: any) {
    evt.preventDefault();
    const counts: { [card: string]: number } = {};
    this.state.selected.forEach((c) => (counts[c] = (counts[c] || 0) + 1));
    if (Object.keys(counts).length != 1) {
      return;
    }

    const players: { [player_id: number]: IPlayer } = {};
    this.props.state.players.forEach((p) => {
      players[p.id] = p;
    });

    for (const c in counts) {
      let already_bid = 0;
      this.props.state.bids.forEach((bid) => {
        if (players[bid.id].name == this.props.name && bid.card == c) {
          already_bid = already_bid < bid.count ? bid.count : already_bid;
        }
      });

      send({ Action: { Bid: [c, counts[c] + already_bid] } });
      this.setSelected([]);
    }
  }

  drawCard() {
    const can_draw =
      this.props.state.players[this.props.state.position].name ==
      this.props.name;
    if (this.timeout) {
      clearTimeout(this.timeout);
      this.timeout = null;
    }
    if (can_draw) {
      send({ Action: "DrawCard" });
    }
  }

  pickUpKitty(evt: any) {
    evt.preventDefault();
    send({ Action: "PickUpKitty" });
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
      this.props.state.players[this.props.state.position].name ==
        this.props.name && this.props.state.deck.length > 0;
    if (
      can_draw &&
      !this.could_draw &&
      this.timeout == null &&
      this.state.autodraw
    ) {
      this.timeout = setTimeout(() => {
        this.drawCard();
      }, 250);
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

    const players: { [player_id: number]: IPlayer } = {};
    this.props.state.players.forEach((p) => {
      players[p.id] = p;
    });

    const my_bids: { [card: string]: number } = {};
    this.props.state.bids.forEach((bid) => {
      if (players[bid.id].name == this.props.name) {
        const existing_bid = my_bids[bid.card] || 0;
        my_bids[bid.card] = existing_bid < bid.count ? bid.count : existing_bid;
      }
    });
    let cards_not_bid = [...this.props.cards];

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
        <GameMode game_mode={this.props.state.game_mode} />
        <Players
          players={this.props.state.players}
          landlord={this.props.state.landlord}
          next={next}
          name={this.props.name}
        />
        <div>
          <h2>Bids</h2>
          {this.props.state.bids.map((bid, idx) => {
            let name = "unknown";
            this.props.state.players.forEach((player) => {
              if (player.id == bid.id) {
                name = player.name;
              }
            });
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
          <input
            type="checkbox"
            name="autodraw"
            checked={this.state.autodraw}
            onChange={this.onAutodrawClicked}
          />
        </label>
        <button
          onClick={this.makeBid}
          disabled={this.state.selected.length == 0}
        >
          Make bid
        </button>
        <button
          onClick={this.pickUpKitty}
          disabled={
            this.props.state.deck.length > 0 ||
            this.props.state.bids.length == 0
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

  fixFriends() {
    if (this.props.state.game_mode != "Tractor") {
      const game_mode = this.props.state.game_mode.FindingFriends;
      const num_friends = game_mode.num_friends;
      const prop_friends = game_mode.friends;
      if (num_friends != this.state.friends.length) {
        if (prop_friends.length != num_friends) {
          const friends = [...this.state.friends];
          while (friends.length < num_friends) {
            friends.push({
              card: "",
              skip: 0,
              player_id: null,
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
  }

  componentDidMount() {
    this.fixFriends();
  }

  componentDidUpdate() {
    this.fixFriends();
  }

  moveCardToKitty(card: string) {
    send({ Action: { MoveCardToKitty: card } });
  }

  moveCardToHand(card: string) {
    send({ Action: { MoveCardToHand: card } });
  }

  startGame(evt: any) {
    evt.preventDefault();
    send({ Action: "BeginPlay" });
  }

  pickFriends(evt: any) {
    evt.preventDefault();
    if (
      this.props.state.game_mode != "Tractor" &&
      this.props.state.game_mode.FindingFriends.num_friends ==
        this.state.friends.length
    ) {
      send({
        Action: {
          SetFriends: this.state.friends,
        },
      });
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
      return (
        <div>
          <GameMode game_mode={this.props.state.game_mode} />
          <Players
            players={this.props.state.players}
            landlord={this.props.state.landlord}
            next={this.props.state.landlord}
            name={this.props.name}
          />
          <Trump trump={this.props.state.trump} />
          {this.props.state.game_mode != "Tractor" ? (
            <div>
              <Friends game_mode={this.props.state.game_mode} />
              {this.state.friends.map((friend, idx) => {
                const onChange = (x: IFriend) => {
                  const new_friends = [...this.state.friends];
                  new_friends[idx] = x;
                  this.setState({ friends: new_friends });
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
            Discarded cards{" "}
            {this.props.state.kitty.length / this.props.state.kitty_size}
          </h2>
          <div className="kitty">
            {this.props.state.kitty.map((c, idx) => (
              <Card key={idx} onClick={() => this.moveCardToHand(c)} card={c} />
            ))}
          </div>
          <button
            onClick={this.startGame}
            disabled={
              this.props.state.kitty.length != this.props.state.kitty_size
            }
          >
            Start game
          </button>
        </div>
      );
    } else {
      return (
        <div>
          <GameMode game_mode={this.props.state.game_mode} />
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

interface IPlayProps {
  state: IPlayPhase;
  name: string;
  cards: string[];
  beep_on_turn: boolean;
  show_last_trick: boolean;
}
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
    this.setState({ selected: new_selected });
  }

  playCards(evt: any) {
    evt.preventDefault();
    send({ Action: { PlayCards: this.state.selected } });
    this.setSelected([]);
  }

  takeBackCards(evt: any) {
    evt.preventDefault();
    send({ Action: "TakeBackCards" });
  }

  endTrick(evt: any) {
    evt.preventDefault();
    send({ Action: "EndTrick" });
  }

  startNewGame(evt: any) {
    evt.preventDefault();
    send({ Action: "StartNewGame" });
  }

  render() {
    const next = this.props.state.trick.player_queue[0];
    let can_take_back = false;
    let can_play = false;
    let is_my_turn = false;
    this.props.state.players.forEach((p) => {
      if (p.name == this.props.name) {
        const last_play = this.props.state.trick.played_cards[
          this.props.state.trick.played_cards.length - 1
        ];
        if (p.id == next) {
          is_my_turn = true;
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
    if (this.props.beep_on_turn && is_my_turn && !this.was_my_turn) {
      beep(3, 440, 200);
    }
    this.was_my_turn = is_my_turn;

    return (
      <div>
        <GameMode game_mode={this.props.state.game_mode} />
        <Players
          players={this.props.state.players}
          landlord={this.props.state.landlord}
          landlords_team={this.props.state.landlords_team}
          name={this.props.name}
          next={next}
        />
        <Trump trump={this.props.state.trump} />
        <Friends game_mode={this.props.state.game_mode} />
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
        {this.props.cards.length == 0 ? (
          <button onClick={this.startNewGame}>Finish game</button>
        ) : null}
        ,
        <Cards
          cards={this.props.cards}
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
          players={this.props.state.players}
          landlords_team={this.props.state.landlords_team}
        />
        <LabeledPlay cards={this.props.state.kitty} label="底牌" />
      </div>
    );
  }
}

class Trick extends React.Component<{ players: IPlayer[]; trick: ITrick }, {}> {
  render() {
    const names_by_id: { [player_id: number]: string } = {};
    this.props.players.forEach((p) => {
      names_by_id[p.id] = p.name;
    });
    const blank_cards =
      this.props.trick.played_cards.length > 0
        ? Array(this.props.trick.played_cards[0].cards.length).fill("🂠")
        : ["🂠"];

    return (
      <div className="trick">
        {this.props.trick.played_cards.map((played, idx) => {
          const winning = this.props.trick.current_winner == played.id;
          return (
            <LabeledPlay
              key={idx}
              label={
                winning
                  ? `${names_by_id[played.id]} (!)`
                  : names_by_id[played.id]
              }
              className={winning ? "winning" : ""}
              cards={played.cards}
            />
          );
        })}
        {this.props.trick.player_queue.map((id, idx) => {
          return (
            <LabeledPlay
              key={idx + this.props.trick.played_cards.length}
              label={names_by_id[id]}
              cards={blank_cards}
            />
          );
        })}
      </div>
    );
  }
}

interface IPointsProps {
  players: IPlayer[];
  points: { [player_id: number]: string[] };
  landlords_team: number[];
}
class Points extends React.Component<IPointsProps, {}> {
  render() {
    return e(
      "div",
      { className: "points" },
      this.props.players.map((player) => {
        let total_points = 0;
        this.props.points[player.id].forEach((c) => {
          total_points += CARD_LUT[c].points;
        });
        const className = this.props.landlords_team.includes(player.id)
          ? "landlord"
          : "";
        const cards =
          this.props.points[player.id].length > 0
            ? this.props.points[player.id]
            : ["🂠"];
        return (
          <LabeledPlay
            key={player.id}
            className={className}
            label={`${player.name}: ${total_points}分`}
            cards={cards}
          />
        );
      })
    );
  }
}

interface ICardsProps {
  selected: string[];
  cards: string[];
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

    return e(
      "div",
      { className: "hand" },
      e(
        "div",
        { className: "selected-cards" },
        this.props.selected.map((c, idx) =>
          e(Card, { key: idx, onClick: () => this.unselectCard(c), card: c })
        ),
        this.props.selected.length == 0 ? e(Card, { card: "🂠" }) : null
      ),
      e("p", null, "Your hand"),
      e(
        "div",
        { className: "unselected-cards" },
        unselected.map((c, idx) =>
          e(Card, { key: idx, onClick: () => this.selectCard(c), card: c })
        ),
        unselected.length == 0 ? e(Card, { card: "🂠" }) : null
      )
    );
  }
}

interface ICardProps {
  card: string;
  onClick?(evt: any): any;
}
class Card extends React.Component<ICardProps, {}> {
  render() {
    const c = CARD_LUT[this.props.card];
    if (!c) {
      return e("span", { className: "card unknown" }, this.props.card);
    }

    const props: { onClick?(evt: any): any; className: string } = {
      className: `card ${c.typ}`,
    };
    if (this.props.onClick) {
      props.onClick = this.props.onClick;
    }
    return e("span", props, c.display_value);
  }
}

interface ILabeledPlayProps {
  className?: string;
  cards: string[];
  label: string;
}
class LabeledPlay extends React.Component<ILabeledPlayProps, {}> {
  render() {
    let className = "labeled-play";
    if (this.props.className) {
      className = className + " " + this.props.className;
    }
    return (
      <div className={className}>
        <div className="play">
          {this.props.cards.map((card, idx) => (
            <Card card={card} key={idx} />
          ))}
        </div>
        <div className="label">this.props.label</div>
      </div>
    );
  }
}

interface IJoinRoomProps {
  name: string;
  room_name: string;
  setName(name: string): void;
}
class JoinRoom extends React.Component<IJoinRoomProps, {}> {
  constructor(props: IJoinRoomProps) {
    super(props);
    this.handleChange = this.handleChange.bind(this);
    this.handleSubmit = this.handleSubmit.bind(this);
  }

  handleChange(event: any) {
    this.props.setName(event.target.value);
  }

  handleSubmit(event: any) {
    event.preventDefault();
    if (this.props.name.length > 0) {
      send({
        room_name: this.props.room_name,
        name: this.props.name,
      });
    }
  }

  render() {
    return (
      <div>
        <form onSubmit={this.handleSubmit}>
          <input
            type={"text"}
            placeholder={"name"}
            value={this.props.name}
            onChange={this.handleChange}
            autoFocus={true}
          />
          <input type="submit" value="join" />
        </form>
      </div>
    );
  }
}

class Trump extends React.Component<{ trump: ITrump }, {}> {
  render() {
    if (this.props.trump.Standard) {
      return (
        <div className="trump">
          The trump suit is{" "}
          <span className={this.props.trump.Standard.suit}>
            {this.props.trump.Standard.suit}
          </span>
          , rank {this.props.trump.Standard.number}
        </div>
      );
    } else if (this.props.trump.NoTrump) {
      return (
        <div className="trump">
          No trump, rank {this.props.trump.NoTrump.number}
        </div>
      );
    } else {
      return null;
    }
  }
}

interface IKickerProps {
  players: IPlayer[];
}
class Kicker extends React.Component<IKickerProps, { to_kick: string }> {
  constructor(props: IKickerProps) {
    super(props);
    this.state = {
      to_kick: "",
    };
    this.onChange = this.onChange.bind(this);
    this.kick = this.kick.bind(this);
  }

  onChange(evt: any) {
    evt.preventDefault();
    this.setState({ to_kick: evt.target.value });
  }
  kick(evt: any) {
    evt.preventDefault();
    send({ Kick: parseInt(this.state.to_kick, 10) });
  }

  render() {
    return e(
      "div",
      { className: "kicker" },
      e(
        "select",
        { value: this.state.to_kick, onChange: this.onChange },
        e("option", { value: "" }, ""),
        this.props.players.map((player) =>
          e("option", { value: player.id, key: player.id }, player.name)
        )
      ),
      e(
        "button",
        { onClick: this.kick, disabled: this.state.to_kick == "" },
        "kick"
      )
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

    if (evt.target.value != "") {
      send({ Action: { SetLandlord: parseInt(evt.target.value, 10) } });
    } else {
      send({ Action: { SetLandlord: null } });
    }
  }

  render() {
    return e(
      "div",
      { className: "landlord-picker" },
      e(
        "label",
        null,
        "leader: ",
        e(
          "select",
          {
            value: this.props.landlord != null ? this.props.landlord : "",
            onChange: this.onChange,
          },
          e("option", { value: "" }, "winner of the bid"),
          this.props.players.map((player) =>
            e("option", { value: player.id, key: player.id }, player.name)
          )
        )
      )
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

    if (evt.target.value != "") {
      send({ Action: { SetNumDecks: parseInt(evt.target.value, 10) } });
    } else {
      send({ Action: { SetNumDecks: null } });
    }
  }

  render() {
    return (
      <div className="num-decks-picker">
        <label>
          number of decks:{" "}
          <select
            value={this.props.num_decks != null ? this.props.num_decks : ""}
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

    if (evt.target.value != "") {
      send({ Action: { SetRank: evt.target.value } });
    }
  }

  render() {
    let rank = "";
    this.props.players.forEach((p) => {
      if (p.name == this.props.name) {
        rank = p.level;
      }
    });
    return (
      <div className="landlord-picker">
        <label>
          rank:{" "}
          <select value={rank} onChange={this.onChange}>
            {[
              "2",
              "3",
              "4",
              "5",
              "6",
              "7",
              "8",
              "9",
              "10",
              "J",
              "K",
              "Q",
              "A",
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

interface IPlayersProps {
  players: IPlayer[];
  landlord?: number | null;
  landlords_team?: number[];
  movable?: boolean;
  next?: number | null;
  name: string;
}
class Players extends React.Component<IPlayersProps, {}> {
  movePlayerLeft(evt: any, player_id: number) {
    evt.preventDefault();
    const player_ids = this.props.players.map((p) => p.id);
    const index = player_ids.indexOf(player_id);
    if (index > 0) {
      const p = player_ids[index];
      player_ids[index] = player_ids[index - 1];
      player_ids[index - 1] = p;
    } else {
      const p = player_ids[index];
      player_ids[index] = player_ids[player_ids.length - 1];
      player_ids[player_ids.length - 1] = p;
    }
    send({ Action: { ReorderPlayers: player_ids } });
  }

  movePlayerRight(evt: any, player_id: number) {
    evt.preventDefault();
    const player_ids = this.props.players.map((p) => p.id);
    const index = player_ids.indexOf(player_id);
    if (index < player_ids.length - 1) {
      const p = player_ids[index];
      player_ids[index] = player_ids[index + 1];
      player_ids[index + 1] = p;
    } else {
      const p = player_ids[index];
      player_ids[index] = player_ids[0];
      player_ids[0] = p;
    }
    send({ Action: { ReorderPlayers: player_ids } });
  }

  render() {
    return e(
      "table",
      { className: "players" },
      e(
        "tbody",
        null,
        e(
          "tr",
          null,
          this.props.players.map((player) => {
            let className = "player";
            let descriptor = `${player.name} (rank ${player.level})`;

            if (player.id == this.props.landlord) {
              descriptor = descriptor + " (当庄)";
            }
            if (player.name == this.props.name) {
              descriptor = descriptor + " (You!)";
            }
            if (
              player.id == this.props.landlord ||
              (this.props.landlords_team &&
                this.props.landlords_team.includes(player.id))
            ) {
              className = className + " landlord";
            }
            if (player.id == this.props.next) {
              className = className + " next";
            }

            return e(
              "td",
              { key: player.id, className: className },
              this.props.movable
                ? e(
                    "button",
                    {
                      onClick: (evt: any) =>
                        this.movePlayerLeft(evt, player.id),
                    },
                    "<"
                  )
                : null,
              descriptor,
              this.props.movable
                ? e(
                    "button",
                    {
                      onClick: (evt: any) =>
                        this.movePlayerRight(evt, player.id),
                    },
                    ">"
                  )
                : null
            );
          })
        )
      )
    );
  }
}

interface IChatProps {
  messages: { from: string; message: string; from_game?: boolean }[];
}
interface IChatState {
  message: string;
}
class Chat extends React.Component<IChatProps, IChatState> {
  private anchor = React.createRef<HTMLDivElement>();

  constructor(props: IChatProps) {
    super(props);
    this.state = { message: "" };
    this.handleChange = this.handleChange.bind(this);
    this.handleSubmit = this.handleSubmit.bind(this);
  }

  componentDidMount() {
    if (this.anchor.current) {
      this.anchor.current.scrollIntoView({ block: "nearest", inline: "start" });
    }
  }

  componentDidUpdate() {
    if (this.anchor.current) {
      this.anchor.current.scrollIntoView({ block: "nearest", inline: "start" });
    }
  }

  handleChange(event: any) {
    this.setState({ message: event.target.value });
  }

  handleSubmit(event: any) {
    event.preventDefault();
    if (this.state.message.length > 0) {
      send({
        Message: this.state.message,
      });
    }
    this.setState({ message: "" });
  }

  render() {
    return (
      <div className="chat">
        <div className="messages">
          {this.props.messages.map((m, idx) => {
            let className = "message";
            if (m.from_game) {
              className = className + " game-message";
            }
            return (
              <p key={idx} className={className}>
                {m.from}: {m.message}
              </p>
            );
          })}
          <div className="chat-anchor" ref={this.anchor} />
        </div>
        <form onSubmit={this.handleSubmit}>
          <input
            type="text"
            placeholder="type message here"
            value={this.state.message}
            onChange={this.handleChange}
          />
          <input type="submit" value="submit" />
        </form>
      </div>
    );
  }
}

class GameMode extends React.Component<{ game_mode: IGameMode }, {}> {
  render() {
    if (this.props.game_mode == "Tractor") {
      return <h1>升级 / Tractor</h1>;
    } else {
      return <h1>找朋友 / Finding Friends</h1>;
    }
  }
}

class Friends extends React.Component<{ game_mode: IGameMode }, {}> {
  render() {
    if (this.props.game_mode != "Tractor") {
      return e(
        "div",
        { className: "pending-friends" },
        this.props.game_mode.FindingFriends.friends.map((friend, idx) => {
          if (friend.player_id != null) {
            return null;
          }

          const c = CARD_LUT[friend.card];
          if (!c) {
            return null;
          }
          const card = `${c.number}${c.typ}`;
          if (friend.skip == 0) {
            return e(
              "p",
              { key: idx },
              "The next person to play ",
              e("span", { className: c.typ }, `${c.number}${c.typ}`),
              " is a friend"
            );
          } else {
            return e(
              "p",
              { key: idx },
              `${friend.skip} `,
              e("span", { className: c.typ }, `${c.number}${c.typ}`),
              " can be played before the next person to play ",
              e("span", { className: c.typ }, `${c.number}${c.typ}`),
              " is a friend"
            );
          }
        })
      );
    } else {
      return null;
    }
  }
}

interface IFriendSelectProps {
  friend: IFriend;
  trump: ITrump;
  num_decks: number;
  onChange(input: any): any;
}
class FriendSelect extends React.Component<IFriendSelectProps, {}> {
  constructor(props: IFriendSelectProps) {
    super(props);
    this.onCardChange = this.onCardChange.bind(this);
    this.onOrdinalChange = this.onOrdinalChange.bind(this);
  }

  onCardChange(evt: any) {
    evt.preventDefault();
    this.props.onChange({
      card: evt.target.value,
      skip: this.props.friend.skip,
    });
  }
  onOrdinalChange(evt: any) {
    evt.preventDefault();
    this.props.onChange({
      card: this.props.friend.card,
      skip: parseInt(evt.target.value, 10),
    });
  }

  render() {
    const number = this.props.trump.Standard
      ? this.props.trump.Standard.number
      : this.props.trump.NoTrump?.number;
    return e(
      "div",
      { className: "friend-select" },
      e(
        "select",
        { value: this.props.friend.card, onChange: this.onCardChange },
        e("option", { value: "" }, " "),
        CARDS.map((c) => {
          return c.number != null && c.number != number
            ? e(
                "option",
                { key: c.value, value: c.value },
                `${c.number}${c.typ}`
              )
            : null;
        })
      ),
      e(
        "select",
        { value: this.props.friend.skip, onChange: this.onOrdinalChange },
        Array(this.props.num_decks)
          .fill(1)
          .map((_, idx) => {
            return e("option", { key: idx, value: idx }, idx + 1);
          })
      )
    );
  }
}

class Errors extends React.Component<{ errors: string[] }, {}> {
  render() {
    return e(
      "div",
      { className: "errors" },
      this.props.errors.map((err, idx) =>
        e("p", { key: idx }, e("code", null, err))
      )
    );
  }
}

if (window.location.hash.length != 17) {
  var arr = new Uint8Array(8);
  window.crypto.getRandomValues(arr);
  const r = Array.from(arr, (d) => ("0" + d.toString(16)).substr(-2)).join("");
  window.location.hash = r;
}

const uri =
  (location.protocol == "https:" ? "wss://" : "ws://") +
  location.host +
  location.pathname +
  (location.pathname.endsWith("/") ? "api" : "/api");
const ws = new WebSocket(uri);

interface State {
  connected: boolean;
  room_name: string;
  name: string;
  game_state: IGameState | null;
  four_color: boolean;
  beep_on_turn: boolean;
  show_last_trick: boolean;
  cards: string[];
  errors: string[];
  messages: { from: string; message: string; from_game: boolean }[];
}

let state: State = {
  connected: false,
  room_name: window.location.hash.slice(1),
  name: window.localStorage.getItem("name") || "",
  game_state: null,
  four_color: window.localStorage.getItem("four_color") == "on" || false,
  beep_on_turn: window.localStorage.getItem("beep_on_turn") == "on" || false,
  show_last_trick:
    window.localStorage.getItem("show_last_trick") == "on" || false,
  cards: [],
  errors: [],
  messages: [],
};

function send(value: any) {
  ws.send(JSON.stringify(value));
}

function renderUI() {
  if (state.connected) {
    if (state.game_state == null) {
      ReactDOM.render(
        e(
          "div",
          null,
          e("h2", null, `Room Name: ${state.room_name}`),
          e(Errors, { errors: state.errors }),
          e(JoinRoom, {
            name: state.name,
            room_name: state.room_name,
            setName: (name: string) => {
              state.name = name;
              window.localStorage.setItem("name", name);
              renderUI();
            },
          })
        ),
        document.getElementById("root")
      );
    } else {
      ReactDOM.render(
        e(
          "div",
          { className: state.four_color ? "four-color" : "" },
          e(Errors, { errors: state.errors }),
          e(
            "div",
            { className: "game" },
            state.game_state.Initialize
              ? e(Initialize, {
                  state: state.game_state.Initialize,
                  cards: state.cards,
                  name: state.name,
                })
              : null,
            state.game_state.Draw
              ? e(Draw, {
                  state: state.game_state.Draw,
                  cards: state.cards,
                  name: state.name,
                })
              : null,
            state.game_state.Exchange
              ? e(Exchange, {
                  state: state.game_state.Exchange,
                  cards: state.cards,
                  name: state.name,
                })
              : null,
            state.game_state.Play
              ? e(Play, {
                  state: state.game_state.Play,
                  cards: state.cards,
                  name: state.name,
                  show_last_trick: state.show_last_trick,
                  beep_on_turn: state.beep_on_turn,
                })
              : null,
            state.game_state.Done ? e("p", null, "Game over") : null
          ),
          e(Chat, { messages: state.messages }),
          e("hr", null),
          e(
            "div",
            { className: "settings" },
            e(
              "label",
              null,
              "four-color mode",
              e("input", {
                name: "four-color",
                type: "checkbox",
                checked: state.four_color,
                onChange: (evt) => {
                  state.four_color = evt.target.checked;
                  if (state.four_color) {
                    window.localStorage.setItem("four_color", "on");
                  } else {
                    window.localStorage.setItem("four_color", "off");
                  }
                  renderUI();
                },
              })
            ),
            e(
              "label",
              null,
              "show last trick",
              e("input", {
                name: "show-last-trick",
                type: "checkbox",
                checked: state.show_last_trick,
                onChange: (evt) => {
                  state.show_last_trick = evt.target.checked;
                  if (state.show_last_trick) {
                    window.localStorage.setItem("show_last_trick", "on");
                  } else {
                    window.localStorage.setItem("show_last_trick", "off");
                  }
                  renderUI();
                },
              })
            ),
            e(
              "label",
              null,
              "beep on turn",
              e("input", {
                name: "show-last-trick",
                type: "checkbox",
                checked: state.beep_on_turn,
                onChange: (evt) => {
                  state.beep_on_turn = evt.target.checked;
                  if (state.beep_on_turn) {
                    window.localStorage.setItem("beep_on_turn", "on");
                  } else {
                    window.localStorage.setItem("beep_on_turn", "off");
                  }
                  renderUI();
                },
              })
            )
          )
        ),
        document.getElementById("root")
      );
    }
  } else {
    ReactDOM.render(
      <p>disconnected from server, please refresh</p>,
      document.getElementById("root")
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
      state.messages.shift();
    }
  }
  if (msg.Broadcast) {
    state.messages.push({
      from: "GAME",
      message: msg.Broadcast,
      from_game: true,
    });
    if (state.messages.length >= 100) {
      state.messages.shift();
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

  if (msg == "Kicked") {
    ws.close();
  }

  renderUI();
};

const beepContext = new AudioContext();

function beep(vol: number, freq: number, duration: number) {
  const v = beepContext.createOscillator();
  const u = beepContext.createGain();
  v.connect(u);
  v.frequency.value = freq;
  v.type = "square";
  u.connect(beepContext.destination);
  u.gain.value = vol * 0.01;
  v.start(beepContext.currentTime);
  v.stop(beepContext.currentTime + duration * 0.001);
}

declare var CARDS: ICardInfo[];

interface ICardInfo {
  value: string;
  display_value: string;
  typ: string;
  number: string | null;
  points: number;
}

interface IPlayer {
  id: number;
  name: string;
  level: string;
}

type IGameMode = "Tractor" | { FindingFriends: IFindingFriends };
interface IFindingFriends {
  num_friends: number;
  friends: [IFriend];
}

interface IFriend {
  card: string;
  skip: number;
  player_id: number | null;
}

interface IGameState {
  Initialize: IInitializePhase | null;
  Draw: IDrawPhase | null;
  Exchange: IExchangePhase | null;
  Play: IPlayPhase | null;
  Done: string | null;
}

interface IInitializePhase {
  max_player_id: number;
  players: IPlayer[];
  num_decks: number | null;
  kitty_size: number | null;
  game_mode: IGameMode;
  landlord: number | null;
}

interface IBid {
  id: number;
  card: string;
  count: number;
}

interface IDrawPhase {
  max_player_id: number;
  num_decks: number;
  game_mode: IGameMode;
  deck: string[];
  players: IPlayer[];
  observers: IPlayer[];
  hands: IHands;
  bids: IBid[];
  position: number;
  landlord: number | null;
  kitty: string[];
  level: number;
}

interface IExchangePhase {
  max_player_id: number;
  num_decks: number;
  game_mode: IGameMode;
  hands: IHands;
  kitty: string[];
  kitty_size: number;
  landlord: number;
  players: IPlayer[];
  observers: IPlayer[];
  trump: ITrump;
}

interface IPlayPhase {
  max_player_id: number;
  num_decks: number;
  game_mode: IGameMode;
  hands: IHands;
  points: { [id: number]: string[] };
  kitty: string[];
  landlord: number;
  landlords_team: number[];
  players: IPlayer[];
  observers: IPlayer[];
  trump: ITrump;
  trick: ITrick;
  last_trick: ITrick | null;
}

interface ITrickUnit {
  Tractor: { count: number; members: string[] } | null;
  Repeated: { count: number; card: string } | null;
}

interface ITrickFormat {
  suit: string;
  trump: ITrump;
  units: [ITrickUnit];
}

interface ITrick {
  player_queue: number[];
  played_cards: { id: number; cards: string[] }[];
  current_winner: number | null;
  trick_format: ITrickFormat | null;
  trump: ITrump;
}

interface IHands {
  hands: { [player_id: number]: { [card: string]: number } };
  level: number;
  trump: ITrump | null;
}

interface ITrump {
  Standard: { suit: string; number: string } | null;
  NoTrump: { number: string } | null;
}
