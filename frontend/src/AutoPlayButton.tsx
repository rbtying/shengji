import * as React from 'react';
import usePrevious from './util/usePrevious';

type Props = {
  onSubmit: () => void;
  canSubmit: boolean;
  currentWinner: number | null;
  isCurrentPlayerTurn: boolean;
  unsetAutoPlayWhenWinnerChanges: boolean;
};

const AutoPlayButton = (props: Props) => {
  const {
    onSubmit,
    canSubmit,
    isCurrentPlayerTurn,
    currentWinner,
    unsetAutoPlayWhenWinnerChanges,
  } = props;

  const [autoplay, setAutoplay] = React.useState<boolean>(false);
  const previousWinner = usePrevious<number | null>(currentWinner);

  React.useEffect(() => {
    if (autoplay) {
      if (!canSubmit) {
        setAutoplay(false);
      } else if (
        unsetAutoPlayWhenWinnerChanges &&
        previousWinner &&
        previousWinner !== currentWinner
      ) {
        setAutoplay(false);
      } else if (isCurrentPlayerTurn) {
        setAutoplay(false);
        onSubmit();
      }
    }
  }, [
    autoplay,
    canSubmit,
    currentWinner,
    isCurrentPlayerTurn,
    currentWinner,
    unsetAutoPlayWhenWinnerChanges,
  ]);

  const handleClick = () => {
    if (isCurrentPlayerTurn) {
      onSubmit();
    } else if (autoplay) {
      setAutoplay(false);
    } else {
      setAutoplay(true);
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
