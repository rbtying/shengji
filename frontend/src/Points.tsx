import * as React from "react";
import { IPlayer } from "./types";
import ArrayUtils from "./util/array";
import ObjectUtils from "./util/object";
import LabeledPlay from "./LabeledPlay";
import classNames from "classnames";
import { cardLookup } from "./util/cardHelpers";

interface IProps {
  players: IPlayer[];
  numDecks: number;
  points: { [playerId: number]: string[] };
  penalties: { [playerId: number]: number };
  landlordTeam: number[];
  landlord: number;
  hideLandlordPoints: boolean;
}

const Points = (props: IProps): JSX.Element => {
  const pointsPerPlayer = ObjectUtils.mapValues(props.points, (cards) =>
    ArrayUtils.sum(cards.map((card) => cardLookup[card].points))
  );
  const totalPointsPlayed = ArrayUtils.sum(Object.values(pointsPerPlayer));
  const nonLandlordPoints = ArrayUtils.sum(
    props.players
      .filter((p) => !props.landlordTeam.includes(p.id))
      .map((p) => pointsPerPlayer[p.id])
  );

  let nonLandlordPointsWithPenalties = nonLandlordPoints;
  props.players.forEach((p) => {
    const penalty = props.penalties[p.id];
    if (penalty > 0) {
      if (props.landlordTeam.includes(p.id)) {
        nonLandlordPointsWithPenalties += penalty;
      } else {
        nonLandlordPointsWithPenalties = Math.max(
          0,
          nonLandlordPoints - penalty
        );
      }
    }
  });
  const penaltyDelta = nonLandlordPointsWithPenalties - nonLandlordPoints;

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

  const segment = props.numDecks * 20;
  let thresholdStr = "";

  if (nonLandlordPointsWithPenalties === 0) {
    thresholdStr = `${landlord.name}'s team will go up 3 levels (next threshold: 5分)`;
  } else if (nonLandlordPointsWithPenalties < segment) {
    thresholdStr = `${landlord.name}'s team will go up 2 levels (next threshold: ${segment}分)`;
  } else if (nonLandlordPointsWithPenalties < 2 * segment) {
    thresholdStr = `${
      landlord.name
    }'s team will go up 1 level (next threshold: ${2 * segment}分)`;
  } else if (nonLandlordPointsWithPenalties < 3 * segment) {
    thresholdStr = `Neither team will go up a level (next threshold: ${
      3 * segment
    }分)`;
  } else if (nonLandlordPointsWithPenalties < 4 * segment) {
    thresholdStr = `The attacking team will go up 1 level (next threshold: ${
      4 * segment
    }分)`;
  } else if (nonLandlordPointsWithPenalties < 5 * segment) {
    thresholdStr = `The attacking team will go up 2 levels (next threshold: ${
      5 * segment
    }分)`;
  } else {
    thresholdStr = "The attacking team will go up 3 levels.";
  }

  return (
    <div className="points">
      <h2>Points</h2>
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
