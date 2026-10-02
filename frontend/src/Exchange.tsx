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
import RiverCrossingNotice from "./RiverCrossingNotice";
import LabeledPlay from "./LabeledPlay";
import { ExchangePhase, Friend } from "./gen-types";
import Cards from "./Cards";
import { prefillCardInfoCache } from "./util/cachePrefill";
import { useEngine } from "./useEngine";

import type { JSX } from "react";

interface IExchangeProps {
  state: ExchangePhase;
  name: string;
}

// Wrapper component to handle cache prefilling with hooks
function ExchangeWrapper(props: IExchangeProps) {
  const engine = useEngine();

  React.useEffect(() => {
    if (props.state.trump && engine) {
      // Prefill cache for trump in Exchange component
      prefillCardInfoCache(engine, props.state.trump);
    }
  }, [props.state.trump, engine]);

  return <Exchange {...props} />;
}

interface IExchangeState {
  friends: Friend[];
  selectedRiverCards: string[];
}
class Exchange extends React.Component<IExchangeProps, IExchangeState> {
  constructor(props: IExchangeProps) {
    super(props);
    this.moveCardToKitty = this.moveCardToKitty.bind(this);
    this.moveCardToHand = this.moveCardToHand.bind(this);
    this.startGame = this.startGame.bind(this);
    this.declineKittyTheft = this.declineKittyTheft.bind(this);
    this.putDownKitty = this.putDownKitty.bind(this);
    this.pickFriends = this.pickFriends.bind(this);
    this.state = {
      friends: [],
      selectedRiverCards: [],
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

  declineKittyTheft(evt: React.SyntheticEvent): void {
    evt.preventDefault();
    (window as any).send({ Action: "DeclineKittyTheft" });
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

    const riverCrossing = this.props.state.river_crossing;
    if (riverCrossing !== null && riverCrossing !== undefined) {
      const riverCrossingInitiators =
        riverCrossing.stage === "Deciding"
          ? []
          : riverCrossing.players
              .filter((player) => player.decision === true)
              .map((player) => player.player_id);
      const playerState = riverCrossing.players.find(
        (p) => p.player_id === playerId,
      );
      const playerPosition = this.props.state.propagated.players.findIndex(
        (p) => p.id === playerId,
      );
      const partnerId =
        playerPosition >= 0 && this.props.state.propagated.players.length === 4
          ? this.props.state.propagated.players[(playerPosition + 2) % 4].id
          : -1;
      const partnerState = riverCrossing.players.find(
        (p) => p.player_id === partnerId,
      );

      let actionUI: JSX.Element = (
        <p>Waiting for other players... / 等待其他玩家……</p>
      );
      if (
        riverCrossing.stage === "Deciding" &&
        playerState?.eligible &&
        playerState.decision === null
      ) {
        actionUI = (
          <>
            <p>
              You have five or fewer trump cards. Would you like to cross the
              river?（你的主牌不超过五张，是否选择五主过河？）
            </p>
            <button
              className="big"
              onClick={() =>
                (window as any).send({
                  Action: { DecideRiverCrossing: true },
                })
              }
            >
              Cross the river / 过河
            </button>
            <button
              className="big"
              onClick={() =>
                (window as any).send({
                  Action: { DecideRiverCrossing: false },
                })
              }
            >
              Decline / 不过河
            </button>
          </>
        );
      } else if (
        riverCrossing.stage === "Deciding" &&
        playerState !== undefined &&
        !playerState.eligible
      ) {
        actionUI = (
          <p>
            You have more than five trump cards and cannot cross. Waiting for
            other players... / 你的主牌多于五张，不能过河。正在等待其他玩家……
          </p>
        );
      } else if (
        riverCrossing.stage === "SelectingCrossingCards" &&
        playerState?.decision === true &&
        !playerState.crossing_cards_submitted
      ) {
        actionUI = (
          <>
            <p>
              Select exactly five cards, including every trump card in your
              hand. They will be revealed to your partner only after all
              crossing selections are submitted. /
              请选择正好五张牌，其中必须包含手中全部主牌；所有需要过河的玩家提交后才会同时交换。
            </p>
            <Cards
              hands={this.props.state.hands}
              playerId={playerId}
              selectedCards={this.state.selectedRiverCards}
              onSelect={(selectedRiverCards) =>
                this.setState({ selectedRiverCards })
              }
              trump={this.props.state.trump}
            />
            <button
              className="big"
              disabled={this.state.selectedRiverCards.length !== 5}
              onClick={() => {
                (window as any).send({
                  Action: {
                    SubmitRiverCrossingCards: this.state.selectedRiverCards,
                  },
                });
                this.setState({ selectedRiverCards: [] });
              }}
            >
              Submit five crossing cards / 提交五张过河牌
            </button>
          </>
        );
      } else if (
        riverCrossing.stage === "SelectingReturnCards" &&
        playerState !== undefined &&
        partnerState?.decision === true &&
        !playerState.return_cards_submitted
      ) {
        actionUI = (
          <>
            <p>
              Your partner sent you these cards. Select exactly five cards to
              return. Return selections are exchanged simultaneously. /
              这是对家给你的牌。请从现在的手牌中选择正好五张归还；双方都提交后才会同时还牌。
            </p>
            {playerState.received_crossing_cards.length > 0 ? (
              <LabeledPlay
                className="kitty"
                trump={this.props.state.trump}
                cards={playerState.received_crossing_cards}
                label="Cards received / 收到的过河牌"
              />
            ) : null}
            <Cards
              hands={this.props.state.hands}
              playerId={playerId}
              selectedCards={this.state.selectedRiverCards}
              onSelect={(selectedRiverCards) =>
                this.setState({ selectedRiverCards })
              }
              trump={this.props.state.trump}
            />
            <button
              className="big"
              disabled={this.state.selectedRiverCards.length !== 5}
              onClick={() => {
                (window as any).send({
                  Action: {
                    SubmitRiverReturnCards: this.state.selectedRiverCards,
                  },
                });
                this.setState({ selectedRiverCards: [] });
              }}
            >
              Submit five return cards / 提交五张还牌
            </button>
          </>
        );
      }

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
            riverCrossingInitiators={riverCrossingInitiators}
          />
          <RiverCrossingNotice
            players={this.props.state.propagated.players}
            initiators={riverCrossingInitiators}
          />
          <Trump trump={this.props.state.trump} />
          <h2>Five-trump river crossing / 五主过河</h2>
          {actionUI}
        </div>
      );
    }

    const kittyTheft = this.props.state.kitty_theft;
    const kittyTheftStage = kittyTheft?.stage ?? "Disabled";
    const currentKittyTheftPlayer = kittyTheft?.current_player ?? null;
    const activeKittyTheftPlayer =
      kittyTheftStage === "Waiting"
        ? currentKittyTheftPlayer
        : this.props.state.exchanger;
    const currentKittyTheftPlayerName =
      this.props.state.propagated.players.find(
        (player) => player.id === activeKittyTheftPlayer,
      )?.name ?? "another player";
    const nextPlayer =
      kittyTheftStage === "Waiting" && currentKittyTheftPlayer !== null
        ? currentKittyTheftPlayer
        : kittyTheftStage === "Exchanging" &&
            this.props.state.exchanger !== null
          ? this.props.state.exchanger
          : this.props.state.landlord;
    const kittyTheftComplete =
      !kittyTheftEnabled ||
      this.props.state.autobid !== null ||
      kittyTheftStage === "Disabled" ||
      kittyTheftStage === "Complete";

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
              <Card
                trump={this.props.state.trump}
                key={idx}
                onClick={() => this.moveCardToHand(c)}
                collapseRight={idx !== this.props.state.kitty.length - 1}
                card={c}
              />
            ))}
          </div>
          {kittyTheftEnabled ? (
            <button
              onClick={this.putDownKitty}
              disabled={
                this.props.state.kitty.length !== this.props.state.kitty_size
              }
              className="big"
            >
              Finalize exchanged cards
            </button>
          ) : null}
        </>
      ) : null;

    const startGame = (
      <button
        onClick={this.startGame}
        disabled={
          this.props.state.kitty.length !== this.props.state.kitty_size ||
          !kittyTheftComplete
        }
        className="big"
      >
        Start game
      </button>
    );
    const bidUI =
      kittyTheftEnabled &&
      this.props.state.finalized &&
      this.props.state.autobid === null &&
      kittyTheftStage === "Waiting" &&
      currentKittyTheftPlayer === playerId ? (
        <>
          <BidArea
            bids={this.props.state.bids!}
            autobid={this.props.state.autobid}
            hands={this.props.state.hands}
            epoch={this.props.state.epoch!}
            name={this.props.name}
            landlord={this.props.state.propagated.landlord!}
            players={this.props.state.propagated.players}
            bidPolicy={this.props.state.propagated.bid_policy!}
            bidReinforcementPolicy={
              this.props.state.propagated.bid_reinforcement_policy!
            }
            jokerBidPolicy={this.props.state.propagated.joker_bid_policy!}
            numDecks={this.props.state.num_decks}
            header={
              <h2>
                Your turn to decide whether to steal the bottom /
                轮到你决定是否炒底
              </h2>
            }
            suffixButtons={
              <button onClick={this.declineKittyTheft} className="big">
                Decline / 放弃炒底
              </button>
            }
            bidTakeBacksEnabled={false}
          />
          <LabeledPlay
            className="kitty"
            trump={this.props.state.trump}
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
            const onChange = (x: Friend): void => {
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
                  this.props.state.propagated.friend_selection_policy!
                }
                num_decks={this.props.state.num_decks}
              />
            );
          })}
          <button onClick={this.pickFriends} className="big">
            Pick friends
          </button>
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
          next={nextPlayer}
          name={this.props.name}
        />
        <Trump trump={this.props.state.trump} />
        {(this.props.state.removed_cards || []).length > 0 ? (
          <p>
            Note:{" "}
            {(this.props.state.removed_cards || []).map((c) => (
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
            <p role="status" aria-live="polite">
              {kittyTheftStage === "Waiting"
                ? `Waiting for ${currentKittyTheftPlayerName} to decide whether to steal the bottom... / 正在等待 ${currentKittyTheftPlayerName} 决定是否炒底……`
                : kittyTheftStage === "Exchanging"
                  ? `Waiting for ${currentKittyTheftPlayerName} to finish exchanging cards... / 正在等待埋底完成……`
                  : "Waiting..."}
            </p>
          </>
        ) : null}
        {playerId !== nextPlayer && <BeepButton />}
        {isLandlord && bidUI === null && kittyTheftComplete ? startGame : null}
        {bidUI}
      </div>
    );
  }
}

export default ExchangeWrapper;
