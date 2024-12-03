import * as React from "react";
import ReactTooltip from "react-tooltip";
import * as ReactModal from "react-modal";
import { EmojiStyle } from "emoji-picker-react";
import ReadyCheck from "./ReadyCheck";
import LandlordSelector from "./LandlordSelector";
import NumDecksSelector from "./NumDecksSelector";
import KittySizeSelector from "./KittySizeSelector";
import RankSelector from "./RankSelector";
import Kicker from "./Kicker";
import ArrayUtils from "./util/array";
import { RandomizePlayersButton } from "./RandomizePlayersButton";
import {
  InitializePhase,
  Player,
  PropagatedState,
  Deck,
  TractorRequirements,
} from "./gen-types";
import { WebsocketContext } from "./WebsocketProvider";

import Header from "./Header";
import Players from "./Players";
import { GameScoringSettings } from "./ScoringSettings";

const Picker = React.lazy(async () => await import("emoji-picker-react"));

interface IDifficultyProps {
  state: InitializePhase;
  setFriendSelectionPolicy: (v: React.ChangeEvent<HTMLSelectElement>) => void;
  setMultipleJoinPolicy: (v: React.ChangeEvent<HTMLSelectElement>) => void;
  setAdvancementPolicy: (v: React.ChangeEvent<HTMLSelectElement>) => void;
  setMaxRank: (v: React.ChangeEvent<HTMLSelectElement>) => void;
  setHideLandlordsPoints: (v: React.ChangeEvent<HTMLSelectElement>) => void;
  setHidePlayedCards: (v: React.ChangeEvent<HTMLSelectElement>) => void;
  setKittyPenalty: (v: React.ChangeEvent<HTMLSelectElement>) => void;
  setThrowPenalty: (v: React.ChangeEvent<HTMLSelectElement>) => void;
  setPlayTakebackPolicy: (v: React.ChangeEvent<HTMLSelectElement>) => void;
  setBidTakebackPolicy: (v: React.ChangeEvent<HTMLSelectElement>) => void;
}

const contentStyle: React.CSSProperties = {
  position: "absolute",
  top: "50%",
  left: "50%",
  width: "80%",
  transform: "translate(-50%, -50%)",
};

const DifficultySettings = (props: IDifficultyProps): JSX.Element => {
  const [modalOpen, setModalOpen] = React.useState<boolean>(false);
  const s = (
    <>
      <div>
        <label>
          Friend selection restriction:{" "}
          <select
            value={props.state.propagated.friend_selection_policy}
            onChange={props.setFriendSelectionPolicy}
          >
            <option value="Unrestricted">Non-trump cards</option>
            <option value="TrumpsIncluded">All cards, including trumps</option>
            <option value="HighestCardNotAllowed">
              Non-trump cards, except the highest
            </option>
            <option value="PointCardNotAllowed">
              Non-trump, non-point cards (except K when playing A)
            </option>
          </select>
        </label>
      </div>
      <div>
        <label>
          Multiple joining policy:{" "}
          <select
            value={props.state.propagated.multiple_join_policy}
            onChange={props.setMultipleJoinPolicy}
          >
            <option value="Unrestricted">
              Players can join the defending team multiple times.
            </option>
            <option value="NoDoubleJoin">
              Each player can only join the defending team once.
            </option>
          </select>
        </label>
      </div>
      <div>
        <label>
          Rank advancement policy:{" "}
          <select
            value={props.state.propagated.advancement_policy}
            onChange={props.setAdvancementPolicy}
          >
            <option value="Unrestricted">A must be defended</option>
            <option value="FullyUnrestricted">Unrestricted</option>
            <option value="DefendPoints">
              Points (5, 10, K) and A must be defended
            </option>
          </select>
        </label>
      </div>
      <div>
        <label>
          Max rank:{" "}
          <select
            value={props.state.propagated.max_rank}
            onChange={props.setMaxRank}
          >
            <option value="NT">No trump</option>
            <option value="A">A</option>
          </select>
        </label>
      </div>
      <div>
        <label>
          Point visibility:{" "}
          <select
            value={
              props.state.propagated.hide_landlord_points ? "hide" : "show"
            }
            onChange={props.setHideLandlordsPoints}
          >
            <option value="show">Show all players&apos; points</option>
            <option value="hide">Hide defending team&apos;s points</option>
          </select>
        </label>
      </div>
      <div>
        <label>
          Played card visibility (in chat):{" "}
          <select
            value={props.state.propagated.hide_played_cards ? "hide" : "show"}
            onChange={props.setHidePlayedCards}
          >
            <option value="show">Show played cards in chat</option>
            <option value="hide">Hide played cards in chat</option>
          </select>
        </label>
      </div>
      <div>
        <label>
          Penalty for points left in the bottom:{" "}
          <select
            value={props.state.propagated.kitty_penalty}
            onChange={props.setKittyPenalty}
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
          Penalty for incorrect throws:{" "}
          <select
            value={props.state.propagated.throw_penalty}
            onChange={props.setThrowPenalty}
          >
            <option value="None">No penalty</option>
            <option value="TenPointsPerAttempt">
              Ten points per bad throw
            </option>
          </select>
        </label>
      </div>
      <div>
        <label>
          Play takeback:{" "}
          <select
            value={props.state.propagated.play_takeback_policy}
            onChange={props.setPlayTakebackPolicy}
          >
            <option value="AllowPlayTakeback">Allow taking back plays</option>
            <option value="NoPlayTakeback">Disallow taking back plays</option>
          </select>
        </label>
      </div>
      <div>
        <label>
          Bid takeback:{" "}
          <select
            value={props.state.propagated.bid_takeback_policy}
            onChange={props.setBidTakebackPolicy}
          >
            <option value="AllowBidTakeback">Allow bid takeback</option>
            <option value="NoBidTakeback">No bid takeback</option>
          </select>
        </label>
      </div>
    </>
  );

  return (
    <div>
      <label>
        Difficulty settings:{" "}
        <button
          className="normal"
          onClick={(evt) => {
            evt.preventDefault();
            setModalOpen(true);
          }}
        >
          Open
        </button>
        <ReactModal
          isOpen={modalOpen}
          onRequestClose={() => setModalOpen(false)}
          shouldCloseOnOverlayClick
          shouldCloseOnEsc
          style={{ content: contentStyle }}
        >
          {s}
        </ReactModal>
      </label>
    </div>
  );
};

