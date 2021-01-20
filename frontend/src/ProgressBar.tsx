import * as React from "react";

interface IProps {
  numDecks: number;
  challengerPoints: number;
  landlordPoints: number;
}

interface CheckpointCircleProps {
  text: number;
  color: string;
  position: string;
  marginTop: string;
}

const CheckpointCircle = (props: CheckpointCircleProps): JSX.Element => {
  return (
    <div
      className="checkpoint-circle"
      style={{
        position: "relative",
        left: props.position,
        transform: "translate(-50%, 0%)",
        backgroundColor: props.color,
        marginTop: props.marginTop,
      }}
    >
      <div className="checkpoint-circle-content">{props.text}</div>
    </div>
  );
};

const convertToPercentage = (proportion: number): string => {
  return (100 * proportion).toFixed(2) + "%";
};

const ProgressBar = (props: IProps): JSX.Element => {
  const landlordColor = "#5bc0de";
  const challengerColor = "#d9534f";
  const neutralColor = "lightgray";

  const { numDecks, challengerPoints, landlordPoints } = props;
  const totalPoints = numDecks * 100;
  const checkpoints = [0.2, 0.4, 0.6, 0.8].map(
    (proportion) => proportion * totalPoints
  );
  const checkpointColors = checkpoints.map((checkpoint) => {
    if (landlordPoints >= checkpoint) {
      return landlordColor;
    } else if (challengerPoints >= totalPoints - checkpoint) {
      return challengerColor;
    } else {
      return neutralColor;
    }
  });
  const landlordPosition = convertToPercentage(landlordPoints / totalPoints);
  const challengerPosition = convertToPercentage(
    (totalPoints - challengerPoints) / totalPoints
  );
  const challengerWidth = convertToPercentage(challengerPoints / totalPoints);

  return (
    <div>
      <div className="progress-bar-neutral">
        <div
          className="progress-bar-landlord"
          style={{ width: landlordPosition }}
        />
        <div
          className="progress-bar-challenger"
          style={{
            marginTop: "-20px",
            position: "relative",
            left: challengerPosition,
            width: challengerWidth,
          }}
        />
      </div>
      <CheckpointCircle
        text={checkpoints[0]}
        color={checkpointColors[0]}
        position="20%"
        marginTop="-30px"
      />
      <CheckpointCircle
        text={checkpoints[1]}
        color={checkpointColors[1]}
        position="40%"
        marginTop="-40px"
      />
      <CheckpointCircle
        text={checkpoints[2]}
        color={checkpointColors[2]}
        position="60%"
        marginTop="-40px"
      />
      <CheckpointCircle
        text={checkpoints[3]}
        color={checkpointColors[3]}
        position="80%"
        marginTop="-40px"
      />
    </div>
  );
};

export default ProgressBar;
