import * as React from "react";
import beep from "./beep";
import { TimerContext } from "./TimerProvider";

// Plays a beep sound as long as the component is mounted.
interface Props {
  beeper?: () => void;
  interval?: number;
}

const defaultBeeper = (): void => beep(3, 440, 200);

const Beeper = ({ beeper = defaultBeeper, interval = 5000 }: Props): null => {
  const { setInterval, clearInterval } = React.useContext(TimerContext);
  React.useEffect(() => {
    beeper();
    const timer = setInterval(beeper, interval);
    return () => clearInterval(timer);
  }, []);

  return null;
};

export default Beeper;
