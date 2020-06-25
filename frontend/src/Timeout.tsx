import * as React from "react";
import { TimerContext } from "./TimerProvider";

interface Props {
  timeout: number;
  callback: () => void;
}

const Timeout = (props: Props): null => {
  const { setTimeout, clearTimeout } = React.useContext(TimerContext);
  React.useEffect(() => {
    const timeout = setTimeout(props.callback, props.timeout);
    return () => clearTimeout(timeout);
  });

  return null;
};

export default Timeout;
