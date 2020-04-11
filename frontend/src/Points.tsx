import * as React from 'react';
import {IPlayer} from './types';
import ArrayUtils from './util/array';
import ObjectUtils from './util/object';
import LabeledPlay from './LabeledPlay';
import classNames from 'classnames';

type Props = {
  players: IPlayer[];
  numDecks: number;
  points: {[playerId: number]: string[]};
  landlordTeam: number[];
  landlord: number;
  hideLandlordPoints: boolean;
};

const Points = (props: Props) => {
  const pointsPerPlayer = ObjectUtils.mapValues(props.points, (cards) =>
    ArrayUtils.sum(cards.map((card) => (window as any).CARD_LUT[card].points)),
  );
  const totalPointsPlayed = ArrayUtils.sum(Object.values(pointsPerPlayer));
  const nonLandlordPoints = ArrayUtils.sum(
    props.players
      .filter((p) => !props.landlordTeam.includes(p.id))
      .map((p) => pointsPerPlayer[p.id]),
  );

  const playerPointElements = props.players.map((player) => {
    const onLandlordTeam = props.landlordTeam.includes(player.id);
    const cards =
      props.points[player.id].length > 0 ? props.points[player.id] : ['🂠'];

    if (props.hideLandlordPoints && onLandlordTeam) {
      return null;
    } else {
      return (
        <LabeledPlay
          key={player.id}
          className={classNames({landlord: onLandlordTeam})}
          label={`${player.name}: ${pointsPerPlayer[player.id]}分`}
          cards={cards}
        />
      );
    }
  });

  // TODO: Pass the landlord as a Player object instead of numeric ID
  const landlord = props.players.find((p) => p.id === props.landlord);

  const segment = props.numDecks * 20;
  let thresholdStr = '';

  if (nonLandlordPoints === 0) {
    thresholdStr = `${landlord}'s team will go up 3 levels (next threshold: 5分)`;
  } else if (nonLandlordPoints < segment) {
    thresholdStr = `${landlord}'s team will go up 2 levels (next threshold: ${segment}分)`;
  } else if (nonLandlordPoints < 2 * segment) {
    thresholdStr = `${landlord}'s team will go up 1 level (next threshold: ${
      2 * segment
    }分)`;
  } else if (nonLandlordPoints < 3 * segment) {
    thresholdStr = `Neither team will go up a level (next threshold: ${
      3 * segment
    }分)`;
  } else if (nonLandlordPoints < 4 * segment) {
    thresholdStr = `The attacking team will go up 1 level (next threshold: ${
      4 * segment
    }分)`;
  } else if (nonLandlordPoints < 5 * segment) {
    thresholdStr = `The attacking team will go up 2 levels (next threshold: ${
      5 * segment
    }分)`;
  } else {
    thresholdStr = 'The attacking team will go up 3 levels.';
  }

  return (
    <div className="points">
      <h2>Points</h2>
      <p>
        {nonLandlordPoints}分
        {props.hideLandlordPoints ? null : ` / ${totalPointsPlayed}分`} stolen
        from {landlord.name}'s team. {thresholdStr}
      </p>
      {playerPointElements}
    </div>
  );
};

export default Points;
