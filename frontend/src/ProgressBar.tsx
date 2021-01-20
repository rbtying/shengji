import * as React from "react";

interface IProps {
  checkpoints: number[];
  numDecks: number;
  challengerPoints: number;
  landlordPoints: number;
  hideLandlordPoints: boolean;
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
      style={{
        position: "relative",
        left: props.position,
        transform: "translate(-50%, 0%)",
        backgroundColor: props.color,
        marginTop: props.marginTop,
        height: "40px",
        width: "40px",
        borderRadius: "25px",
      }}
    >
      <div
        style={{
          width: "100%",
          height: "100%",
          display: "flex",
          justifyContent: "center",
          alignItems: "center",
        }}
      >
        {props.text}
      </div>
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
  const checkpointColors = props.checkpoints.map((checkpoint) => {
    if (landlordPoints >= checkpoint) {
      return landlordColor;
    } else if (challengerPoints >= totalPoints - checkpoint) {
      return challengerColor;
    } else {
      return neutralColor;
    }
  });
  const landlordPosition = convertToPercentage(
    (totalPoints - landlordPoints) / totalPoints
  );
  const landlordWidth = convertToPercentage(landlordPoints / totalPoints);
  const challengerPosition = convertToPercentage(
    challengerPoints / totalPoints
  );

  return (
    <div>
      <div
        style={{
          width: "100%",
          borderRadius: "5px",
          backgroundColor: "lightgray",
        }}
      >
        <div
          style={{
            width: challengerPosition,
            height: "20px",
            borderRadius: "5px",
            backgroundColor: "#5bc0de",
          }}
        />
        {!props.hideLandlordPoints && (
          <div
            className="progress-bar-landlord"
            style={{
              marginTop: "-20px",
              position: "relative",
              left: landlordPosition,
              width: landlordWidth,
              height: "20px",
              borderRadius: "5px",
              backgroundColor: "#d9534f",
            }}
          />
        )}
      </div>
      {props.checkpoints.map((checkpoint, i) => {
        return (
          <CheckpointCircle
            key={i}
            text={checkpoint}
            color={checkpointColors[i]}
            position={convertToPercentage(checkpoint / totalPoints)}
            marginTop={i === 0 ? "-30px" : "-40px"}
          />
        );
      })}
    </div>
  );
};

export default ProgressBar;
