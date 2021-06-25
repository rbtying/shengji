import * as React from "react";
import ProgressBar from "./ProgressBar";
import { IPlayer, IGameScoringParameters, IDeck } from "./types";
import ArrayUtils from "./util/array";
import ObjectUtils from "./util/object";
import LabeledPlay from "./LabeledPlay";
import classNames from "classnames";
import { cardLookup } from "./util/cardHelpers";
import WasmContext from "./WasmContext";

interface IProps {
  players: IPlayer[];
  decks: IDeck[];
  points: { [playerId: number]: string[] };
  penalties: { [playerId: number]: number };
  landlordTeam: number[];
  landlord: number;
  hideLandlordPoints: boolean;
  smallerTeamSize: boolean;
  gameScoringParameters: IGameScoringParameters;
}

export const calculatePoints = (
  players: IPlayer[],
  landlordTeam: number[],
  points: { [playerId: number]: string[] },
  penalties: { [playerId: number]: number }
): {
  nonLandlordPoints: number;
  totalPointsPlayed: number;
  nonLandlordPointsWithPenalties: number;
} => {
  const pointsPerPlayer = ObjectUtils.mapValues(points, (cards) =>
    ArrayUtils.sum(cards.map((card) => cardLookup[card].points))
  );
  const totalPointsPlayed = ArrayUtils.sum(Object.values(pointsPerPlayer));
  const nonLandlordPoints = ArrayUtils.sum(
    players
      .filter((p) => !landlordTeam.includes(p.id))
      .map((p) => pointsPerPlayer[p.id])
  );

  let nonLandlordPointsWithPenalties = nonLandlordPoints;
  players.forEach((p) => {
    const penalty = penalties[p.id];
    if (penalty > 0) {
      if (landlordTeam.includes(p.id)) {
        nonLandlordPointsWithPenalties += penalty;
      } else {
        nonLandlordPointsWithPenalties -= penalty;
      }
    }
  });

  return {
    nonLandlordPoints,
    nonLandlordPointsWithPenalties,
    totalPointsPlayed,
  };
};

const Points = (props: IProps): JSX.Element => {
  const pointsPerPlayer = ObjectUtils.mapValues(props.points, (cards) =>
    ArrayUtils.sum(cards.map((card) => cardLookup[card].points))
  );
  const { computeScore, explainScoring } = React.useContext(WasmContext);
  const {
    totalPointsPlayed,
    nonLandlordPointsWithPenalties,
    nonLandlordPoints,
  } = calculatePoints(
    props.players,
    props.landlordTeam,
    props.points,
    props.penalties
  );
  const penaltyDelta = nonLandlordPointsWithPenalties - nonLandlordPoints;

  const { score, next_threshold: nextThreshold } = computeScore({
    params: props.gameScoringParameters,
    decks: props.decks,
    smaller_landlord_team_size: props.smallerTeamSize,
    non_landlord_points: nonLandlordPointsWithPenalties,
  });

  const playerPointElements = props.players.map((player) => {
    const onLandlordTeam = props.landlordTeam.includes(player.id);
    const cards =
      props.points[player.id].length > 0 ? props.points[player.id] : ["🂠"];
    const penalty =
      player.id in props.penalties ? props.penalties[player.id] : 0;

    if (props.hideLandlordPoints && onLandlordTeam) {
      return null;
    } else {
      return (
        <LabeledPlay
          key={player.id}
          className={classNames({ landlord: onLandlordTeam })}
          label={`${player.name}: ${pointsPerPlayer[player.id] - penalty}分`}
          cards={cards}
        />
      );
    }
  });

  // TODO: Pass the landlord as a Player object instead of numeric ID
  const landlord = props.players.find((p) => p.id === props.landlord);

  let thresholdStr = "";
  if (score.landlord_won) {
    thresholdStr = `${landlord.name}'s team will go up ${
      score.landlord_delta
    } level${score.landlord_delta === 1 ? "" : "s"}`;
    if (score.landlord_bonus) {
      thresholdStr += ", including a small-team bonus";
    }
  } else if (score.non_landlord_delta === 0) {
    thresholdStr = "Neither team will go up a level";
  } else {
    thresholdStr = `The attacking team will go up ${
      score.non_landlord_delta
    } level${score.non_landlord_delta === 1 ? "" : "s"}`;
  }

  thresholdStr += ` (next threshold: ${nextThreshold}分)`;

  const { results: scoreTransitions, total_points: totalPoints } =
    explainScoring({
      params: props.gameScoringParameters,
      smaller_landlord_team_size: props.smallerTeamSize,
      decks: props.decks,
    });

  return (
    <div className="points">
      <h2>Points</h2>
      <ProgressBar
        checkpoints={scoreTransitions
          .map((transition) => transition.point_threshold)
          .filter((threshold) => threshold >= 10 && threshold < totalPoints)}
        totalPoints={totalPoints}
        landlordPoints={totalPointsPlayed - nonLandlordPoints}
        challengerPoints={nonLandlordPointsWithPenalties}
        hideLandlordPoints={props.hideLandlordPoints}
      />
      <p>
        {penaltyDelta === 0
          ? nonLandlordPoints
          : `${nonLandlordPoints} + ${penaltyDelta}`}
        分{props.hideLandlordPoints ? null : ` / ${totalPointsPlayed}分`} stolen
        from {landlord.name}&apos;s team. {thresholdStr}
      </p>
      {playerPointElements}
    </div>
  );
};

export default Points;
