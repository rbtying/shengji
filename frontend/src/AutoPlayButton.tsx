import * as React from 'react';

type Props = {
  onSubmit: () => void;
  canSubmit: boolean;
  currentWinner: number | null;
  isCurrentPlayerTurn: boolean;
  unsetAutoPlayWhenWinnerChanges: boolean;
};

type State = {
  autoplay: boolean;
  lastWinner: number | null;
};

const AutoPlayButton = (props: Props) => {
  const {
    onSubmit,
    canSubmit,
    isCurrentPlayerTurn,
    currentWinner,
    unsetAutoPlayWhenWinnerChanges,
  } = props;
  const [autoplay, setAutoplay] = React.useState<State>({
    autoplay: false,
    lastWinner: props.currentWinner,
  });

  React.useEffect(() => {
    if (
      autoplay.autoplay &&
      (!unsetAutoPlayWhenWinnerChanges ||
        autoplay.lastWinner === currentWinner) &&
      isCurrentPlayerTurn
    ) {
      setAutoplay({
        autoplay: false,
        lastWinner: null,
      });
      onSubmit();
    }
  }, [
    autoplay,
    isCurrentPlayerTurn,
    currentWinner,
    unsetAutoPlayWhenWinnerChanges,
  ]);

  React.useEffect(() => {
    if (
      !canSubmit ||
      (autoplay.autoplay &&
        autoplay.lastWinner !== currentWinner &&
        unsetAutoPlayWhenWinnerChanges)
    ) {
      setAutoplay({
        autoplay: false,
        lastWinner: null,
      });
    }
  }, [canSubmit, currentWinner, unsetAutoPlayWhenWinnerChanges]);

  const handleClick = () => {
    if (isCurrentPlayerTurn) {
      onSubmit();
    } else if (autoplay.autoplay) {
      setAutoplay({
        autoplay: false,
        lastWinner: null,
      });
    } else {
      setAutoplay({
        autoplay: true,
        lastWinner: currentWinner,
      });
    }
  };
  return (
    <button onClick={handleClick} disabled={!canSubmit}>
      {isCurrentPlayerTurn
        ? 'Play selected cards'
        : autoplay.autoplay
        ? "Don't autoplay selected cards"
        : 'Autoplay selected cards'}
    </button>
  );
};

export default AutoPlayButton;
