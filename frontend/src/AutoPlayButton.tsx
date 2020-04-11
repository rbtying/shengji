import * as React from 'react';

type Props = {
  onSubmit: () => void;
  canSubmit: boolean;
  isCurrentPlayerTurn: boolean;
};

const AutoPlayButton = (props: Props) => {
  const {onSubmit, canSubmit, isCurrentPlayerTurn} = props;
  const [autoplay, setAutoplay] = React.useState<boolean>(false);

  React.useEffect(() => {
    if (autoplay && isCurrentPlayerTurn) {
      setAutoplay(false);
      onSubmit();
    }
  }, [autoplay, isCurrentPlayerTurn]);

  React.useEffect(() => {
    if (!canSubmit) {
      setAutoplay(false);
    }
  }, [canSubmit]);

  const handleClick = () => {
    if (isCurrentPlayerTurn) {
      onSubmit();
    } else {
      setAutoplay(!autoplay);
    }
  };
  return (
    <button onClick={handleClick} disabled={!canSubmit}>
      {isCurrentPlayerTurn
        ? 'Play selected cards'
        : autoplay
        ? 'Undo autoplay selected cards'
        : 'Autoplay selected cards'}
    </button>
  );
};

export default AutoPlayButton;
