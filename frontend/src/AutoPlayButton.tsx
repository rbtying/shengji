import * as React from 'react';

type Props = {
  onSubmit: () => void;
  canSubmit: boolean;
  currentWinner: number | null;
  isCurrentPlayerTurn: boolean;
};

type State = {
  autoplay: boolean;
  lastWinner: number | null;
};

const AutoPlayButton = (props: Props) => {
  const {onSubmit, canSubmit, isCurrentPlayerTurn, currentWinner} = props;
  const [autoplay, setAutoplay] = React.useState<State>({
    autoplay: false,
    lastWinner: props.currentWinner,
  });

  React.useEffect(() => {
    if (
      autoplay.autoplay &&
      autoplay.lastWinner === currentWinner &&
      isCurrentPlayerTurn
    ) {
      setAutoplay({
        autoplay: false,
        lastWinner: null,
      });
      onSubmit();
    }
  }, [autoplay, isCurrentPlayerTurn, currentWinner]);

  React.useEffect(() => {
    if (
      !canSubmit ||
      (autoplay.autoplay && autoplay.lastWinner !== currentWinner)
    ) {
      setAutoplay({
        autoplay: false,
        lastWinner: null,
      });
    }
  }, [canSubmit, currentWinner]);

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
