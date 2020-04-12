import * as React from 'react';
import beep from './beep';
import {TimerConsumer} from './TimerProvider';

// Plays a beep sound as long as the component is mounted.
type Props = {
  beeper?: () => void;
  interval?: number;
};

type InnerProps = {
  beeper: () => void;
  interval: number;
  setInterval: (fn: () => void, interval: number) => number;
  clearInterval: (id: number) => void;
};

const defaultBeeper = () => beep(3, 440, 200);

const _Beeper = ({
  beeper,
  interval,
  setInterval,
  clearInterval,
}: InnerProps): null => {
  React.useEffect(() => {
    beeper();
    const timer = setInterval(beeper, interval);
    return () => clearInterval(timer);
  }, []);

  return null;
};

const Beeper = ({beeper = defaultBeeper, interval = 5000}: Props) => (
  <TimerConsumer>
    {({setInterval, clearInterval}) => (
      <_Beeper
        beeper={beeper}
        interval={interval}
        setInterval={setInterval}
        clearInterval={clearInterval}
      />
    )}
  </TimerConsumer>
);

export default Beeper;
