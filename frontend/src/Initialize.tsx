import * as React from 'react';
import LandlordSelector from './LandlordSelector';
import NumDecksSelector from './NumDecksSelector';
import RankSelector from './RankSelector';
import Kicker from './Kicker';
import ArrayUtils from './util/array';
import {IInitializePhase} from './types';
import {WebsocketContext} from './WebsocketProvider';
import {IPlayer} from './types';
import Header from './Header';
import Players from './Players';

type Props = {
  state: IInitializePhase;
  cards: string[];
  name: string;
};

const Initialize = (props: Props) => {
  const {send} = React.useContext(WebsocketContext);
  const setGameMode = (evt: any) => {
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
  };

  const setNumFriends = (evt: any) => {
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
  };

  const setKittySize = (evt: any) => {
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
  };

  const setKittyPenalty = (evt: any) => {
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
  };

  const setKittyBidPolicy = (evt: any) => {
    evt.preventDefault();
    if (evt.target.value !== '') {
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
      send({
        Action: {
          SetTrickDrawPolicy: evt.target.value,
        },
      });
    }
  };

  const setAdvancementPolicy = (evt: any) => {
    evt.preventDefault();
    if (evt.target.value !== '') {
      send({
        Action: {
          SetAdvancementPolicy: evt.target.value,
        },
      });
    } else {
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

  const setHideLandlordsPoints = (evt: any) => {
    evt.preventDefault();
    send({Action: {SetHideLandlordsPoints: evt.target.value === 'hide'}});
  };

  const setHidePlayedCards = (evt: any) => {
    evt.preventDefault();
    send({Action: {SetHidePlayedCards: evt.target.value === 'hide'}});
  };

  const startGame = (evt: any) => {
    evt.preventDefault();
    send({Action: 'StartGame'});
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
        onKick={(playerId: number) => send({Kick: playerId})}
      />
      <div className="game-settings">
        <h3>Game settings</h3>
        <div>
          <label>
            Game mode:{' '}
            <select value={modeAsString} onChange={setGameMode}>
              <option value="Tractor">升级 / Tractor</option>
              <option value="FindingFriends">找朋友 / Finding Friends</option>
            </select>
          </label>
        </div>
        <div>
          {props.state.propagated.game_mode !== 'Tractor' ? (
            <label>
              Number of friends:{' '}
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
            send({Action: {SetNumDecks: newNumDecks}})
          }
        />
        <div>
          <label>
            Number of cards in the bottom:{' '}
            <select
              value={props.state.propagated.kitty_size || ''}
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
            Point visibility:{' '}
            <select
              value={
                props.state.propagated.hide_landlord_points ? 'hide' : 'show'
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
              value={props.state.propagated.hide_played_cards ? 'hide' : 'show'}
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
            Trump policy for cards revealed from the bottom:{' '}
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
            Penalty for points left in the bottom:{' '}
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
            Penalty for incorrect throws:{' '}
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
            Card protection policy:{' '}
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
        <h3>Continuation settings</h3>
        <LandlordSelector
          players={props.state.propagated.players}
          landlordId={props.state.propagated.landlord}
          onChange={(newLandlord: number | null) =>
            send({Action: {SetLandlord: newLandlord}})
          }
        />
        <RankSelector
          rank={currentPlayer.level}
          onChangeRank={(newRank: string) => send({Action: {SetRank: newRank}})}
        />
      </div>
    </div>
  );
};

export default Initialize;
