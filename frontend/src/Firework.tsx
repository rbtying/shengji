import * as React from "react";
import { startFireworks } from "./startFireworks";

// Generate a firework for the duration
interface IProps {
  duration?: number;
}

const Firework = ({ duration = 10 }: IProps): null => {
  const [fireworkStarted, setFireworkStarted] = React.useState<boolean>(false);
  React.useEffect(() => {
    startFireworks(duration);
  }, [fireworkStarted]);

  if (!fireworkStarted) {
    setFireworkStarted(true);
  }
  return null;
};

export default Firework;
