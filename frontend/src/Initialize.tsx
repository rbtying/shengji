import * as React from 'react';
import ReactTooltip from 'react-tooltip';
import LandlordSelector from './LandlordSelector';
import NumDecksSelector from './NumDecksSelector';
import RankSelector from './RankSelector';
import Kicker from './Kicker';
import ArrayUtils from './util/array';
import { IInitializePhase, IPropagatedState } from './types';
import { WebsocketContext } from './WebsocketProvider';
import { IPlayer } from './types';
import Header from './Header';
import Players from './Players';

type Props = {
  state: IInitializePhase;
  cards: string[];
  name: string;
};

const Initialize = (props: Props) => {
  const { send } = React.useContext(WebsocketContext);
  const setGameMode = (evt: any) => {
    evt.preventDefault();
    if (evt.target.value === 'Tractor') {
      setGameModeValue('Tractor');
      send({ Action: { SetGameMode: 'Tractor' } });
    } else {
      setGameModeValue('FindingFriends');
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

  const setNumFriends = (evt: any) => {
    evt.preventDefault();
    if (evt.target.value === '') {
      setNumFriendsValue('');
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
      setNumFriendsValue(num);
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

  const setKittySize = (evt: any) => {
    evt.preventDefault();
    if (evt.target.value !== '') {
      const size = parseInt(evt.target.value, 10);
      setKittySizeValue(size);
      send({
        Action: {
          SetKittySize: size,
        },
      });
    } else {
      setKittySizeValue(null);
      send({
        Action: {
          SetKittySize: null,
        },
      });
    }
  };

  const setFriendSelectionPolicy = (evt: any) => {
    evt.preventDefault();
    if (evt.target.value !== '') {
      setFriendSelectionPolicyValue(evt.target.value);
      send({
        Action: {
          SetFriendSelectionPolicy: evt.target.value,
        },
      });
    }
  };

  const setKittyPenalty = (evt: any) => {
    evt.preventDefault();
    if (evt.target.value !== '') {
      setKittyPenaltyValue(evt.target.value);
      send({
        Action: {
          SetKittyPenalty: evt.target.value,
        },
      });
    } else {
      setKittyPenaltyValue(null);
      send({
        Action: {
          SetKittyPenalty: null,
        },
      });
    }
  };

  const setKittyBidPolicy = (evt: any) => {
    evt.preventDefault();
    if (evt.target.value !== '') {
      setKittyBidPolicyValue(evt.target.value);
      send({
        Action: {
          SetKittyBidPolicy: evt.target.value,
        },
      });
    }
  };

  const setTrickDrawPolicy = (evt: any) => {
    evt.preventDefault();
    if (evt.target.value !== '') {
      SetTrickDrawPolicyValue(evt.target.value);
      send({
        Action: {
          SetTrickDrawPolicy: evt.target.value,
        },
      });
    }
  };

  const setThrowEvaluationPolicy = (evt: any) => {
    evt.preventDefault();
    if (evt.target.value !== '') {
      setThrowEvaluationPolicyValue(evt.target.value);
      send({
        Action: {
          SetThrowEvaluationPolicy: evt.target.value,
        },
      });
    }
  };

  const setAdvancementPolicy = (evt: any) => {
    evt.preventDefault();
    if (evt.target.value !== '') {
      setAdvancementPolicyValue(evt.target.value);
      send({
        Action: {
          SetAdvancementPolicy: evt.target.value,
        },
      });
    } else {
      setAdvancementPolicyValue('Unrestricted');
      send({
        Action: {
          SetAdvancementPolicy: 'Unrestricted',
        },
      });
    }
  };

  const setThrowPenalty = (evt: any) => {
    evt.preventDefault();
    if (evt.target.value !== '') {
      setThrowPenaltyValue(evt.target.value);
      send({
        Action: {
          SetThrowPenalty: evt.target.value,
        },
      });
    } else {
      setThrowPenaltyValue(null);
      send({
        Action: {
          SetThrowPenalty: null,
        },
      });
    }
  };

  const setHideLandlordsPoints = (evt: any) => {
    evt.preventDefault();
    setHideLandlordPointsValue(evt.target.value === 'hide');
    send({ Action: { SetHideLandlordsPoints: evt.target.value === 'hide' } });
  };

  const setHidePlayedCards = (evt: any) => {
    evt.preventDefault();
    setHidePlayedCardsValue(evt.target.value === 'hide');
    send({ Action: { SetHidePlayedCards: evt.target.value === 'hide' } });
  };

  const startGame = (evt: any) => {
    evt.preventDefault();
    send({ Action: 'StartGame' });
  };

  const modeAsString =
    props.state.propagated.game_mode === 'Tractor'
      ? 'Tractor'
      : 'FindingFriends';
  const numFriends =
    props.state.propagated.game_mode === 'Tractor' ||
      props.state.propagated.game_mode.FindingFriends.num_friends === null
      ? ''
      : props.state.propagated.game_mode.FindingFriends.num_friends;
  const decksEffective =
    props.state.propagated.num_decks ||
    Math.floor(props.state.propagated.players.length / 2);
  let kittyOffset =
    (decksEffective * 54) % props.state.propagated.players.length;
  if (kittyOffset === 0) {
    kittyOffset += props.state.propagated.players.length;
  }

  let currentPlayer = props.state.propagated.players.find(
    (p: IPlayer) => p.name === props.name,
  );
  if (!currentPlayer) {
    currentPlayer = props.state.propagated.observers.find(
      (p) => p.name === props.name,
    );
  }

  const [gameModeValue, setGameModeValue] = React.useState(modeAsString);
  const [numDecksValue, setNumDecksValue] = React.useState(props.state.propagated.num_decks);
  const [numFriendsValue, setNumFriendsValue] = React.useState(numFriends);
  const [kittySizeValue, setKittySizeValue] = React.useState(props.state.propagated.kitty_size);
  const [friendSelectionPolicyValue, setFriendSelectionPolicyValue] = React.useState(props.state.propagated.friend_selection_policy);
  const [hideLandlordPointsValue, setHideLandlordPointsValue] = React.useState(props.state.propagated.hide_landlord_points);
  const [hidePlayedCardsValue, setHidePlayedCardsValue] = React.useState(props.state.propagated.hide_played_cards);
  const [advancementPolicyValue, setAdvancementPolicyValue] = React.useState(props.state.propagated.advancement_policy);
  const [kittyBidPolicyValue, setKittyBidPolicyValue] = React.useState(props.state.propagated.kitty_bid_policy);
  const [kittyPenaltyValue, setKittyPenaltyValue] = React.useState(props.state.propagated.kitty_penalty);
  const [throwPenaltyValue, setThrowPenaltyValue] = React.useState(props.state.propagated.throw_penalty);
  const [trickDrawPolicyValue, SetTrickDrawPolicyValue] = React.useState(props.state.propagated.trick_draw_policy);
  const [throwEvaluationPolicyValue, setThrowEvaluationPolicyValue] = React.useState(props.state.propagated.throw_evaluation_policy);

  const saveGameSettings = (evt: any) => {
    evt.preventDefault();
    localStorage.setItem('gameSettingsInLocalStorage', JSON.stringify(props.state.propagated));
  };

  const loadGameSettings = (evt: any) => {
    evt.preventDefault();
    const settings = localStorage.getItem('gameSettingsInLocalStorage');
    if (settings !== null) {
      const gameSettings: IPropagatedState = JSON.parse(settings);
      for (const [key, value] of Object.entries(gameSettings)) {
        switch (key) {
          case 'game_mode':
            if (value === 'Tractor') {
              setGameModeValue('Tractor');
              send({
                Action: {
                  SetGameMode: 'Tractor'
                }
              });
            } else {
              setGameModeValue('FindingFriends');
              send({
                Action: {
                  SetGameMode: {
                    FindingFriends: {
                      num_friends: value.num_friends,
                    },
                  },
                },
              });
            }
            break;
          case 'num_decks':
            setNumDecksValue(value);
            send({
              Action: {
                SetNumDecks: value
              },
            })
            break;
          case 'kitty_size':
            setKittySizeValue(value);
            send({
              Action: {
                SetKittySize: value,
              },
            });
            break;
          case 'friend_selection_policy':
            setFriendSelectionPolicyValue(value);
            send({
              Action: {
                SetFriendSelectionPolicy: value,
              },
            });
            break;
          case 'hide_landlord_points':
            setHideLandlordPointsValue(value);
            send({
              Action: {
                SetHideLandlordsPoints: value
              },
            });
            break;
          case 'hide_played_cards':
            setHidePlayedCardsValue(value);
            send({ Action: { SetHidePlayedCards: value } });
            break;
          case 'advancement_policy':
            setAdvancementPolicyValue(value);
            send({
              Action: {
                SetAdvancementPolicy: value,
              },
            });
            break;
          case 'kitty_bid_policy':
            setKittyBidPolicyValue(value);
            send({
              Action: {
                SetKittyBidPolicy: value,
              },
            });
            break;
          case 'kitty_penalty':
            setKittyPenaltyValue(value);
            send({
              Action: {
                SetKittyPenalty: value,
              },
            });
            break;
          case 'throw_penalty':
            setThrowPenaltyValue(value);
            send({
              Action: {
                SetThrowPenalty: value,
              },
            });
            break;
          case 'trick_draw_policy':
            SetTrickDrawPolicyValue(value);
            send({
              Action: {
                SetTrickDrawPolicy: value,
              },
            });
            break;
          case 'throw_evaluation_policy':
            setThrowEvaluationPolicyValue(value);
            send({
              Action: {
                SetThrowEvaluationPolicy: value,
              },
            });
            break;
        }
      }
    }
  }

  const resetGameSettings = (evt: any) => {
    evt.preventDefault();

    setGameModeValue('Tractor');
    send({
      Action: {
        SetGameMode: 'Tractor'
      }
    });

    setNumDecksValue(null);
    send({
      Action: {
        SetNumDecks: null
      },
    })

    setKittySizeValue(null);
    send({
      Action: {
        SetKittySize: null,
      },
    });

    setFriendSelectionPolicyValue('Unrestricted');
    send({
      Action: {
        SetFriendSelectionPolicy: 'Unrestricted',
      },
    });

    setHideLandlordPointsValue(false);
    send({
      Action: {
        SetHideLandlordsPoints: false
      },
    });

    setHidePlayedCardsValue(false);
    send({ Action: { SetHidePlayedCards: false } });

    setAdvancementPolicyValue('Unrestricted');
    send({
      Action: {
        SetAdvancementPolicy: 'Unrestricted',
      },
    });

    setKittyBidPolicyValue('FirstCard');
    send({
      Action: {
        SetKittyBidPolicy: 'FirstCard',
      },
    });

    setKittyPenaltyValue('Times');
    send({
      Action: {
        SetKittyPenalty: 'Times',
      },
    });

    setThrowPenaltyValue('None');
    send({
      Action: {
        SetThrowPenalty: 'None',
      },
    });

    SetTrickDrawPolicyValue('NoProtections');
    send({
      Action: {
        SetTrickDrawPolicy: 'NoProtections',
      },
    });

    setThrowEvaluationPolicyValue('All');
    send({
      Action: {
        SetThrowEvaluationPolicy: 'All',
      },
    });

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
        Send link to other players to allow them to join the game:{' '}
        <a href={window.location.href} target="_blank">
          <code>{window.location.href}</code>
        </a>
      </p>
      {props.state.propagated.players.length >= 4 ? (
        <button onClick={startGame}>Start game</button>
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
            Game mode:{' '}
            <select value={gameModeValue === null ? '' : gameModeValue} onChange={setGameMode}>
              <option value="Tractor">升级 / Tractor</option>
              <option value="FindingFriends">找朋友 / Finding Friends</option>
            </select>
          </label>
        </div>
        <div>
          {props.state.propagated.game_mode !== 'Tractor' ? (
            <label>
              Number of friends:{' '}
              <select value={numFriendsValue} onChange={setNumFriends}>
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
          numDecks={numDecksValue}
          onChange={(newNumDecks: number | null) => {
            setNumDecksValue(newNumDecks);
            send({ Action: { SetNumDecks: newNumDecks } });
          }
          }
        />
        <div>
          <label>
            Number of cards in the bottom:{' '}
            <select
              value={kittySizeValue || ''}
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
            Friend Selection Restriction:{' '}
            <select
              value={friendSelectionPolicyValue}
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
            Point visibility:{' '}
            <select
              value={
                hideLandlordPointsValue ? 'hide' : 'show'
              }
              onChange={setHideLandlordsPoints}
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
              value={hidePlayedCardsValue ? 'hide' : 'show'}
              onChange={setHidePlayedCards}
            >
              <option value="show">Show played cards in chat</option>
              <option value="hide">Hide played cards in chat</option>
            </select>
          </label>
        </div>
        <div>
          <label>
            Rank advancement policy:{' '}
            <select
              value={advancementPolicyValue}
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
            Trump policy for cards revealed from the bottom:{' '}
            <select
              value={kittyBidPolicyValue}
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
            Penalty for points left in the bottom:{' '}
            <select
              value={kittyPenaltyValue}
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
            Penalty for incorrect throws:{' '}
            <select
              value={throwPenaltyValue}
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
            Card protection policy:{' '}
            <select
              value={trickDrawPolicyValue}
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
            Multi-throw evaluation policy:{' '}
            <select
              value={throwEvaluationPolicyValue}
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
        <div></div>

        <button data-tip data-for="saveTip" onClick={saveGameSettings} >Save</button>
        <ReactTooltip id="saveTip" place="top" effect="solid">
          Save game settings
        </ReactTooltip>
        <button data-tip data-for="loadTip" onClick={loadGameSettings} >Load</button>
        <ReactTooltip id="loadTip" place="top" effect="solid">
          Load saved game settings
        </ReactTooltip>
        <button data-tip data-for="resetTip" onClick={resetGameSettings} >Reset</button>
        <ReactTooltip id="resetTip" place="top" effect="solid">
          Reset game settings to defaults
        </ReactTooltip>
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
          onChangeRank={(newRank: string) => send({ Action: { SetRank: newRank } })}
        />
      </div>
    </div>
  );
};

export default Initialize;
