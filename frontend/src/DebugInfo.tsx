import * as React from "react";
import { AppStateContext } from "./AppStateProvider";

export const DebugInfo = (_props: unknown): JSX.Element => {
  const appState = React.useContext(AppStateContext);

  return (
    <pre>
      {JSON.stringify(
        {
          gameState: appState.state.gameState,
          settings: appState.state.settings,
          roomName: appState.state.roomName,
        },
        null,
        2
      )}
    </pre>
  );
};

export default DebugInfo;
