import * as React from 'react';
import {IPlayPhase} from './types';
import Header from './Header';
import Beeper from './Beeper';
import Trump from './Trump';
import Friends from './Friends';
import Trick from './Trick';
import Cards from './Cards';
import Points from './Points';
import LabeledPlay from './LabeledPlay';
import Players from './Players';
import ArrayUtils from './util/array';
import AutoPlayButton from './AutoPlayButton';
import {WebsocketContext} from './WebsocketProvider';

type Props = {
  playPhase: IPlayPhase;
  name: string;
  cards: string[];
  beepOnTurn: boolean;
  showLastTrick: boolean;
  unsetAutoPlayWhenWinnerChanges: boolean;
};

const Play = (props: Props) => {
  const {send} = React.useContext(WebsocketContext);
  const [selected, setSelected] = React.useState<string[]>([]);

  const playCards = () => {
    send({Action: {PlayCards: selected}});
    setSelected([]);
  };

  const sendEvent = (event: {}) => () => send(event);
  const takeBackCards = sendEvent({Action: 'TakeBackCards'});
  const endTrick = sendEvent({Action: 'EndTrick'});
  const startNewGame = sendEvent({Action: 'StartNewGame'});

  const {playPhase} = props;

  // TODO: instead of telling who the player is by checking the name, pass in
  // the Player object
  let currentPlayer = playPhase.propagated.players.find(
    (p) => p.name === props.name,
  );
  if (!currentPlayer) {
    currentPlayer = playPhase.propagated.observers.find(
      (p) => p.name === props.name,
    );
  }
  const nextPlayer = playPhase.trick.player_queue[0];
  const lastPlay =
    playPhase.trick.played_cards[playPhase.trick.played_cards.length - 1];

  const isCurrentPlayerTurn = currentPlayer.id === nextPlayer;
  const canPlay = lastPlay
    ? selected.length === lastPlay.cards.length
    : selected.length > 0;
  const canTakeBack = lastPlay && currentPlayer.id === lastPlay.id;

  const shouldBeBeeping = props.beepOnTurn && isCurrentPlayerTurn;

  const remainingCardsInHands = ArrayUtils.sum(
    Object.values(playPhase.hands.hands).map((playerHand) =>
      ArrayUtils.sum(Object.values(playerHand)),
    ),
  );
  const canFinish =
    remainingCardsInHands === 0 && playPhase.trick.played_cards.length === 0;

  return (
    <div>
      {shouldBeBeeping ? <Beeper /> : null}
      <Header gameMode={playPhase.propagated.game_mode} />
      <Players
        players={playPhase.propagated.players}
        landlord={playPhase.landlord}
        landlords_team={playPhase.landlords_team}
        name={props.name}
        next={nextPlayer}
      />
      <Trump trump={playPhase.trump} />
      <Friends gameMode={playPhase.game_mode} />
      <Trick trick={playPhase.trick} players={playPhase.propagated.players} />
      <AutoPlayButton
        onSubmit={playCards}
        canSubmit={canPlay}
        currentWinner={playPhase.trick.current_winner}
        unsetAutoPlayWhenWinnerChanges={props.unsetAutoPlayWhenWinnerChanges}
        isCurrentPlayerTurn={isCurrentPlayerTurn}
      />
      <button onClick={takeBackCards} disabled={!canTakeBack}>
        Take back last play
      </button>
      <button
        onClick={endTrick}
        disabled={playPhase.trick.player_queue.length > 0}
      >
        Finish trick
      </button>
      {canFinish && <button onClick={startNewGame}>Finish game</button>}
      <Cards
        cardsInHand={props.cards}
        selectedCards={selected}
        onSelect={setSelected}
        notifyEmpty={isCurrentPlayerTurn}
      />
      {playPhase.last_trick && props.showLastTrick ? (
        <div>
          <p>Previous trick</p>
          <Trick
            trick={playPhase.last_trick}
            players={playPhase.propagated.players}
          />
        </div>
      ) : null}
      <Points
        points={playPhase.points}
        penalties={playPhase.penalties}
        numDecks={playPhase.num_decks}
        players={playPhase.propagated.players}
        landlordTeam={playPhase.landlords_team}
        landlord={playPhase.landlord}
        hideLandlordPoints={playPhase.propagated.hide_landlord_points}
      />
      <LabeledPlay cards={playPhase.kitty} label="底牌" />
    </div>
  );
};

export default Play;
