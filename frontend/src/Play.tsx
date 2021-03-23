import * as React from "react";
import * as ReactModal from "react-modal";
import { IPlayPhase, ITrickFormat, IHands, TrickDrawPolicy } from "./types";
import Header from "./Header";
import Beeper from "./Beeper";
import Trump from "./Trump";
import Friends from "./Friends";
import Trick from "./Trick";
import Cards from "./Cards";
import Points, { calculatePoints } from "./Points";
import LabeledPlay from "./LabeledPlay";
import Players from "./Players";
import ArrayUtils from "./util/array";
import AutoPlayButton from "./AutoPlayButton";
import BeepButton from "./BeepButton";
import { WebsocketContext } from "./WebsocketProvider";
import WasmContext, {
  IFoundViablePlay,
  ISortedAndGroupedCards,
} from "./WasmContext";
import InlineCard from "./InlineCard";

const contentStyle: React.CSSProperties = {
  position: "absolute",
  top: "50%",
  left: "50%",
  transform: "translate(-50%, -50%)",
};

interface IProps {
  playPhase: IPlayPhase;
  name: string;
  beepOnTurn: boolean;
  showLastTrick: boolean;
  unsetAutoPlayWhenWinnerChanges: boolean;
  showTrickInPlayerOrder: boolean;
}

