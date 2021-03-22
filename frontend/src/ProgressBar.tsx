import * as React from "react";

interface IProps {
  checkpoints: number[];
  totalPoints: number;
  challengerPoints: number;
  landlordPoints: number;
  hideLandlordPoints: boolean;
}

interface CheckpointCircleProps {
  text: number;
  color: string;
  borderColor?: string;
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
        height: "30px",
        width: "30px",
        borderWidth: "5px",
        borderStyle: "solid",
        borderColor:
          props.borderColor !== undefined ? props.borderColor : props.color,
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
  const landlordColor = "#d9534f";
  const challengerColor = "#5bc0de";
  const neutralColor = "lightgray";

  const { totalPoints, challengerPoints, landlordPoints } = props;
  const checkpointColors = props.checkpoints.map((checkpoint) => {
    if (challengerPoints >= checkpoint) {
      return challengerColor;
    } else if (landlordPoints >= totalPoints - checkpoint) {
      return landlordColor;
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
    <div style={{ color: "#000" }}>
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
            backgroundColor: challengerColor,
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
              backgroundColor: landlordColor,
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
      <CheckpointCircle
        text={challengerPoints}
        color={"#fff"}
        borderColor={challengerColor}
        position={challengerPosition}
        marginTop={"-40px"}
      />
      {!props.hideLandlordPoints && (
        <CheckpointCircle
          text={totalPoints - landlordPoints}
          color={"#fff"}
          borderColor={landlordColor}
          position={landlordPosition}
          marginTop={"-40px"}
        />
      )}
    </div>
  );
};

export default ProgressBar;
