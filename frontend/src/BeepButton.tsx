import * as React from "react";
import { WebsocketContext } from "./WebsocketProvider";

import type { JSX } from "react";

const BeepButton = (): JSX.Element => {
  const { send } = React.useContext(WebsocketContext);

  return (
    <button
      className="big"
      onClick={() =>
        confirm("Do you really want to send a beep to the current player?") &&
        send("Beep")
      }
    >
      🛎️
    </button>
  );
};

export default BeepButton;