const Play = (props: IProps): JSX.Element => {
  const { send } = React.useContext(WebsocketContext);
  const [selected, setSelected] = React.useState<string[]>([]);
  const [grouping, setGrouping] = React.useState<IFoundViablePlay[]>([]);
  const {
    findViablePlays,
    canPlayCards,
    nextThresholdReachable,
    sortAndGroupCards,
  } = React.useContext(WasmContext);

  const playCards = (): void => {
    send({ Action: { PlayCardsWithHint: [selected, grouping[0].grouping] } });
    setSelected([]);
    setGrouping([]);
  };

  const sendEvent = (event: {}) => () => send(event);
  const takeBackCards = sendEvent({ Action: "TakeBackCards" });
  const endTrick = sendEvent({ Action: "EndTrick" });
  const endGameEarly = sendEvent({ Action: "EndGameEarly" });
  const startNewGame = sendEvent({ Action: "StartNewGame" });

  const { playPhase } = props;

  // TODO: instead of telling who the player is by checking the name, pass in
  // the Player object
  let isSpectator = true;
  let currentPlayer = playPhase.propagated.players.find(
    (p) => p.name === props.name
  );
  if (currentPlayer === undefined) {
    currentPlayer = playPhase.propagated.observers.find(
      (p) => p.name === props.name
    );
  } else {
    isSpectator = false;
  }
  if (currentPlayer === undefined) {
    currentPlayer = {
      id: -1,
      name: props.name,
      level: "",
      metalevel: 0,
    };
  }

  React.useEffect(() => {
    // When the hands change, our `selected` cards may become invalid, since we
    // could have raced and selected cards that we just played.
    //
    // In that case, let's fix the selected cards.
    const hand =
      currentPlayer.id in playPhase.hands.hands
        ? { ...playPhase.hands.hands[currentPlayer.id] }
        : {};
    selected.forEach((card) => {
      if (card in hand) {
        hand[card] = hand[card] - 1;
      } else {
        hand[card] = -1;
      }
    });

    const toRemove = Object.entries(hand)
      .filter((x) => x[1] < 0)
      .map((x) => x[0]);

    const newSelected = ArrayUtils.minus(selected, toRemove);

    setSelected(newSelected);
    setGrouping(findViablePlays(playPhase.trump, newSelected));
  }, [playPhase.hands.hands, currentPlayer.id]);

  const nextPlayer = playPhase.trick.player_queue[0];
  const lastPlay =
    playPhase.trick.played_cards[playPhase.trick.played_cards.length - 1];

  const isCurrentPlayerTurn = currentPlayer.id === nextPlayer;
  let canPlay = false;
  if (!isSpectator) {
    canPlay = canPlayCards({
      trick: playPhase.trick,
      id: currentPlayer.id,
      hands: playPhase.hands,
      cards: selected,
      trick_draw_policy: playPhase.propagated.trick_draw_policy,
    });
    // In order to play the first trick, the grouping must be disambiguated!
    if (lastPlay === undefined) {
      canPlay = canPlay && grouping.length === 1;
    }
  }
  canPlay = canPlay && !playPhase.game_ended_early;
  const canTakeBack =
    lastPlay !== undefined &&
    currentPlayer.id === lastPlay.id &&
    !playPhase.game_ended_early;

  const shouldBeBeeping =
    props.beepOnTurn && isCurrentPlayerTurn && !playPhase.game_ended_early;

  const remainingCardsInHands = ArrayUtils.sum(
    Object.values(playPhase.hands.hands).map((playerHand) =>
      ArrayUtils.sum(Object.values(playerHand))
    )
  );

  const { totalPointsPlayed, nonLandlordPointsWithPenalties } = calculatePoints(
    playPhase.propagated.players,
    playPhase.landlords_team,
    playPhase.points,
    playPhase.penalties
  );

  const noCardsLeft =
    remainingCardsInHands === 0 && playPhase.trick.played_cards.length === 0;

  const canFinish = noCardsLeft || playPhase.game_ended_early;

  const canEndGameEarly =
    !canFinish &&
    !nextThresholdReachable({
      decks: playPhase.decks,
      params: playPhase.propagated.game_scoring_parameters,
      non_landlord_points: nonLandlordPointsWithPenalties,
      observed_points: totalPointsPlayed,
    });

  const landlordSuffix =
    playPhase.propagated.landlord_emoji !== undefined &&
    playPhase.propagated.landlord_emoji !== null &&
    playPhase.propagated.landlord_emoji !== ""
      ? playPhase.propagated.landlord_emoji
      : "(当庄)";

  const landlordTeamSize = playPhase.landlords_team.length;
  let configFriendTeamSize = 0;
  let smallerTeamSize = false;
  if (playPhase.game_mode !== "Tractor") {
    configFriendTeamSize =
      playPhase.game_mode.FindingFriends.num_friends != null
        ? playPhase.game_mode.FindingFriends.num_friends + 1
        : playPhase.propagated.players.length / 2;
    smallerTeamSize = landlordTeamSize < configFriendTeamSize;
  }

  const getCardsFromHand = (pid: number): ISortedAndGroupedCards[] => {
    const cardsInHand =
      pid in playPhase.hands.hands
        ? Object.entries(playPhase.hands.hands[pid]).flatMap(([c, ct]) =>
            Array(ct).fill(c)
          )
        : [];
    return sortAndGroupCards({
      cards: cardsInHand,
      trump: props.playPhase.trump,
    });
  };

  return (
    <div>
      {shouldBeBeeping ? <Beeper /> : null}
      <Header
        gameMode={playPhase.propagated.game_mode}
        chatLink={playPhase.propagated.chat_link}
      />
      <Players
        players={playPhase.propagated.players}
        observers={playPhase.propagated.observers}
        landlord={playPhase.landlord}
        landlords_team={playPhase.landlords_team}
        name={props.name}
        next={nextPlayer}
      />
      <Trump trump={playPhase.trump} />
      <Friends gameMode={playPhase.game_mode} showPlayed={true} />
      {playPhase.removed_cards.length > 0 ? (
        <p>
          Note:{" "}
          {playPhase.removed_cards.map((c) => (
            <InlineCard key={c} card={c} />
          ))}{" "}
          have been removed from the deck
        </p>
      ) : null}
      <Trick
        trick={playPhase.trick}
        players={playPhase.propagated.players}
        landlord={playPhase.landlord}
        landlord_suffix={landlordSuffix}
        landlords_team={playPhase.landlords_team}
        next={nextPlayer}
        name={props.name}
        showTrickInPlayerOrder={props.showTrickInPlayerOrder}
      />
      <AutoPlayButton
        onSubmit={playCards}
        playDescription={
          grouping.length === 1 && lastPlay === undefined
            ? grouping[0].description
            : null
        }
        canSubmit={canPlay}
        currentWinner={playPhase.trick.current_winner}
        unsetAutoPlayWhenWinnerChanges={props.unsetAutoPlayWhenWinnerChanges}
        isCurrentPlayerTurn={isCurrentPlayerTurn}
      />
      {playPhase.propagated.play_takeback_policy === "AllowPlayTakeback" && (
        <button onClick={takeBackCards} disabled={!canTakeBack}>
          Take back last play
        </button>
      )}
      <button
        onClick={endTrick}
        disabled={
          playPhase.trick.player_queue.length > 0 || playPhase.game_ended_early
        }
      >
        Finish trick
      </button>
      {canEndGameEarly && (
        <button
          onClick={() => {
            confirm(
              "Do you want to end the game early? There may still be points in the bottom..."
            ) && endGameEarly();
          }}
        >
          End game early
        </button>
      )}
      {canFinish && <button onClick={startNewGame}>Finish game</button>}
      <BeepButton />
      {canFinish && !noCardsLeft && (
        <div>
          <p>Cards remaining (that were not played):</p>
          {playPhase.propagated.players.map((p) => (
            <LabeledPlay
              key={p.id}
              label={p.name}
              cards={getCardsFromHand(p.id).flatMap((g) => g.cards)}
            />
          ))}
        </div>
      )}
      {!canFinish && (
        <>
          {playPhase.trick.trick_format !== null &&
          !isSpectator &&
          playPhase.trick.player_queue.includes(currentPlayer.id) ? (
            <TrickFormatHelper
              format={playPhase.trick.trick_format}
              hands={playPhase.hands}
              playerId={currentPlayer.id}
              trickDrawPolicy={playPhase.propagated.trick_draw_policy}
            />
          ) : null}
          {lastPlay === undefined &&
            isCurrentPlayerTurn &&
            grouping.length > 1 && (
              <div>
                <p>
                  It looks like you are making a play that can be interpreted in
                  multiple ways!
                </p>
                <p>Which of the following did you mean?</p>
                {grouping.map((g, gidx) => (
                  <button
                    key={gidx}
                    onClick={(evt) => {
                      evt.preventDefault();
                      setGrouping([g]);
                    }}
                    className="normal"
                  >
                    {g.description}
                  </button>
                ))}
              </div>
            )}
          <Cards
            hands={playPhase.hands}
            playerId={currentPlayer.id}
            trump={playPhase.trump}
            selectedCards={selected}
            onSelect={(newSelected) => {
              setSelected(newSelected);
              setGrouping(findViablePlays(playPhase.trump, newSelected));
            }}
            notifyEmpty={isCurrentPlayerTurn}
          />
        </>
      )}
      {playPhase.last_trick !== undefined &&
      playPhase.last_trick !== null &&
      props.showLastTrick ? (
        <div>
          <p>Previous trick</p>
          <Trick
            trick={playPhase.last_trick}
            players={playPhase.propagated.players}
            landlord={playPhase.landlord}
            landlord_suffix={landlordSuffix}
            landlords_team={playPhase.landlords_team}
            name={props.name}
            showTrickInPlayerOrder={props.showTrickInPlayerOrder}
          />
        </div>
      ) : null}
      <Points
        points={playPhase.points}
        penalties={playPhase.penalties}
        decks={playPhase.decks}
        players={playPhase.propagated.players}
        landlordTeam={playPhase.landlords_team}
        landlord={playPhase.landlord}
        hideLandlordPoints={playPhase.propagated.hide_landlord_points}
        gameScoringParameters={playPhase.propagated.game_scoring_parameters}
        smallerTeamSize={smallerTeamSize}
      />
      <LabeledPlay className="kitty" cards={playPhase.kitty} label="底牌" />
    </div>
  );
};