interface IDeckSettings {
  decks: Deck[];
  setSpecialDecks: (specialDecks: Deck[]) => void;
}

const DeckSettings = (props: IDeckSettings): JSX.Element => {
  const [modalOpen, setModalOpen] = React.useState<boolean>(false);
  const isNotDefault = (d: Deck): boolean =>
    !(d.min === "2" && !d.exclude_big_joker && !d.exclude_small_joker);
  const onChange = (decks: Deck[]): void => {
    // exclude the decks that are the same as default
    const filtered = decks.filter((d) => isNotDefault(d));
    props.setSpecialDecks(filtered);
  };

  const setDeckAtIndex = (deck: Deck, index: number): void => {
    const newDecks = [...props.decks];
    newDecks[index] = deck;
    onChange(newDecks);
  };
  const numbers = [
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
    "Q",
    "K",
    "A",
  ];

  const s = (
    <>
      {props.decks.map((d, i) => (
        <div
          key={i}
          style={{
            display: "inline-block",
            border: "1px solid #000",
            padding: "5px",
            margin: "5px",
          }}
        >
          Deck {i + 1}
          {isNotDefault(d) ? " (modified)" : " (standard)"}
          <form>
            <label style={{ display: "block" }}>
              Include HJ (大王){" "}
              <input
                type="checkbox"
                checked={!d.exclude_big_joker}
                onChange={(evt) =>
                  setDeckAtIndex(
                    { ...d, exclude_big_joker: !evt.target.checked },
                    i,
                  )
                }
              />
            </label>
            <label style={{ display: "block" }}>
              Include LJ (小王){" "}
              <input
                type="checkbox"
                checked={!d.exclude_small_joker}
                onChange={(evt) =>
                  setDeckAtIndex(
                    { ...d, exclude_small_joker: !evt.target.checked },
                    i,
                  )
                }
              />
            </label>
            <label>
              Minimum card:{" "}
              <select
                value={d.min}
                onChange={(evt) =>
                  setDeckAtIndex({ ...d, min: evt.target.value }, i)
                }
              >
                {numbers.map((n) => (
                  <option key={n} value={n}>
                    {n}
                  </option>
                ))}
              </select>
            </label>
          </form>
        </div>
      ))}
    </>
  );

  return (
    <div>
      <label>
        More deck customization:{" "}
        <button
          className="normal"
          onClick={(evt) => {
            evt.preventDefault();
            setModalOpen(true);
          }}
        >
          Open
        </button>
        <ReactModal
          isOpen={modalOpen}
          onRequestClose={() => setModalOpen(false)}
          shouldCloseOnOverlayClick
          shouldCloseOnEsc
          style={{ content: contentStyle }}
        >
          {s}
        </ReactModal>
      </label>
    </div>
  );
};

interface ITractorRequirementsProps {
  tractorRequirements: TractorRequirements;
  numDecks: number;
  onChange: (requirements: TractorRequirements) => void;
}

