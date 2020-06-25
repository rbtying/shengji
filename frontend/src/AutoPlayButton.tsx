import * as React from "react";

interface IProps {
  onSubmit: () => void;
  canSubmit: boolean;
  currentWinner: number | null;
  isCurrentPlayerTurn: boolean;
  unsetAutoPlayWhenWinnerChanges: boolean;
}

type AutoPlay = {
  observedWinner: number | null;
} | null;

const AutoPlayButton = (props: IProps): JSX.Element => {
  const {
    onSubmit,
    canSubmit,
    isCurrentPlayerTurn,
    currentWinner,
    unsetAutoPlayWhenWinnerChanges,
  } = props;

  const [autoplay, setAutoplay] = React.useState<AutoPlay | null>(null);

  React.useEffect(() => {
    if (autoplay !== null) {
      if (!canSubmit) {
        setAutoplay(null);
      } else if (
        unsetAutoPlayWhenWinnerChanges &&
        autoplay.observedWinner !== currentWinner
      ) {
        setAutoplay(null);
      } else if (isCurrentPlayerTurn) {
        setAutoplay(null);
        onSubmit();
      }
    }
  }, [
    autoplay,
    canSubmit,
    currentWinner,
    isCurrentPlayerTurn,
    unsetAutoPlayWhenWinnerChanges,
  ]);

  const handleClick = (): void => {
    if (isCurrentPlayerTurn) {
      onSubmit();
    } else if (autoplay !== null) {
      setAutoplay(null);
    } else {
      setAutoplay({ observedWinner: currentWinner });
    }
  };
  return (
    <button onClick={handleClick} disabled={!canSubmit}>
      {isCurrentPlayerTurn
        ? "Play selected cards"
        : autoplay !== null
        ? "Don't autoplay selected cards"
        : "Autoplay selected cards"}
    </button>
  );
};

export default AutoPlayButton;
