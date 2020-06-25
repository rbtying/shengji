import * as React from "react";
import * as ReactDOM from "react-dom";
import * as ReactModal from "react-modal";

import AppStateProvider from "./AppStateProvider";
import WebsocketProvider from "./WebsocketProvider";
import TimerProvider from "./TimerProvider";
import Root from "./Root";

const bootstrap = (): void => {
  if (window.location.hash.length !== 17) {
    const arr = new Uint8Array(8);
    window.crypto.getRandomValues(arr);
    const r = Array.from(arr, (d) => ("0" + d.toString(16)).substr(-2)).join(
      ""
    );
    window.location.hash = r;
  }
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