const TractorRequirementsE = (
  props: ITractorRequirementsProps,
): JSX.Element => {
  return (
    <div>
      <label>Tractor requirements: </label>
      <input
        type="number"
        style={{ width: "3em" }}
        onChange={(v) =>
          props.onChange({
            ...props.tractorRequirements,
            min_count: v.target.valueAsNumber,
          })
        }
        value={props.tractorRequirements.min_count}
        min="2"
        max={props.numDecks}
      />
      <label> cards wide by </label>
      <input
        type="number"
        style={{ width: "3em" }}
        onChange={(v) =>
          props.onChange({
            ...props.tractorRequirements,
            min_length: v.target.valueAsNumber,
          })
        }
        value={props.tractorRequirements.min_length}
        min="2"
        max="12"
      />
      <label> tuples long</label>
    </div>
  );
};

interface IScoringSettings {
  state: InitializePhase;
  decks: Deck[];
}
const ScoringSettings = (props: IScoringSettings): JSX.Element => {
  const [modalOpen, setModalOpen] = React.useState<boolean>(false);
  return (
    <div>
      <label>
        Scoring settings:{" "}
        <button
          className="normal"
          onClick={(evt) => {
            evt.preventDefault();
            setModalOpen(true);
          }}
        >
          Open
        </button>
        <ReactModal
          isOpen={modalOpen}
          onRequestClose={() => setModalOpen(false)}
          shouldCloseOnOverlayClick
          shouldCloseOnEsc
          style={{ content: contentStyle }}
        >
          <GameScoringSettings
            params={props.state.propagated.game_scoring_parameters!}
            decks={props.decks}
          />
        </ReactModal>
      </label>
    </div>
  );
};

interface IUncommonSettings {
  state: InitializePhase;
  numDecksEffective: number;
  setBidPolicy: (v: React.ChangeEvent<HTMLSelectElement>) => void;
  setBidReinforcementPolicy: (v: React.ChangeEvent<HTMLSelectElement>) => void;
  setJokerBidPolicy: (v: React.ChangeEvent<HTMLSelectElement>) => void;
  setShouldRevealKittyAtEndOfGame: (
    v: React.ChangeEvent<HTMLSelectElement>,
  ) => void;
  setFirstLandlordSelectionPolicy: (
    v: React.ChangeEvent<HTMLSelectElement>,
  ) => void;
  setGameStartPolicy: (v: React.ChangeEvent<HTMLSelectElement>) => void;
  setGameShadowingPolicy: (v: React.ChangeEvent<HTMLSelectElement>) => void;
  setKittyBidPolicy: (v: React.ChangeEvent<HTMLSelectElement>) => void;
  setHideThrowHaltingPlayer: (v: React.ChangeEvent<HTMLSelectElement>) => void;
  setTractorRequirements: (v: TractorRequirements) => void;
}

