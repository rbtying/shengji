import * as React from "react";
import ReactTooltip from "react-tooltip";
import { IEmojiData } from "emoji-picker-react";
import LandlordSelector from "./LandlordSelector";
import NumDecksSelector from "./NumDecksSelector";
import RankSelector from "./RankSelector";
import Kicker from "./Kicker";
import ArrayUtils from "./util/array";
import { IInitializePhase, IPlayer, IPropagatedState } from "./types";
import { WebsocketContext } from "./WebsocketProvider";

import Header from "./Header";
import Players from "./Players";

const Picker = React.lazy(async () => await import("emoji-picker-react"));

interface IProps {
  state: IInitializePhase;
  cards: string[];
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

  const setKittySize = (evt: React.ChangeEvent<HTMLSelectElement>): void => {
    evt.preventDefault();
    if (evt.target.value !== "") {
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
  };

  const setFriendSelectionPolicy = (
    evt: React.ChangeEvent<HTMLSelectElement>
  ): void => {
    evt.preventDefault();
    if (evt.target.value !== "") {
      send({
        Action: {
          SetFriendSelectionPolicy: evt.target.value,
        },
      });
    }
  };

  const setFirstLandlordSelectionPolicy = (
    evt: React.ChangeEvent<HTMLSelectElement>
  ): void => {
    evt.preventDefault();
    if (evt.target.value !== "") {
      send({
        Action: {
          SetFirstLandlordSelectionPolicy: evt.target.value,
        },
      });
    }
  };

  const setBidPolicy = (evt: React.ChangeEvent<HTMLSelectElement>): void => {
    evt.preventDefault();
    if (evt.target.value !== "") {
      send({
        Action: {
          SetBidPolicy: evt.target.value,
        },
      });
    }
  };

  const setKittyTheftPolicy = (
    evt: React.ChangeEvent<HTMLSelectElement>
  ): void => {
    evt.preventDefault();
    if (evt.target.value !== "") {
      send({
        Action: {
          SetKittyTheftPolicy: evt.target.value,
        },
      });
    }
  };

  const setKittyPenalty = (evt: React.ChangeEvent<HTMLSelectElement>): void => {
    evt.preventDefault();
    if (evt.target.value !== "") {
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
  };

  const setKittyBidPolicy = (
    evt: React.ChangeEvent<HTMLSelectElement>
  ): void => {
    evt.preventDefault();
    if (evt.target.value !== "") {
      send({
        Action: {
          SetKittyBidPolicy: evt.target.value,
        },
      });
    }
  };

  const setTrickDrawPolicy = (
    evt: React.ChangeEvent<HTMLSelectElement>
  ): void => {
    evt.preventDefault();
    if (evt.target.value !== "") {
      send({
        Action: {
          SetTrickDrawPolicy: evt.target.value,
        },
      });
    }
  };

  const setThrowEvaluationPolicy = (
    evt: React.ChangeEvent<HTMLSelectElement>
  ): void => {
    evt.preventDefault();
    if (evt.target.value !== "") {
      send({
        Action: {
          SetThrowEvaluationPolicy: evt.target.value,
        },
      });
    }
  };

  const setPlayTakebackPolicy = (
    evt: React.ChangeEvent<HTMLSelectElement>
  ): void => {
    evt.preventDefault();
    if (evt.target.value !== "") {
      send({
        Action: {
          SetPlayTakebackPolicy: evt.target.value,
        },
      });
    }
  };

  const setGameShadowingPolicy = (
    evt: React.ChangeEvent<HTMLSelectElement>
  ): void => {
    evt.preventDefault();
    if (evt.target.value !== "") {
      send({
        Action: {
          SetGameShadowingPolicy: evt.target.value,
        },
      });
    }
  };

  const setGameStartPolicy = (
    evt: React.ChangeEvent<HTMLSelectElement>
  ): void => {
    evt.preventDefault();
    if (evt.target.value !== "") {
      send({
        Action: {
          SetGameStartPolicy: evt.target.value,
        },
      });
    }
  };

  const setAdvancementPolicy = (
    evt: React.ChangeEvent<HTMLSelectElement>
  ): void => {
    evt.preventDefault();
    if (evt.target.value !== "") {
      send({
        Action: {
          SetAdvancementPolicy: evt.target.value,
        },
      });
    } else {
      send({
        Action: {
          SetAdvancementPolicy: "Unrestricted",
        },
      });
    }
  };

  const setBonusLevelPolicy = (
    evt: React.ChangeEvent<HTMLSelectElement>
  ): void => {
    evt.preventDefault();
    if (evt.target.value !== "") {
      send({
        Action: {
          SetBonusLevelPolicy: evt.target.value,
        },
      });
    } else {
      send({
        Action: {
          SetBonusLevelPolicy: "NoBonusLevel",
        },
      });
    }
  };

  const setBidTakebackPolicy = (
    evt: React.ChangeEvent<HTMLSelectElement>
  ): void => {
    evt.preventDefault();
    if (evt.target.value !== "") {
      send({
        Action: {
          SetBidTakebackPolicy: evt.target.value,
        },
      });
    }
  };

  const setThrowPenalty = (evt: React.ChangeEvent<HTMLSelectElement>): void => {
    evt.preventDefault();
    if (evt.target.value !== "") {
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
  };

  const setHideLandlordsPoints = (
    evt: React.ChangeEvent<HTMLSelectElement>
  ): void => {
    evt.preventDefault();
    send({ Action: { SetHideLandlordsPoints: evt.target.value === "hide" } });
  };

  const setHidePlayedCards = (
    evt: React.ChangeEvent<HTMLSelectElement>
  ): void => {
    evt.preventDefault();
    send({ Action: { SetHidePlayedCards: evt.target.value === "hide" } });
  };

  const startGame = (evt: React.SyntheticEvent): void => {
    evt.preventDefault();
    send({ Action: "StartGame" });
  };

  const setEmoji = (evt: MouseEvent, emojiObject: IEmojiData | null): void => {
    evt.preventDefault();
    send({
      Action: {
        SetLandlordEmoji:
          emojiObject !== undefined && emojiObject !== null
            ? emojiObject.emoji
            : null,
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
      : Math.floor(props.state.propagated.players.length / 2);
  let kittyOffset =
    (decksEffective * 54) % props.state.propagated.players.length;
  if (kittyOffset === 0) {
    kittyOffset += props.state.propagated.players.length;
  }

  let currentPlayer = props.state.propagated.players.find(
    (p: IPlayer) => p.name === props.name
  );
  if (currentPlayer === undefined) {
    currentPlayer = props.state.propagated.observers.find(
      (p) => p.name === props.name
    );
  }

  const saveGameSettings = (evt: React.SyntheticEvent): void => {
    evt.preventDefault();
    localStorage.setItem(
      "gameSettingsInLocalStorage",
      JSON.stringify(props.state.propagated)
    );
  };

  const setGameSettings = (gameSettings: IPropagatedState): void => {
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
              // reset the size again, as setting deck numn resets kitty_size to default
              send({
                Action: {
                  SetKittySize: kittySize,
                },
              });
            }
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
          case "bonus_level_policy":
            send({
              Action: {
                SetBonusLevelPolicy: value,
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
        }
      }
    }
  };

  const loadGameSettings = (evt: React.SyntheticEvent): void => {
    evt.preventDefault();
    const settings = localStorage.getItem("gameSettingsInLocalStorage");
    if (settings !== null) {
      let gameSettings: IPropagatedState;
      try {
        gameSettings = JSON.parse(settings);
        setGameSettings(gameSettings);
      } catch (err) {
        localStorage.setItem(
          "gameSettingsInLocalStorage",
          JSON.stringify(props.state.propagated)
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
        <button
          disabled={
            props.state.propagated.game_start_policy === "AllowLandlordOnly" &&
            props.state.propagated.landlord != null &&
            props.state.propagated.players[props.state.propagated.landlord]
              .name !== props.name
          }
          onClick={startGame}
        >
          Start game
        </button>
      ) : (
        <h2>Waiting for players...</h2>
      )}
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
                    0
                  ),
                  (idx) => (
                    <option value={idx + 1} key={idx}>
                      {idx + 1}
                    </option>
                  )
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
        <div>
          <label>
            Number of cards in the bottom:{" "}
            <select
              value={
                props.state.propagated.kitty_size !== undefined &&
                props.state.propagated.kitty_size !== null
                  ? props.state.propagated.kitty_size
                  : ""
              }
              onChange={setKittySize}
            >
              <option value="">default</option>
              <option value={kittyOffset}>{kittyOffset} cards</option>
              <option
                value={kittyOffset + props.state.propagated.players.length}
              >
                {kittyOffset + props.state.propagated.players.length} cards
              </option>
              <option
                value={kittyOffset + 2 * props.state.propagated.players.length}
              >
                {kittyOffset + 2 * props.state.propagated.players.length} cards
              </option>
              <option
                value={kittyOffset + 3 * props.state.propagated.players.length}
              >
                {kittyOffset + 3 * props.state.propagated.players.length} cards
              </option>
            </select>
          </label>
        </div>
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
            </select>
          </label>
        </div>
        <div>
          <label>
            Trump policy for cards revealed from the bottom:{" "}
            <select
              value={props.state.propagated.kitty_bid_policy}
              onChange={setKittyBidPolicy}
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
            Bid Policy:{" "}
            <select
              value={props.state.propagated.bid_policy}
              onChange={setBidPolicy}
            >
              <option value="JokerOrGreaterLength">
                Joker bids to outbid non-joker bids with the same number of
                cards
              </option>
              <option value="GreaterLength">
                All bids must have more cards than the previous bids
              </option>
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
          onChangeRank={(newRank: string) =>
            send({ Action: { SetRank: newRank } })
          }
        />
        <h3>Difficulty and information settings</h3>
        <div>
          <label>
            Friend selection restriction:{" "}
            <select
              value={props.state.propagated.friend_selection_policy}
              onChange={setFriendSelectionPolicy}
            >
              <option value="Unrestricted">Non-trump cards</option>
              <option value="HighestCardNotAllowed">
                Non-trump cards, except the highest
              </option>
            </select>
          </label>
        </div>
        <div>
          <label>
            Rank advancement policy:{" "}
            <select
              value={props.state.propagated.advancement_policy}
              onChange={setAdvancementPolicy}
            >
              <option value="Unrestricted">Unrestricted</option>
              <option value="DefendPoints">
                Points (5, 10, K) must be defended
              </option>
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
              onChange={setHideLandlordsPoints}
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
              onChange={setHidePlayedCards}
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
              onChange={setKittyPenalty}
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
              onChange={setThrowPenalty}
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
              onChange={setPlayTakebackPolicy}
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
              onChange={setBidTakebackPolicy}
            >
              <option value="AllowBidTakeback">Allow bid takeback</option>
              <option value="NoBidTakeback">No bid takeback</option>
            </select>
          </label>
        </div>
        <div>
          <label>
            Small defending team bonus rank policy:{" "}
            <select
              value={props.state.propagated.bonus_level_policy}
              onChange={setBonusLevelPolicy}
            >
              <option value="BonusLevelForSmallerLandlordTeam">
                Bonus level for smaller defending team
              </option>
              <option value="NoBonusLevel">
                No bonus level for defending team
              </option>
            </select>
          </label>
        </div>
        <div>
          <label>
            Landlord selection from bid:{" "}
            <select
              value={props.state.propagated.first_landlord_selection_policy}
              onChange={setFirstLandlordSelectionPolicy}
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
                showPicker ? setShowPicker(false) : setShowPicker(true);
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
                <Picker onEmojiClick={setEmoji} />
              </React.Suspense>
            ) : null}
          </label>
        </div>
        <div>
          <label>
            Game shadowing policy:{" "}
            <select
              value={props.state.propagated.game_shadowing_policy}
              onChange={setGameShadowingPolicy}
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
              onChange={setGameStartPolicy}
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
