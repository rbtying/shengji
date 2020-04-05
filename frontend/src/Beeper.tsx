import * as React from 'react';
import beep from './beep';

// Plays a beep sound as long as the component is mounted.
type Props = {
  beeper?: () => void;
  interval?: number;
};

const defaultBeeper = () => beep(3, 440, 200);

const Beeper = ({beeper = defaultBeeper, interval = 5000}: Props): null => {
  React.useEffect(() => {
    beeper();
    const timer = window.setInterval(beeper, interval);
    return () => window.clearInterval(timer);
  });

  return null;
};

export default Beeper;