const UncommonSettings = (props: IUncommonSettings): JSX.Element => {
  const [modalOpen, setModalOpen] = React.useState<boolean>(false);
  const s = (
    <>
      <div>
        <label>
          Game shadowing policy:{" "}
          <select
            value={props.state.propagated.game_shadowing_policy}
            onChange={props.setGameShadowingPolicy}
          >
            <option value="AllowMultipleSessions">
              Allow players to be shadowed by joining with the same name
            </option>
            <option value="SingleSessionOnly">
              Do not allow players to be shadowed
            </option>
          </select>
        </label>
      </div>
      <div>
        <label>
          Game start policy:{" "}
          <select
            value={props.state.propagated.game_start_policy}
            onChange={props.setGameStartPolicy}
          >
            <option value="AllowAnyPlayer">
              Allow any player to start a game
            </option>
            <option value="AllowLandlordOnly">
              Allow only landlord to start a game
            </option>
          </select>
        </label>
      </div>
      <div>
        <label>
          Landlord selection from bid:{" "}
          <select
            value={props.state.propagated.first_landlord_selection_policy}
            onChange={props.setFirstLandlordSelectionPolicy}
          >
            <option value="ByWinningBid">
              Winning bid decides both landlord and trump
            </option>
            <option value="ByFirstBid">
              First bid decides landlord, winning bid decides trump
            </option>
          </select>
        </label>
      </div>
      <div>
        <label>
          Trump policy for cards revealed from the bottom:{" "}
          <select
            value={props.state.propagated.kitty_bid_policy}
            onChange={props.setKittyBidPolicy}
          >
            <option value="FirstCard">First card revealed</option>
            <option value="FirstCardOfLevelOrHighest">
              First card revealed of the appropriate rank
            </option>
          </select>
        </label>
      </div>
      <div>
        <label>
          Bid policy:{" "}
          <select
            value={props.state.propagated.bid_policy}
            onChange={props.setBidPolicy}
          >
            <option value="JokerOrHigherSuit">
              Joker or higher suit bids to outbid non-joker bids with the same
              number of cards
            </option>
            <option value="JokerOrGreaterLength">
              Joker bids to outbid non-joker bids with the same number of cards
            </option>
            <option value="GreaterLength">
              All bids must have more cards than the previous bids
            </option>
          </select>
        </label>
      </div>
      <div>
        <label>
          Bid reinforcement policy:{" "}
          <select
            value={props.state.propagated.bid_reinforcement_policy}
            onChange={props.setBidReinforcementPolicy}
          >
            <option value="ReinforceWhileWinning">
              The current winning bid can be reinforced
            </option>
            <option value="ReinforceWhileEquivalent">
              A bid can be reinforced after it is overturned
            </option>
            <option value="OverturnOrReinforceWhileWinning">
              The current winning bid can be overturned by the same bidder
            </option>
          </select>
        </label>
      </div>
      <div>
        <label>
          Joker bid policy:{" "}
          <select
            value={props.state.propagated.joker_bid_policy}
            onChange={props.setJokerBidPolicy}
          >
            <option value="BothTwoOrMore">
              At least two jokers (or number of decks) to bid no trump
            </option>
            <option value="BothNumDecks">
              All the low or high jokers to bid no trump
            </option>
            <option value="LJNumDecksHJNumDecksLessOne">
              All the low jokers or all but one high joker to bid no trump
            </option>
            <option value="Disabled">No trump / joker bids disabled</option>
          </select>
        </label>
      </div>
      <TractorRequirementsE
        tractorRequirements={props.state.propagated.tractor_requirements!}
        numDecks={props.numDecksEffective}
        onChange={(req) => props.setTractorRequirements(req)}
      />
      <div>
        <label>
          Should reveal kitty at end of game:{" "}
          <select
            value={
              props.state.propagated.should_reveal_kitty_at_end_of_game
                ? "show"
                : "hide"
            }
            onChange={props.setShouldRevealKittyAtEndOfGame}
          >
            <option value="hide">
              Do not reveal contents of the kitty at the end of the game in chat
            </option>
            <option value="show">
              Reveal contents of the kitty at the end of the game in chat
            </option>
          </select>
        </label>
      </div>
      <div>
        <label>
          Show player which defeats throw:{" "}
          <select
            value={
              props.state.propagated.hide_throw_halting_player ? "hide" : "show"
            }
            onChange={props.setHideThrowHaltingPlayer}
          >
            <option value="hide">
              Hide the player who defeats a potential throw
            </option>
            <option value="show">
              Show the player who defeats a potential throw
            </option>
          </select>
        </label>
      </div>
    </>
  );
  return (
    <div>
      <label>
        More game settings:{" "}
        <button
          className="normal"
          onClick={(evt) => {
            evt.preventDefault();
            setModalOpen(true);
          }}
        >
          Open
        </button>
        <ReactModal
          isOpen={modalOpen}
          onRequestClose={() => setModalOpen(false)}
          shouldCloseOnOverlayClick
          shouldCloseOnEsc
          style={{ content: contentStyle }}
        >
          {s}
        </ReactModal>
      </label>
    </div>
  );
};

interface IProps {
  state: InitializePhase;
  name: string;
}

