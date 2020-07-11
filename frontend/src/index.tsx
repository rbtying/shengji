import * as React from "react";
import * as ReactDOM from "react-dom";
import * as ReactModal from "react-modal";

import AppStateProvider from "./AppStateProvider";
import WebsocketProvider from "./WebsocketProvider";
import TimerProvider from "./TimerProvider";
import Root from "./Root";

const bootstrap = (): void => {
  const root = document.getElementById("root");
  ReactModal.setAppElement(root);
  ReactDOM.render(
    <TimerProvider>
      <AppStateProvider>
        <WebsocketProvider>
          <Root />
        </WebsocketProvider>
      </AppStateProvider>
    </TimerProvider>,
    root
  );
};

bootstrap();
