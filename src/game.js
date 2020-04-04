"use strict";
var __extends = (this && this.__extends) || (function () {
    var extendStatics = function (d, b) {
        extendStatics = Object.setPrototypeOf ||
            ({ __proto__: [] } instanceof Array && function (d, b) { d.__proto__ = b; }) ||
            function (d, b) { for (var p in b) if (b.hasOwnProperty(p)) d[p] = b[p]; };
        return extendStatics(d, b);
    };
    return function (d, b) {
        extendStatics(d, b);
        function __() { this.constructor = d; }
        d.prototype = b === null ? Object.create(b) : (__.prototype = b.prototype, new __());
    };
})();
var __spreadArrays = (this && this.__spreadArrays) || function () {
    for (var s = 0, i = 0, il = arguments.length; i < il; i++) s += arguments[i].length;
    for (var r = Array(s), k = 0, i = 0; i < il; i++)
        for (var a = arguments[i], j = 0, jl = a.length; j < jl; j++, k++)
            r[k] = a[j];
    return r;
};
var e = React.createElement;
var CARD_LUT = {};
CARDS.forEach(function (c) {
    CARD_LUT[c.value] = c;
});
var Initialize = /** @class */ (function (_super) {
    __extends(Initialize, _super);
    function Initialize(props) {
        var _this = _super.call(this, props) || this;
        _this.setGameMode = _this.setGameMode.bind(_this);
        _this.startGame = _this.startGame.bind(_this);
        _this.setKittySize = _this.setKittySize.bind(_this);
        return _this;
    }
    Initialize.prototype.setGameMode = function (evt) {
        evt.preventDefault();
        if (evt.target.value == "Tractor") {
            send({ Action: { SetGameMode: "Tractor" } });
        }
        else {
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
    };
    Initialize.prototype.setKittySize = function (evt) {
        evt.preventDefault();
        if (evt.target.value != "") {
            var size = parseInt(evt.target.value, 10);
            send({
                Action: {
                    SetKittySize: size,
                },
            });
        }
    };
    Initialize.prototype.startGame = function (evt) {
        evt.preventDefault();
        send({ Action: "StartGame" });
    };
    Initialize.prototype.render = function () {
        var mode_as_string = this.props.state.game_mode == "Tractor" ? "Tractor" : "FindingFriends";
        return (React.createElement("div", null,
            React.createElement(GameMode, { game_mode: this.props.state.game_mode }),
            React.createElement(Players, { players: this.props.state.players, landlord: this.props.state.landlord, next: null, movable: true, name: this.props.name }),
            React.createElement("p", null,
                "Send the link to other players to allow them to join the game:",
                " ",
                React.createElement("a", { href: window.location.href, target: "_blank" },
                    React.createElement("code", null, window.location.href))),
            React.createElement("select", { value: mode_as_string, onChange: this.setGameMode },
                React.createElement("option", { value: "Tractor" }, "\u5347\u7EA7 / Tractor"),
                React.createElement("option", { value: "FindingFriends" }, "\u627E\u670B\u53CB / Finding Friends")),
            React.createElement(NumDecksSelector, { num_decks: this.props.state.num_decks, players: this.props.state.players }),
            this.props.state.players.length >= 4 ? (React.createElement("button", { onClick: this.startGame }, "Start game")) : (React.createElement("p", null, "Waiting for players...")),
            React.createElement(Kicker, { players: this.props.state.players }),
            React.createElement(LandlordSelector, { players: this.props.state.players, landlord: this.props.state.landlord }),
            React.createElement(RankSelector, { players: this.props.state.players, name: this.props.name, num_decks: this.props.state.num_decks })));
    };
    return Initialize;
}(React.Component));
var Draw = /** @class */ (function (_super) {
    __extends(Draw, _super);
    function Draw(props) {
        var _this = _super.call(this, props) || this;
        _this.could_draw = false;
        _this.timeout = null;
        _this.state = {
            selected: [],
            autodraw: true,
        };
        _this.setSelected = _this.setSelected.bind(_this);
        _this.makeBid = _this.makeBid.bind(_this);
        _this.drawCard = _this.drawCard.bind(_this);
        _this.onAutodrawClicked = _this.onAutodrawClicked.bind(_this);
        return _this;
    }
    Draw.prototype.setSelected = function (new_selected) {
        this.setState({ selected: new_selected });
    };
    Draw.prototype.makeBid = function (evt) {
        var _this = this;
        evt.preventDefault();
        var counts = {};
        this.state.selected.forEach(function (c) { return (counts[c] = (counts[c] || 0) + 1); });
        if (Object.keys(counts).length != 1) {
            return;
        }
        var players = {};
        this.props.state.players.forEach(function (p) {
            players[p.id] = p;
        });
        var _loop_1 = function (c) {
            var already_bid = 0;
            this_1.props.state.bids.forEach(function (bid) {
                if (players[bid.id].name == _this.props.name && bid.card == c) {
                    already_bid = already_bid < bid.count ? bid.count : already_bid;
                }
            });
            send({ Action: { Bid: [c, counts[c] + already_bid] } });
            this_1.setSelected([]);
        };
        var this_1 = this;
        for (var c in counts) {
            _loop_1(c);
        }
    };
    Draw.prototype.drawCard = function () {
        var can_draw = this.props.state.players[this.props.state.position].name ==
            this.props.name;
        if (this.timeout) {
            clearTimeout(this.timeout);
            this.timeout = null;
        }
        if (can_draw) {
            send({ Action: "DrawCard" });
        }
    };
    Draw.prototype.pickUpKitty = function (evt) {
        evt.preventDefault();
        send({ Action: "PickUpKitty" });
    };
    Draw.prototype.onAutodrawClicked = function (evt) {
        this.setState({
            autodraw: evt.target.checked,
        });
        if (evt.target.checked) {
            this.drawCard();
        }
        else {
            if (this.timeout) {
                clearTimeout(this.timeout);
                this.timeout = null;
            }
        }
    };
    Draw.prototype.render = function () {
        var _this = this;
        var can_draw = this.props.state.players[this.props.state.position].name ==
            this.props.name && this.props.state.deck.length > 0;
        if (can_draw &&
            !this.could_draw &&
            this.timeout == null &&
            this.state.autodraw) {
            this.timeout = setTimeout(function () {
                _this.drawCard();
            }, 250);
        }
        this.could_draw = can_draw;
        var next = this.props.state.players[this.props.state.position].id;
        var next_idx = this.props.state.position;
        if (this.props.state.deck.length == 0 && this.props.state.bids.length > 0) {
            next = this.props.state.bids[this.props.state.bids.length - 1].id;
            this.props.state.players.forEach(function (player, idx) {
                if (player.id == next) {
                    next_idx = idx;
                }
            });
        }
        var players = {};
        this.props.state.players.forEach(function (p) {
            players[p.id] = p;
        });
        var my_bids = {};
        this.props.state.bids.forEach(function (bid) {
            if (players[bid.id].name == _this.props.name) {
                var existing_bid = my_bids[bid.card] || 0;
                my_bids[bid.card] = existing_bid < bid.count ? bid.count : existing_bid;
            }
        });
        var cards_not_bid = __spreadArrays(this.props.cards);
        Object.keys(my_bids).forEach(function (card) {
            var count = my_bids[card] || 0;
            for (var i = 0; i < count; i = i + 1) {
                var card_idx = cards_not_bid.indexOf(card);
                if (card_idx >= 0) {
                    cards_not_bid.splice(card_idx, 1);
                }
            }
        });
        return (React.createElement("div", null,
            React.createElement(GameMode, { game_mode: this.props.state.game_mode }),
            React.createElement(Players, { players: this.props.state.players, landlord: this.props.state.landlord, next: next, name: this.props.name }),
            React.createElement("div", null,
                React.createElement("h2", null, "Bids"),
                this.props.state.bids.map(function (bid, idx) {
                    var name = "unknown";
                    _this.props.state.players.forEach(function (player) {
                        if (player.id == bid.id) {
                            name = player.name;
                        }
                    });
                    return (React.createElement(LabeledPlay, { label: name, key: idx, cards: Array(bid.count).fill(bid.card) }));
                })),
            React.createElement("button", { onClick: function (evt) {
                    evt.preventDefault();
                    _this.drawCard();
                }, disabled: !can_draw }, "Draw card"),
            React.createElement("label", null,
                React.createElement("input", { type: "checkbox", name: "autodraw", checked: this.state.autodraw, onChange: this.onAutodrawClicked })),
            React.createElement("button", { onClick: this.makeBid, disabled: this.state.selected.length == 0 }, "Make bid"),
            React.createElement("button", { onClick: this.pickUpKitty, disabled: this.props.state.deck.length > 0 ||
                    this.props.state.bids.length == 0 }, "Pick up cards from the bottom"),
            React.createElement(Cards, { cards: cards_not_bid, selected: this.state.selected, setSelected: this.setSelected })));
    };
    return Draw;
}(React.Component));
var Exchange = /** @class */ (function (_super) {
    __extends(Exchange, _super);
    function Exchange(props) {
        var _this = _super.call(this, props) || this;
        _this.moveCardToKitty = _this.moveCardToKitty.bind(_this);
        _this.moveCardToHand = _this.moveCardToHand.bind(_this);
        _this.startGame = _this.startGame.bind(_this);
        _this.pickFriends = _this.pickFriends.bind(_this);
        _this.state = {
            friends: [],
        };
        _this.fixFriends = _this.fixFriends.bind(_this);
        return _this;
    }
    Exchange.prototype.fixFriends = function () {
        if (this.props.state.game_mode != "Tractor") {
            var game_mode = this.props.state.game_mode.FindingFriends;
            var num_friends = game_mode.num_friends;
            var prop_friends = game_mode.friends;
            if (num_friends != this.state.friends.length) {
                if (prop_friends.length != num_friends) {
                    var friends = __spreadArrays(this.state.friends);
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
                }
                else {
                    this.setState({ friends: prop_friends });
                }
            }
        }
        else {
            if (this.state.friends.length != 0) {
                this.setState({ friends: [] });
            }
        }
    };
    Exchange.prototype.componentDidMount = function () {
        this.fixFriends();
    };
    Exchange.prototype.componentDidUpdate = function () {
        this.fixFriends();
    };
    Exchange.prototype.moveCardToKitty = function (card) {
        send({ Action: { MoveCardToKitty: card } });
    };
    Exchange.prototype.moveCardToHand = function (card) {
        send({ Action: { MoveCardToHand: card } });
    };
    Exchange.prototype.startGame = function (evt) {
        evt.preventDefault();
        send({ Action: "BeginPlay" });
    };
    Exchange.prototype.pickFriends = function (evt) {
        evt.preventDefault();
        if (this.props.state.game_mode != "Tractor" &&
            this.props.state.game_mode.FindingFriends.num_friends ==
                this.state.friends.length) {
            send({
                Action: {
                    SetFriends: this.state.friends,
                },
            });
        }
    };
    Exchange.prototype.render = function () {
        var _this = this;
        var landlord_idx = 0;
        this.props.state.players.forEach(function (player, idx) {
            if (player.id == _this.props.state.landlord) {
                landlord_idx = idx;
            }
        });
        if (this.props.state.players[landlord_idx].name == this.props.name) {
            return (React.createElement("div", null,
                React.createElement(GameMode, { game_mode: this.props.state.game_mode }),
                React.createElement(Players, { players: this.props.state.players, landlord: this.props.state.landlord, next: this.props.state.landlord, name: this.props.name }),
                React.createElement(Trump, { trump: this.props.state.trump }),
                this.props.state.game_mode != "Tractor" ? (React.createElement("div", null,
                    React.createElement(Friends, { game_mode: this.props.state.game_mode }),
                    this.state.friends.map(function (friend, idx) {
                        var onChange = function (x) {
                            var new_friends = __spreadArrays(_this.state.friends);
                            new_friends[idx] = x;
                            _this.setState({ friends: new_friends });
                        };
                        return (React.createElement(FriendSelect, { onChange: onChange, key: idx, friend: friend, trump: _this.props.state.trump, num_decks: _this.props.state.num_decks }));
                    }),
                    React.createElement("button", { onClick: this.pickFriends }, "Pick friends"))) : null,
                React.createElement("h2", null, "Your hand"),
                React.createElement("div", { className: "hand" }, this.props.cards.map(function (c, idx) { return (React.createElement(Card, { key: idx, onClick: function () { return _this.moveCardToKitty(c); }, card: c })); })),
                React.createElement("h2", null,
                    "Discarded cards",
                    " ",
                    this.props.state.kitty.length / this.props.state.kitty_size),
                React.createElement("div", { className: "kitty" }, this.props.state.kitty.map(function (c, idx) { return (React.createElement(Card, { key: idx, onClick: function () { return _this.moveCardToHand(c); }, card: c })); })),
                React.createElement("button", { onClick: this.startGame, disabled: this.props.state.kitty.length != this.props.state.kitty_size }, "Start game")));
        }
        else {
            return (React.createElement("div", null,
                React.createElement(GameMode, { game_mode: this.props.state.game_mode }),
                React.createElement(Players, { players: this.props.state.players, landlord: this.props.state.landlord, next: this.props.state.landlord, name: this.props.name }),
                React.createElement(Trump, { trump: this.props.state.trump }),
                React.createElement("div", { className: "hand" }, this.props.cards.map(function (c, idx) { return (React.createElement(Card, { key: idx, card: c })); })),
                React.createElement("p", null, "Waiting...")));
        }
    };
    return Exchange;
}(React.Component));
var Play = /** @class */ (function (_super) {
    __extends(Play, _super);
    function Play(props) {
        var _this = _super.call(this, props) || this;
        _this.was_my_turn = false;
        _this.state = {
            selected: [],
        };
        _this.setSelected = _this.setSelected.bind(_this);
        _this.playCards = _this.playCards.bind(_this);
        _this.takeBackCards = _this.takeBackCards.bind(_this);
        _this.endTrick = _this.endTrick.bind(_this);
        return _this;
    }
    Play.prototype.setSelected = function (new_selected) {
        this.setState({ selected: new_selected });
    };
    Play.prototype.playCards = function (evt) {
        evt.preventDefault();
        send({ Action: { PlayCards: this.state.selected } });
        this.setSelected([]);
    };
    Play.prototype.takeBackCards = function (evt) {
        evt.preventDefault();
        send({ Action: "TakeBackCards" });
    };
    Play.prototype.endTrick = function (evt) {
        evt.preventDefault();
        send({ Action: "EndTrick" });
    };
    Play.prototype.startNewGame = function (evt) {
        evt.preventDefault();
        send({ Action: "StartNewGame" });
    };
    Play.prototype.render = function () {
        var _this = this;
        var next = this.props.state.trick.player_queue[0];
        var can_take_back = false;
        var can_play = false;
        var is_my_turn = false;
        this.props.state.players.forEach(function (p) {
            if (p.name == _this.props.name) {
                var last_play = _this.props.state.trick.played_cards[_this.props.state.trick.played_cards.length - 1];
                if (p.id == next) {
                    is_my_turn = true;
                    if (last_play) {
                        can_play = _this.state.selected.length == last_play.cards.length;
                    }
                    else {
                        can_play = _this.state.selected.length > 0;
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
        return (React.createElement("div", null,
            React.createElement(GameMode, { game_mode: this.props.state.game_mode }),
            React.createElement(Players, { players: this.props.state.players, landlord: this.props.state.landlord, landlords_team: this.props.state.landlords_team, name: this.props.name, next: next }),
            React.createElement(Trump, { trump: this.props.state.trump }),
            React.createElement(Friends, { game_mode: this.props.state.game_mode }),
            React.createElement(Trick, { trick: this.props.state.trick, players: this.props.state.players }),
            React.createElement("button", { onClick: this.playCards, disabled: !can_play }, "Play selected cards"),
            React.createElement("button", { onClick: this.takeBackCards, disabled: !can_take_back }, "Take back last play"),
            React.createElement("button", { onClick: this.endTrick, disabled: this.props.state.trick.player_queue.length > 0 }, "Finish trick"),
            this.props.cards.length == 0 ? (React.createElement("button", { onClick: this.startNewGame }, "Finish game")) : null,
            ",",
            React.createElement(Cards, { cards: this.props.cards, selected: this.state.selected, setSelected: this.setSelected }),
            this.props.state.last_trick && this.props.show_last_trick ? (React.createElement("div", null,
                React.createElement("p", null, "Previous trick"),
                React.createElement(Trick, { trick: this.props.state.last_trick, players: this.props.state.players }))) : null,
            React.createElement(Points, { points: this.props.state.points, players: this.props.state.players, landlords_team: this.props.state.landlords_team }),
            React.createElement(LabeledPlay, { cards: this.props.state.kitty, label: "\u5E95\u724C" })));
    };
    return Play;
}(React.Component));
var Trick = /** @class */ (function (_super) {
    __extends(Trick, _super);
    function Trick() {
        return _super !== null && _super.apply(this, arguments) || this;
    }
    Trick.prototype.render = function () {
        var _this = this;
        var names_by_id = {};
        this.props.players.forEach(function (p) {
            names_by_id[p.id] = p.name;
        });
        var blank_cards = this.props.trick.played_cards.length > 0
            ? Array(this.props.trick.played_cards[0].cards.length).fill("🂠")
            : ["🂠"];
        return (React.createElement("div", { className: "trick" },
            this.props.trick.played_cards.map(function (played, idx) {
                var winning = _this.props.trick.current_winner == played.id;
                return (React.createElement(LabeledPlay, { key: idx, label: winning
                        ? names_by_id[played.id] + " (!)"
                        : names_by_id[played.id], className: winning ? "winning" : "", cards: played.cards }));
            }),
            this.props.trick.player_queue.map(function (id, idx) {
                return (React.createElement(LabeledPlay, { key: idx + _this.props.trick.played_cards.length, label: names_by_id[id], cards: blank_cards }));
            })));
    };
    return Trick;
}(React.Component));
var Points = /** @class */ (function (_super) {
    __extends(Points, _super);
    function Points() {
        return _super !== null && _super.apply(this, arguments) || this;
    }
    Points.prototype.render = function () {
        var _this = this;
        return e("div", { className: "points" }, this.props.players.map(function (player) {
            var total_points = 0;
            _this.props.points[player.id].forEach(function (c) {
                total_points += CARD_LUT[c].points;
            });
            var className = _this.props.landlords_team.includes(player.id)
                ? "landlord"
                : "";
            var cards = _this.props.points[player.id].length > 0
                ? _this.props.points[player.id]
                : ["🂠"];
            return (React.createElement(LabeledPlay, { key: player.id, className: className, label: player.name + ": " + total_points + "\u5206", cards: cards }));
        }));
    };
    return Points;
}(React.Component));
var Cards = /** @class */ (function (_super) {
    __extends(Cards, _super);
    function Cards(props) {
        var _this = _super.call(this, props) || this;
        _this.selectCard = _this.selectCard.bind(_this);
        _this.unselectCard = _this.unselectCard.bind(_this);
        return _this;
    }
    Cards.prototype.selectCard = function (card) {
        var new_selected = __spreadArrays(this.props.selected);
        new_selected.push(card);
        this.props.setSelected(new_selected);
    };
    Cards.prototype.unselectCard = function (card) {
        var pos = this.props.selected.indexOf(card);
        if (pos >= 0) {
            var new_selected = __spreadArrays(this.props.selected);
            new_selected.splice(pos, 1);
            this.props.setSelected(new_selected);
        }
    };
    Cards.prototype.render = function () {
        var _this = this;
        var unselected = __spreadArrays(this.props.cards);
        this.props.selected.forEach(function (card) {
            unselected.splice(unselected.indexOf(card), 1);
        });
        return e("div", { className: "hand" }, e("div", { className: "selected-cards" }, this.props.selected.map(function (c, idx) {
            return e(Card, { key: idx, onClick: function () { return _this.unselectCard(c); }, card: c });
        }), this.props.selected.length == 0 ? e(Card, { card: "🂠" }) : null), e("p", null, "Your hand"), e("div", { className: "unselected-cards" }, unselected.map(function (c, idx) {
            return e(Card, { key: idx, onClick: function () { return _this.selectCard(c); }, card: c });
        }), unselected.length == 0 ? e(Card, { card: "🂠" }) : null));
    };
    return Cards;
}(React.Component));
var Card = /** @class */ (function (_super) {
    __extends(Card, _super);
    function Card() {
        return _super !== null && _super.apply(this, arguments) || this;
    }
    Card.prototype.render = function () {
        var c = CARD_LUT[this.props.card];
        if (!c) {
            return e("span", { className: "card unknown" }, this.props.card);
        }
        var props = {
            className: "card " + c.typ,
        };
        if (this.props.onClick) {
            props.onClick = this.props.onClick;
        }
        return e("span", props, c.display_value);
    };
    return Card;
}(React.Component));
var LabeledPlay = /** @class */ (function (_super) {
    __extends(LabeledPlay, _super);
    function LabeledPlay() {
        return _super !== null && _super.apply(this, arguments) || this;
    }
    LabeledPlay.prototype.render = function () {
        var className = "labeled-play";
        if (this.props.className) {
            className = className + " " + this.props.className;
        }
        return (React.createElement("div", { className: className },
            React.createElement("div", { className: "play" }, this.props.cards.map(function (card, idx) { return (React.createElement(Card, { card: card, key: idx })); })),
            React.createElement("div", { className: "label" }, "this.props.label")));
    };
    return LabeledPlay;
}(React.Component));
var JoinRoom = /** @class */ (function (_super) {
    __extends(JoinRoom, _super);
    function JoinRoom(props) {
        var _this = _super.call(this, props) || this;
        _this.handleChange = _this.handleChange.bind(_this);
        _this.handleSubmit = _this.handleSubmit.bind(_this);
        return _this;
    }
    JoinRoom.prototype.handleChange = function (event) {
        this.props.setName(event.target.value);
    };
    JoinRoom.prototype.handleSubmit = function (event) {
        event.preventDefault();
        if (this.props.name.length > 0) {
            send({
                room_name: this.props.room_name,
                name: this.props.name,
            });
        }
    };
    JoinRoom.prototype.render = function () {
        return (React.createElement("div", null,
            React.createElement("form", { onSubmit: this.handleSubmit },
                React.createElement("input", { type: "text", placeholder: "name", value: this.props.name, onChange: this.handleChange, autoFocus: true }),
                ",",
                React.createElement("input", { type: "submit", value: "join" }))));
    };
    return JoinRoom;
}(React.Component));
var Trump = /** @class */ (function (_super) {
    __extends(Trump, _super);
    function Trump() {
        return _super !== null && _super.apply(this, arguments) || this;
    }
    Trump.prototype.render = function () {
        if (this.props.trump.Standard) {
            return (React.createElement("div", { className: "trump" },
                "The trump suit is",
                " ",
                React.createElement("span", { className: this.props.trump.Standard.suit }, this.props.trump.Standard.suit),
                ", rank ",
                this.props.trump.Standard.number));
        }
        else if (this.props.trump.NoTrump) {
            return (React.createElement("div", { className: "trump" },
                "No trump, rank ",
                this.props.trump.NoTrump.number));
        }
        else {
            return null;
        }
    };
    return Trump;
}(React.Component));
var Kicker = /** @class */ (function (_super) {
    __extends(Kicker, _super);
    function Kicker(props) {
        var _this = _super.call(this, props) || this;
        _this.state = {
            to_kick: "",
        };
        _this.onChange = _this.onChange.bind(_this);
        _this.kick = _this.kick.bind(_this);
        return _this;
    }
    Kicker.prototype.onChange = function (evt) {
        evt.preventDefault();
        this.setState({ to_kick: evt.target.value });
    };
    Kicker.prototype.kick = function (evt) {
        evt.preventDefault();
        send({ Kick: parseInt(this.state.to_kick, 10) });
    };
    Kicker.prototype.render = function () {
        return e("div", { className: "kicker" }, e("select", { value: this.state.to_kick, onChange: this.onChange }, e("option", { value: "" }, ""), this.props.players.map(function (player) {
            return e("option", { value: player.id, key: player.id }, player.name);
        })), e("button", { onClick: this.kick, disabled: this.state.to_kick == "" }, "kick"));
    };
    return Kicker;
}(React.Component));
var LandlordSelector = /** @class */ (function (_super) {
    __extends(LandlordSelector, _super);
    function LandlordSelector(props) {
        var _this = _super.call(this, props) || this;
        _this.onChange = _this.onChange.bind(_this);
        return _this;
    }
    LandlordSelector.prototype.onChange = function (evt) {
        evt.preventDefault();
        if (evt.target.value != "") {
            send({ Action: { SetLandlord: parseInt(evt.target.value, 10) } });
        }
        else {
            send({ Action: { SetLandlord: null } });
        }
    };
    LandlordSelector.prototype.render = function () {
        return e("div", { className: "landlord-picker" }, e("label", null, "leader: ", e("select", {
            value: this.props.landlord != null ? this.props.landlord : "",
            onChange: this.onChange,
        }, e("option", { value: "" }, "winner of the bid"), this.props.players.map(function (player) {
            return e("option", { value: player.id, key: player.id }, player.name);
        }))));
    };
    return LandlordSelector;
}(React.Component));
var NumDecksSelector = /** @class */ (function (_super) {
    __extends(NumDecksSelector, _super);
    function NumDecksSelector(props) {
        var _this = _super.call(this, props) || this;
        _this.onChange = _this.onChange.bind(_this);
        return _this;
    }
    NumDecksSelector.prototype.onChange = function (evt) {
        evt.preventDefault();
        if (evt.target.value != "") {
            send({ Action: { SetNumDecks: parseInt(evt.target.value, 10) } });
        }
        else {
            send({ Action: { SetNumDecks: null } });
        }
    };
    NumDecksSelector.prototype.render = function () {
        return (React.createElement("div", { className: "num-decks-picker" },
            React.createElement("label", null,
                "number of decks:",
                " ",
                React.createElement("select", { value: this.props.num_decks != null ? this.props.num_decks : "", onChange: this.onChange },
                    React.createElement("option", { value: "" }, "default"),
                    Array(this.props.players.length)
                        .fill(0)
                        .map(function (_, idx) {
                        var val = idx + 1;
                        return (React.createElement("option", { value: val, key: idx }, val));
                    })))));
    };
    return NumDecksSelector;
}(React.Component));
var RankSelector = /** @class */ (function (_super) {
    __extends(RankSelector, _super);
    function RankSelector(props) {
        var _this = _super.call(this, props) || this;
        _this.onChange = _this.onChange.bind(_this);
        return _this;
    }
    RankSelector.prototype.onChange = function (evt) {
        evt.preventDefault();
        if (evt.target.value != "") {
            send({ Action: { SetRank: evt.target.value } });
        }
    };
    RankSelector.prototype.render = function () {
        var _this = this;
        var rank = "";
        this.props.players.forEach(function (p) {
            if (p.name == _this.props.name) {
                rank = p.level;
            }
        });
        return (React.createElement("div", { className: "landlord-picker" },
            React.createElement("label", null,
                "rank:",
                " ",
                React.createElement("select", { value: rank, onChange: this.onChange }, [
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
                ].map(function (rank) { return (React.createElement("option", { value: rank, key: rank }, rank)); })))));
    };
    return RankSelector;
}(React.Component));
var Players = /** @class */ (function (_super) {
    __extends(Players, _super);
    function Players() {
        return _super !== null && _super.apply(this, arguments) || this;
    }
    Players.prototype.movePlayerLeft = function (evt, player_id) {
        evt.preventDefault();
        var player_ids = this.props.players.map(function (p) { return p.id; });
        var index = player_ids.indexOf(player_id);
        if (index > 0) {
            var p = player_ids[index];
            player_ids[index] = player_ids[index - 1];
            player_ids[index - 1] = p;
        }
        else {
            var p = player_ids[index];
            player_ids[index] = player_ids[player_ids.length - 1];
            player_ids[player_ids.length - 1] = p;
        }
        send({ Action: { ReorderPlayers: player_ids } });
    };
    Players.prototype.movePlayerRight = function (evt, player_id) {
        evt.preventDefault();
        var player_ids = this.props.players.map(function (p) { return p.id; });
        var index = player_ids.indexOf(player_id);
        if (index < player_ids.length - 1) {
            var p = player_ids[index];
            player_ids[index] = player_ids[index + 1];
            player_ids[index + 1] = p;
        }
        else {
            var p = player_ids[index];
            player_ids[index] = player_ids[0];
            player_ids[0] = p;
        }
        send({ Action: { ReorderPlayers: player_ids } });
    };
    Players.prototype.render = function () {
        var _this = this;
        return e("table", { className: "players" }, e("tbody", null, e("tr", null, this.props.players.map(function (player) {
            var className = "player";
            var descriptor = player.name + " (rank " + player.level + ")";
            if (player.id == _this.props.landlord) {
                descriptor = descriptor + " (当庄)";
            }
            if (player.name == _this.props.name) {
                descriptor = descriptor + " (You!)";
            }
            if (player.id == _this.props.landlord ||
                (_this.props.landlords_team &&
                    _this.props.landlords_team.includes(player.id))) {
                className = className + " landlord";
            }
            if (player.id == _this.props.next) {
                className = className + " next";
            }
            return e("td", { key: player.id, className: className }, _this.props.movable
                ? e("button", {
                    onClick: function (evt) {
                        return _this.movePlayerLeft(evt, player.id);
                    },
                }, "<")
                : null, descriptor, _this.props.movable
                ? e("button", {
                    onClick: function (evt) {
                        return _this.movePlayerRight(evt, player.id);
                    },
                }, ">")
                : null);
        }))));
    };
    return Players;
}(React.Component));
var Chat = /** @class */ (function (_super) {
    __extends(Chat, _super);
    function Chat(props) {
        var _this = _super.call(this, props) || this;
        _this.anchor = React.createRef();
        _this.state = { message: "" };
        _this.handleChange = _this.handleChange.bind(_this);
        _this.handleSubmit = _this.handleSubmit.bind(_this);
        return _this;
    }
    Chat.prototype.componentDidMount = function () {
        if (this.anchor.current) {
            this.anchor.current.scrollIntoView({ block: "nearest", inline: "start" });
        }
    };
    Chat.prototype.componentDidUpdate = function () {
        if (this.anchor.current) {
            this.anchor.current.scrollIntoView({ block: "nearest", inline: "start" });
        }
    };
    Chat.prototype.handleChange = function (event) {
        this.setState({ message: event.target.value });
    };
    Chat.prototype.handleSubmit = function (event) {
        event.preventDefault();
        if (this.state.message.length > 0) {
            send({
                Message: this.state.message,
            });
        }
        this.setState({ message: "" });
    };
    Chat.prototype.render = function () {
        return (React.createElement("div", { className: "chat" },
            React.createElement("div", { className: "messages" },
                this.props.messages.map(function (m, idx) {
                    var className = "message";
                    if (m.from_game) {
                        className = className + " game-message";
                    }
                    return (React.createElement("p", { key: idx, className: className },
                        m.from,
                        ": ",
                        m.message));
                }),
                React.createElement("div", { className: "chat-anchor", ref: this.anchor })),
            React.createElement("form", { onSubmit: this.handleSubmit },
                React.createElement("input", { type: "text", placeholder: "type message here", value: this.state.message, onChange: this.handleChange }),
                React.createElement("input", { type: "submit", value: "submit" }))));
    };
    return Chat;
}(React.Component));
var GameMode = /** @class */ (function (_super) {
    __extends(GameMode, _super);
    function GameMode() {
        return _super !== null && _super.apply(this, arguments) || this;
    }
    GameMode.prototype.render = function () {
        if (this.props.game_mode == "Tractor") {
            return React.createElement("h1", null, "\u5347\u7EA7 / Tractor");
        }
        else {
            return React.createElement("h1", null, "\u627E\u670B\u53CB / Finding Friends");
        }
    };
    return GameMode;
}(React.Component));
var Friends = /** @class */ (function (_super) {
    __extends(Friends, _super);
    function Friends() {
        return _super !== null && _super.apply(this, arguments) || this;
    }
    Friends.prototype.render = function () {
        if (this.props.game_mode != "Tractor") {
            return e("div", { className: "pending-friends" }, this.props.game_mode.FindingFriends.friends.map(function (friend, idx) {
                if (friend.player_id != null) {
                    return null;
                }
                var c = CARD_LUT[friend.card];
                if (!c) {
                    return null;
                }
                var card = "" + c.number + c.typ;
                if (friend.skip == 0) {
                    return e("p", { key: idx }, "The next person to play ", e("span", { className: c.typ }, "" + c.number + c.typ), " is a friend");
                }
                else {
                    return e("p", { key: idx }, friend.skip + " ", e("span", { className: c.typ }, "" + c.number + c.typ), " can be played before the next person to play ", e("span", { className: c.typ }, "" + c.number + c.typ), " is a friend");
                }
            }));
        }
        else {
            return null;
        }
    };
    return Friends;
}(React.Component));
var FriendSelect = /** @class */ (function (_super) {
    __extends(FriendSelect, _super);
    function FriendSelect(props) {
        var _this = _super.call(this, props) || this;
        _this.onCardChange = _this.onCardChange.bind(_this);
        _this.onOrdinalChange = _this.onOrdinalChange.bind(_this);
        return _this;
    }
    FriendSelect.prototype.onCardChange = function (evt) {
        evt.preventDefault();
        this.props.onChange({
            card: evt.target.value,
            skip: this.props.friend.skip,
        });
    };
    FriendSelect.prototype.onOrdinalChange = function (evt) {
        evt.preventDefault();
        this.props.onChange({
            card: this.props.friend.card,
            skip: parseInt(evt.target.value, 10),
        });
    };
    FriendSelect.prototype.render = function () {
        var _a;
        var number = this.props.trump.Standard
            ? this.props.trump.Standard.number
            : (_a = this.props.trump.NoTrump) === null || _a === void 0 ? void 0 : _a.number;
        return e("div", { className: "friend-select" }, e("select", { value: this.props.friend.card, onChange: this.onCardChange }, e("option", { value: "" }, " "), CARDS.map(function (c) {
            return c.number != null && c.number != number
                ? e("option", { key: c.value, value: c.value }, "" + c.number + c.typ)
                : null;
        })), e("select", { value: this.props.friend.skip, onChange: this.onOrdinalChange }, Array(this.props.num_decks)
            .fill(1)
            .map(function (_, idx) {
            return e("option", { key: idx, value: idx }, idx + 1);
        })));
    };
    return FriendSelect;
}(React.Component));
var Errors = /** @class */ (function (_super) {
    __extends(Errors, _super);
    function Errors() {
        return _super !== null && _super.apply(this, arguments) || this;
    }
    Errors.prototype.render = function () {
        return e("div", { className: "errors" }, this.props.errors.map(function (err, idx) {
            return e("p", { key: idx }, e("code", null, err));
        }));
    };
    return Errors;
}(React.Component));
if (window.location.hash.length != 17) {
    var arr = new Uint8Array(8);
    window.crypto.getRandomValues(arr);
    var r = Array.from(arr, function (d) { return ("0" + d.toString(16)).substr(-2); }).join("");
    window.location.hash = r;
}
var uri = (location.protocol == "https:" ? "wss://" : "ws://") +
    location.host +
    location.pathname +
    (location.pathname.endsWith("/") ? "api" : "/api");
var ws = new WebSocket(uri);
var state = {
    connected: false,
    room_name: window.location.hash.slice(1),
    name: window.localStorage.getItem("name") || "",
    game_state: null,
    four_color: window.localStorage.getItem("four_color") == "on" || false,
    beep_on_turn: window.localStorage.getItem("beep_on_turn") == "on" || false,
    show_last_trick: window.localStorage.getItem("show_last_trick") == "on" || false,
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
            ReactDOM.render(e("div", null, e("h2", null, "Room Name: " + state.room_name), e(Errors, { errors: state.errors }), e(JoinRoom, {
                name: state.name,
                room_name: state.room_name,
                setName: function (name) {
                    state.name = name;
                    window.localStorage.setItem("name", name);
                    renderUI();
                },
            })), document.getElementById("root"));
        }
        else {
            ReactDOM.render(e("div", { className: state.four_color ? "four-color" : "" }, e(Errors, { errors: state.errors }), e("div", { className: "game" }, state.game_state.Initialize
                ? e(Initialize, {
                    state: state.game_state.Initialize,
                    cards: state.cards,
                    name: state.name,
                })
                : null, state.game_state.Draw
                ? e(Draw, {
                    state: state.game_state.Draw,
                    cards: state.cards,
                    name: state.name,
                })
                : null, state.game_state.Exchange
                ? e(Exchange, {
                    state: state.game_state.Exchange,
                    cards: state.cards,
                    name: state.name,
                })
                : null, state.game_state.Play
                ? e(Play, {
                    state: state.game_state.Play,
                    cards: state.cards,
                    name: state.name,
                    show_last_trick: state.show_last_trick,
                    beep_on_turn: state.beep_on_turn,
                })
                : null, state.game_state.Done ? e("p", null, "Game over") : null), e(Chat, { messages: state.messages }), e("hr", null), e("div", { className: "settings" }, e("label", null, "four-color mode", e("input", {
                name: "four-color",
                type: "checkbox",
                checked: state.four_color,
                onChange: function (evt) {
                    state.four_color = evt.target.checked;
                    if (state.four_color) {
                        window.localStorage.setItem("four_color", "on");
                    }
                    else {
                        window.localStorage.setItem("four_color", "off");
                    }
                    renderUI();
                },
            })), e("label", null, "show last trick", e("input", {
                name: "show-last-trick",
                type: "checkbox",
                checked: state.show_last_trick,
                onChange: function (evt) {
                    state.show_last_trick = evt.target.checked;
                    if (state.show_last_trick) {
                        window.localStorage.setItem("show_last_trick", "on");
                    }
                    else {
                        window.localStorage.setItem("show_last_trick", "off");
                    }
                    renderUI();
                },
            })), e("label", null, "beep on turn", e("input", {
                name: "show-last-trick",
                type: "checkbox",
                checked: state.beep_on_turn,
                onChange: function (evt) {
                    state.beep_on_turn = evt.target.checked;
                    if (state.beep_on_turn) {
                        window.localStorage.setItem("beep_on_turn", "on");
                    }
                    else {
                        window.localStorage.setItem("beep_on_turn", "off");
                    }
                    renderUI();
                },
            })))), document.getElementById("root"));
        }
    }
    else {
        ReactDOM.render(React.createElement("p", null, "disconnected from server, please refresh"), document.getElementById("root"));
    }
}
ws.onopen = function () {
    state.connected = true;
    renderUI();
};
ws.onclose = function (evt) {
    state.connected = false;
    renderUI();
};
ws.onmessage = function (event) {
    var msg = JSON.parse(event.data);
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
        setTimeout(function () {
            state.errors = state.errors.filter(function (v) { return v != msg.Error; });
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
var beepContext = new AudioContext();
function beep(vol, freq, duration) {
    var v = beepContext.createOscillator();
    var u = beepContext.createGain();
    v.connect(u);
    v.frequency.value = freq;
    v.type = "square";
    u.connect(beepContext.destination);
    u.gain.value = vol * 0.01;
    v.start(beepContext.currentTime);
    v.stop(beepContext.currentTime + duration * 0.001);
}