const Initialize = (props: IProps): JSX.Element => {
  const { send } = React.useContext(WebsocketContext);
  const [showPicker, setShowPicker] = React.useState<boolean>(false);
  const setGameMode = (evt: React.ChangeEvent<HTMLSelectElement>): void => {
    evt.preventDefault();
    if (evt.target.value === "Tractor") {
      send({ Action: { SetGameMode: "Tractor" } });
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
  };

  const setNumFriends = (evt: React.ChangeEvent<HTMLSelectElement>): void => {
    evt.preventDefault();
    if (evt.target.value === "") {
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
  };

  const onSelectString =
    (action: string): ((evt: React.ChangeEvent<HTMLSelectElement>) => void) =>
    (evt: React.ChangeEvent<HTMLSelectElement>): void => {
      evt.preventDefault();
      if (evt.target.value !== "") {
        send({ Action: { [action]: evt.target.value } });
      }
    };

  const onSelectStringDefault =
    (
      action: string,
      defaultValue: null | string,
    ): ((evt: React.ChangeEvent<HTMLSelectElement>) => void) =>
    (evt: React.ChangeEvent<HTMLSelectElement>): void => {
      evt.preventDefault();
      if (evt.target.value !== "") {
        send({ Action: { [action]: evt.target.value } });
      } else {
        send({ Action: { [action]: defaultValue } });
      }
    };

  const setFriendSelectionPolicy = onSelectString("SetFriendSelectionPolicy");
  const setMultipleJoinPolicy = onSelectString("SetMultipleJoinPolicy");
  const setFirstLandlordSelectionPolicy = onSelectString(
    "SetFirstLandlordSelectionPolicy",
  );
  const setBidPolicy = onSelectString("SetBidPolicy");
  const setBidReinforcementPolicy = onSelectString("SetBidReinforcementPolicy");
  const setJokerBidPolicy = onSelectString("SetJokerBidPolicy");
  const setKittyTheftPolicy = onSelectString("SetKittyTheftPolicy");
  const setKittyBidPolicy = onSelectString("SetKittyBidPolicy");
  const setTrickDrawPolicy = onSelectString("SetTrickDrawPolicy");
  const setThrowEvaluationPolicy = onSelectString("SetThrowEvaluationPolicy");
  const setPlayTakebackPolicy = onSelectString("SetPlayTakebackPolicy");
  const setGameShadowingPolicy = onSelectString("SetGameShadowingPolicy");
  const setGameStartPolicy = onSelectString("SetGameStartPolicy");
  const setBidTakebackPolicy = onSelectString("SetBidTakebackPolicy");
  const setGameVisibility = onSelectString("SetGameVisibility");

  const setShouldRevealKittyAtEndOfGame = (
    evt: React.ChangeEvent<HTMLSelectElement>,
  ): void => {
    evt.preventDefault();
    if (evt.target.value !== "") {
      send({
        Action: {
          SetShouldRevealKittyAtEndOfGame: evt.target.value === "show",
        },
      });
    }
  };
  const setHideThrowHaltingPlayer = (
    evt: React.ChangeEvent<HTMLSelectElement>,
  ): void => {
    evt.preventDefault();
    if (evt.target.value !== "") {
      send({
        Action: {
          SetHideThrowHaltingPlayer: evt.target.value === "hide",
        },
      });
    }
  };

  const setKittyPenalty = onSelectStringDefault("SetKittyPenalty", null);
  const setAdvancementPolicy = onSelectStringDefault(
    "SetAdvancementPolicy",
    "Unrestricted",
  );
  const setMaxRank = onSelectStringDefault("SetMaxRank", "NT");
  const setThrowPenalty = onSelectStringDefault("SetThrowPenalty", null);

  const setHideLandlordsPoints = (
    evt: React.ChangeEvent<HTMLSelectElement>,
  ): void => {
    evt.preventDefault();
    send({ Action: { SetHideLandlordsPoints: evt.target.value === "hide" } });
  };

  const setHidePlayedCards = (
    evt: React.ChangeEvent<HTMLSelectElement>,
  ): void => {
    evt.preventDefault();
    send({ Action: { SetHidePlayedCards: evt.target.value === "hide" } });
  };

  const startGame = (evt: React.SyntheticEvent): void => {
    evt.preventDefault();
    send({ Action: "StartGame" });
  };

  const setEmoji = (emoji: string): void => {
    send({
      Action: {
        SetLandlordEmoji: emoji,
      },
    });
  };

  const modeAsString =
    props.state.propagated.game_mode === "Tractor"
      ? "Tractor"
      : "FindingFriends";
  const numFriends =
    props.state.propagated.game_mode === "Tractor" ||
    props.state.propagated.game_mode.FindingFriends.num_friends === null
      ? ""
      : props.state.propagated.game_mode.FindingFriends.num_friends;
  const decksEffective =
    props.state.propagated.num_decks !== undefined &&
    props.state.propagated.num_decks !== null &&
    props.state.propagated.num_decks > 0
      ? props.state.propagated.num_decks
      : Math.max(Math.floor(props.state.propagated.players.length / 2), 1);
  const decks = [...(props.state.propagated.special_decks || [])];
  while (decks.length < decksEffective) {
    decks.push({
      exclude_big_joker: false,
      exclude_small_joker: false,
      min: "2",
    });
  }
  decks.length = decksEffective;

  let currentPlayer = props.state.propagated.players.find(
    (p: Player) => p.name === props.name,
  );
  if (currentPlayer === undefined) {
    currentPlayer = props.state.propagated.observers.find(
      (p) => p.name === props.name,
    );
  }
  if (currentPlayer === undefined) {
    currentPlayer = {
      id: -1,
      name: props.name,
      level: "",
      metalevel: 0,
    };
  }

  const landlordIndex = props.state.propagated.players.findIndex(
    (p: Player) => p.id === props.state.propagated.landlord,
  );
  const saveGameSettings = (evt: React.SyntheticEvent): void => {
    evt.preventDefault();
    localStorage.setItem(
      "gameSettingsInLocalStorage",
      JSON.stringify(props.state.propagated),
    );
  };

  const setGameSettings = (gameSettings: PropagatedState): void => {
    if (gameSettings !== null) {
      let kittySizeSet = false;
      let kittySize = null;
      for (const [key, value] of Object.entries(gameSettings)) {
        switch (key) {
          case "game_mode":
            send({
              Action: {
                SetGameMode: value,
              },
            });
            break;
          case "num_decks":
            send({
              Action: {
                SetNumDecks: value,
              },
            });
            if (kittySizeSet) {
              // reset the size again, as setting deck num resets kitty_size to default
              send({
                Action: {
                  SetKittySize: kittySize,
                },
              });
            }
            break;
          case "special_decks":
            send({
              Action: {
                SetSpecialDecks: value,
              },
            });
            break;
          case "kitty_size":
            send({
              Action: {
                SetKittySize: value,
              },
            });
            kittySizeSet = true;
            kittySize = value;
            break;
          case "friend_selection_policy":
            send({
              Action: {
                SetFriendSelectionPolicy: value,
              },
            });
            break;
          case "multiple_join_policy":
            send({
              Action: {
                SetMultipleJoinPolicy: value,
              },
            });
            break;
          case "first_landlord_selection_policy":
            send({
              Action: {
                SetFirstLandlordSelectionPolicy: value,
              },
            });
            break;
          case "hide_landlord_points":
            send({
              Action: {
                SetHideLandlordsPoints: value,
              },
            });
            break;
          case "hide_played_cards":
            send({ Action: { SetHidePlayedCards: value } });
            break;
          case "advancement_policy":
            send({
              Action: {
                SetAdvancementPolicy: value,
              },
            });
            break;
          case "max_rank":
            send({
              Action: {
                SetMaxRank: value,
              },
            });
            break;
          case "kitty_bid_policy":
            send({
              Action: {
                SetKittyBidPolicy: value,
              },
            });
            break;
          case "kitty_penalty":
            send({
              Action: {
                SetKittyPenalty: value,
              },
            });
            break;
          case "kitty_theft_policy":
            send({
              Action: {
                SetKittyTheftPolicy: value,
              },
            });
            break;
          case "throw_penalty":
            send({
              Action: {
                SetThrowPenalty: value,
              },
            });
            break;
          case "trick_draw_policy":
            send({
              Action: {
                SetTrickDrawPolicy: value,
              },
            });
            break;
          case "throw_evaluation_policy":
            send({
              Action: {
                SetThrowEvaluationPolicy: value,
              },
            });
            break;
          case "landlord_emoji":
            send({
              Action: {
                SetLandlordEmoji: value,
              },
            });
            break;
          case "bid_policy":
            send({
              Action: {
                SetBidPolicy: value,
              },
            });
            break;
          case "bid_reinforcement_policy":
            send({
              Action: {
                SetBidReinforcementPolicy: value,
              },
            });
            break;
          case "joker_bid_policy":
            send({
              Action: {
                SetJokerBidPolicy: value,
              },
            });
            break;
          case "should_reveal_kitty_at_end_of_game":
            send({
              Action: {
                SetShouldRevealKittyAtEndOfGame: value,
              },
            });
            break;
          case "hide_throw_halting_player":
            send({ Action: { SetHideThrowHaltingPlayer: value } });
            break;
          case "game_scoring_parameters":
            send({
              Action: {
                SetGameScoringParameters: value,
              },
            });
            break;
          case "play_takeback_policy":
            send({
              Action: {
                SetPlayTakebackPolicy: value,
              },
            });
            break;
          case "bid_takeback_policy":
            send({
              Action: {
                SetBidTakebackPolicy: value,
              },
            });
            break;
          case "game_shadowing_policy":
            send({
              Action: {
                SetGameShadowingPolicy: value,
              },
            });
            break;
          case "game_start_policy":
            send({
              Action: {
                SetGameStartPolicy: value,
              },
            });
            break;
          case "tractor_requirements":
            send({
              Action: {
                SetTractorRequirements: value,
              },
            });
            break;
          case "game_visibility":
            send({
              Action: {
                SetGameVisibility: value,
              },
            });
            break;
        }
      }
    }
  };

  const loadGameSettings = (evt: React.SyntheticEvent): void => {
    evt.preventDefault();
    const settings = localStorage.getItem("gameSettingsInLocalStorage");
    if (settings !== null) {
      let gameSettings: PropagatedState;
      try {
        gameSettings = JSON.parse(settings);

        const fetchAsync = async (): Promise<void> => {
          const fetchResult = await fetch("default_settings.json");
          const fetchJSON = await fetchResult.json();
          const combined = { ...fetchJSON, ...gameSettings };
          if (
            combined.bonus_level_policy !== undefined &&
            combined.game_scoring_parameters !== undefined &&
            combined.bonus_level_policy !==
              combined.game_scoring_parameters.bonus_level_policy
          ) {
            combined.game_scoring_parameters.bonus_level_policy =
              combined.bonus_level_policy;
          }
          setGameSettings(combined);
        };

        fetchAsync().catch((e) => {
          console.error(e);
          localStorage.setItem(
            "gameSettingsInLocalStorage",
            JSON.stringify(props.state.propagated),
          );
        });
      } catch {
        localStorage.setItem(
          "gameSettingsInLocalStorage",
          JSON.stringify(props.state.propagated),
        );
      }
    }
  };

  const resetGameSettings = (evt: React.SyntheticEvent): void => {
    evt.preventDefault();

    const fetchAsync = async (): Promise<void> => {
      const fetchResult = await fetch("default_settings.json");
      const fetchJSON = await fetchResult.json();
      setGameSettings(fetchJSON);
    };

    fetchAsync().catch((e) => console.error(e));
  };

  return (
    <div>
      <Header
        gameMode={props.state.propagated.game_mode}
        chatLink={props.state.propagated.chat_link}
      />
      <Players
        players={props.state.propagated.players}
        observers={props.state.propagated.observers}
        landlord={props.state.propagated.landlord}
        next={null}
        movable={true}
        name={props.name}
      />
      <p>
        Send link to other players to allow them to join the game:{" "}
        <a href={window.location.href} target="_blank" rel="noreferrer">
          <code>{window.location.href}</code>
        </a>
      </p>
      {props.state.propagated.players.length >= 4 ? (
        <>
          <button
            className="big"
            disabled={
              props.state.propagated.game_start_policy ===
                "AllowLandlordOnly" &&
              landlordIndex !== -1 &&
              props.state.propagated.players[landlordIndex].name !== props.name
            }
            onClick={startGame}
          >
            Start game
          </button>
          <ReadyCheck />
        </>
      ) : (
        <h2>Waiting for players...</h2>
      )}
      <RandomizePlayersButton players={props.state.propagated.players}>
        Randomize player order
      </RandomizePlayersButton>
      <Kicker
        players={props.state.propagated.players}
        onKick={(playerId: number) => send({ Kick: playerId })}
      />
      <div className="game-settings">
        <h3>Game settings</h3>
        <div>
          <label>
            Game mode:{" "}
            <select value={modeAsString} onChange={setGameMode}>
              <option value="Tractor">升级 / Tractor</option>
              <option value="FindingFriends">找朋友 / Finding Friends</option>
            </select>
          </label>
        </div>
        <div>
          {props.state.propagated.game_mode !== "Tractor" ? (
            <label>
              Number of friends:{" "}
              <select value={numFriends} onChange={setNumFriends}>
                <option value="">default</option>
                {ArrayUtils.range(
                  Math.max(
                    Math.floor(props.state.propagated.players.length / 2) - 1,
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
          numPlayers={props.state.propagated.players.length}
          numDecks={props.state.propagated.num_decks}
          onChange={(newNumDecks: number | null) =>
            send({ Action: { SetNumDecks: newNumDecks } })
          }
        />
        <DeckSettings
          decks={decks}
          setSpecialDecks={(d) => send({ Action: { SetSpecialDecks: d } })}
        />
        <KittySizeSelector
          numPlayers={props.state.propagated.players.length}
          decks={decks}
          kittySize={props.state.propagated.kitty_size}
          onChange={(newKittySize: number | null) =>
            send({ Action: { SetKittySize: newKittySize } })
          }
        />
        <div>
          <label>
            Bids after cards are exchanged from the bottom:{" "}
            <select
              value={props.state.propagated.kitty_theft_policy}
              onChange={setKittyTheftPolicy}
            >
              <option value="AllowKittyTheft">Allowed (炒地皮)</option>
              <option value="NoKittyTheft">Not allowed</option>
            </select>
          </label>
        </div>
        <div>
          <label>
            Card protection policy:{" "}
            <select
              value={props.state.propagated.trick_draw_policy}
              onChange={setTrickDrawPolicy}
            >
              <option value="NoProtections">No protections</option>
              <option value="LongerTuplesProtected">
                Longer tuple (triple) is protected from shorter (pair)
              </option>
              <option value="OnlyDrawTractorOnTractor">
                Only tractors can draw tractors
              </option>
              <option value="LongerTuplesProtectedAndOnlyDrawTractorOnTractor">
                Longer tuples are protected from shorter, and only tractors can
                draw tractors
              </option>
              <option value="NoFormatBasedDraw">
                No format-based requirements (pairs do not draw pairs)
              </option>
            </select>
          </label>
        </div>
        <div>
          <label>
            Multi-throw evaluation policy:{" "}
            <select
              value={props.state.propagated.throw_evaluation_policy}
              onChange={setThrowEvaluationPolicy}
            >
              <option value="All">
                Subsequent throw must beat all cards to win
              </option>
              <option value="Highest">
                Subsequent throw must beat highest card to win
              </option>
              <option value="TrickUnitLength">
                Subsequent throw must beat largest component to win
              </option>
            </select>
          </label>
        </div>
        <ScoringSettings state={props.state} decks={decks} />
        <UncommonSettings
          state={props.state}
          numDecksEffective={decksEffective}
          setBidPolicy={setBidPolicy}
          setBidReinforcementPolicy={setBidReinforcementPolicy}
          setJokerBidPolicy={setJokerBidPolicy}
          setShouldRevealKittyAtEndOfGame={setShouldRevealKittyAtEndOfGame}
          setHideThrowHaltingPlayer={setHideThrowHaltingPlayer}
          setFirstLandlordSelectionPolicy={setFirstLandlordSelectionPolicy}
          setGameStartPolicy={setGameStartPolicy}
          setGameShadowingPolicy={setGameShadowingPolicy}
          setKittyBidPolicy={setKittyBidPolicy}
          setTractorRequirements={(requirements) =>
            send({ Action: { SetTractorRequirements: requirements } })
          }
        />
        <DifficultySettings
          state={props.state}
          setFriendSelectionPolicy={setFriendSelectionPolicy}
          setMultipleJoinPolicy={setMultipleJoinPolicy}
          setAdvancementPolicy={setAdvancementPolicy}
          setMaxRank={setMaxRank}
          setHideLandlordsPoints={setHideLandlordsPoints}
          setHidePlayedCards={setHidePlayedCards}
          setKittyPenalty={setKittyPenalty}
          setThrowPenalty={setThrowPenalty}
          setPlayTakebackPolicy={setPlayTakebackPolicy}
          setBidTakebackPolicy={setBidTakebackPolicy}
        />
        <div>
          <label>
            Game Visibility{" "}
            <select
              value={props.state.propagated.game_visibility}
              onChange={setGameVisibility}
            >
              <option value={"Unlisted"}>Unlisted</option>
              <option value={"Public"}>Public</option>
            </select>
          </label>
        </div>
        <h3>Continuation settings</h3>
        <LandlordSelector
          players={props.state.propagated.players}
          landlordId={props.state.propagated.landlord}
          onChange={(newLandlord: number | null) =>
            send({ Action: { SetLandlord: newLandlord } })
          }
        />
        <RankSelector
          rank={currentPlayer.level}
          metaRank={currentPlayer.metalevel}
          onChangeRank={(newRank: string) =>
            send({ Action: { SetRank: newRank } })
          }
          onChangeMetaRank={(newMetaRank: number) =>
            send({ Action: { SetMetaRank: newMetaRank } })
          }
        />
        <h3>Misc settings</h3>
        <div>
          <label>
            Landlord label:{" "}
            {props.state.propagated.landlord_emoji !== null &&
            props.state.propagated.landlord_emoji !== undefined &&
            props.state.propagated.landlord_emoji !== ""
              ? props.state.propagated.landlord_emoji
              : "当庄"}{" "}
            <button
              className="normal"
              onClick={() => {
                setShowPicker(!showPicker);
              }}
            >
              {showPicker ? "Hide" : "Pick"}
            </button>
            <button
              className="normal"
              disabled={props.state.propagated.landlord_emoji == null}
              onClick={() => {
                send({ Action: { SetLandlordEmoji: null } });
              }}
            >
              Default
            </button>
            {showPicker ? (
              <React.Suspense fallback={"..."}>
                <Picker
                  onEmojiClick={(ecd) => setEmoji(ecd.emoji)}
                  emojiStyle={EmojiStyle.NATIVE}
                />
              </React.Suspense>
            ) : null}
          </label>
        </div>
        <div>
          <label>
            Setting Management:
            <button
              className="normal"
              data-tip
              data-for="saveTip"
              onClick={saveGameSettings}
            >
              Save
            </button>
            <ReactTooltip id="saveTip" place="top" effect="solid">
              Save game settings
            </ReactTooltip>
            <button
              className="normal"
              data-tip
              data-for="loadTip"
              onClick={loadGameSettings}
            >
              Load
            </button>
            <ReactTooltip id="loadTip" place="top" effect="solid">
              Load saved game settings
            </ReactTooltip>
            <button
              className="normal"
              data-tip
              data-for="resetTip"
              onClick={resetGameSettings}
            >
              Reset
            </button>
            <ReactTooltip id="resetTip" place="top" effect="solid">
              Reset game settings to defaults
            </ReactTooltip>
          </label>
        </div>
      </div>
    </div>
  );
};

export default Initialize;
