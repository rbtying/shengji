import * as React from 'react';

type Props = {
  onSubmit: () => void;
  canSubmit: boolean;
  currentWinner: number | null;
  isCurrentPlayerTurn: boolean;
  unsetAutoPlayWhenWinnerChanges: boolean;
};

type AutoPlay = {
  observedWinner: number | null;
} | null;

const AutoPlayButton = (props: Props) => {
  const {
    onSubmit,
    canSubmit,
    isCurrentPlayerTurn,
    currentWinner,
    unsetAutoPlayWhenWinnerChanges,
  } = props;

  const [autoplay, setAutoplay] = React.useState<AutoPlay>(null);

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

  const handleClick = () => {
    if (isCurrentPlayerTurn) {
      onSubmit();
    } else if (autoplay !== null) {
      setAutoplay(null);
    } else {
      setAutoplay({observedWinner: currentWinner});
    }
  };
  return (
    <button onClick={handleClick} disabled={!canSubmit}>
      {isCurrentPlayerTurn
        ? 'Play selected cards'
        : autoplay
        ? "Don't autoplay selected cards"
        : 'Autoplay selected cards'}
    </button>
  );
};

export default AutoPlayButton;
