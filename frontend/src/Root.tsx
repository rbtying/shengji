import * as React from 'react';
import Errors from './Errors';
import Initialize from './Initialize';
import Draw from './Draw';
import Exchange from './Exchange';
import JoinRoom from './JoinRoom';
import {AppStateContext} from './AppStateProvider';
import {TimerContext} from './TimerProvider';
import Credits from './Credits';
import Chat from './Chat';
import Play from './Play';

const Root = () => {
  const send = (window as any).send;
  const {state, updateState} = React.useContext(AppStateContext);
  const timerContext = React.useContext(TimerContext);
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
      const cards = [...state.cards];
      if (state.settings.reverseCardOrder) {
        cards.reverse();
      }
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
                cards={cards}
                name={state.name}
              />
            ) : null}
            {state.game_state.Draw ? (
              <Draw
                state={state.game_state.Draw}
                cards={cards}
                name={state.name}
                setTimeout={timerContext.setTimeout}
                clearTimeout={timerContext.clearTimeout}
              />
            ) : null}
            {state.game_state.Exchange ? (
              <Exchange
                state={state.game_state.Exchange}
                cards={cards}
                name={state.name}
              />
            ) : null}
            {state.game_state.Play ? (
              <Play
                playPhase={state.game_state.Play}
                cards={cards}
                name={state.name}
                showLastTrick={state.settings.showLastTrick}
                unsetAutoPlayWhenWinnerChanges={
                  state.settings.unsetAutoPlayWhenWinnerChanges
                }
                showTrickInPlayerOrder={state.settings.showTrickInPlayerOrder}
                beepOnTurn={state.settings.beepOnTurn}
              />
            ) : null}
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

export default Root;