const TrickFormatHelper = (props: {
  format: ITrickFormat;
  hands: IHands;
  playerId: number;
  trickDrawPolicy: TrickDrawPolicy;
}): JSX.Element => {
  const [modalOpen, setModalOpen] = React.useState<boolean>(false);
  const { decomposeTrickFormat } = React.useContext(WasmContext);
  const decomp = decomposeTrickFormat({
    trick_format: props.format,
    hands: props.hands,
    player_id: props.playerId,
    trick_draw_policy: props.trickDrawPolicy,
  });
  const trickSuit = props.format.suit;
  const bestMatch = decomp.findIndex((d) => d.playable.length > 0);

  return (
    <>
      <button
        onClick={(evt) => {
          evt.preventDefault();
          setModalOpen(true);
        }}
      >
        ?
      </button>
      <ReactModal
        isOpen={modalOpen}
        onRequestClose={() => setModalOpen(false)}
        shouldCloseOnOverlayClick
        shouldCloseOnEsc
        style={{ content: contentStyle }}
      >
        <p>
          In order to win, you have to play {decomp[0].description} in{" "}
          {trickSuit}
        </p>
        {decomp[0].playable.length > 0 && (
          <p>
            It looks like you are able to match this format, e.g. with
            {decomp[0].playable.map((c, cidx) => (
              <InlineCard key={cidx} card={c} />
            ))}
          </p>
        )}

        {decomp.length > 1 && props.trickDrawPolicy !== "NoFormatBasedDraw" && (
          <>
            <p>
              If you can&apos;t play that, but you <em>can</em> play one of the
              following, you have to play it
            </p>
            <ol>
              {decomp.slice(1).map((d, idx) => (
                <li
                  key={idx}
                  style={{
                    fontWeight: idx === bestMatch - 1 ? "bold" : "normal",
                  }}
                >
                  {d.description} in {trickSuit}
                  {idx === bestMatch - 1 && (
                    <>
                      {" "}
                      (for example:{" "}
                      {d.playable.map((c, cidx) => (
                        <InlineCard key={cidx} card={c} />
                      ))}
                      )
                    </>
                  )}
                </li>
              ))}
            </ol>
          </>
        )}
        <p>
          Otherwise, you have to play as many {trickSuit} as you can. The
          remaining cards can be anything.
        </p>
        {trickSuit !== "Trump" && (
          <p>
            If you have no cards in {trickSuit}, you can play{" "}
            {decomp[0].description} in Trump to potentially win the trick.
          </p>
        )}
      </ReactModal>
    </>
  );
};

export default Play;
