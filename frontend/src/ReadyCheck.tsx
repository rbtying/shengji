import * as React from "react";
import { WebsocketContext } from "./WebsocketProvider";

const ReadyCheck = (): JSX.Element => {
  const { send } = React.useContext(WebsocketContext);

  return (
    <button
      className="big"
      onClick={() =>
        confirm("Are you ready to start the game?") && send("ReadyCheck")
      }
    >
      Check if everyone is ready!
    </button>
  );
};

export default ReadyCheck;
