import * as React from "react";
import * as ReactDOM from "react-dom";
import * as ReactModal from "react-modal";
import "./style.css";

import AppStateProvider from "./AppStateProvider";
import WebsocketProvider from "./WebsocketProvider";
import TimerProvider from "./TimerProvider";
import Root from "./Root";

const WasmProvider = React.lazy(async () => await import("./WasmProvider"));

const bootstrap = (): void => {
  const root = document.getElementById("root");
  ReactModal.setAppElement(root);
  ReactDOM.render(
    <React.Suspense fallback={"loading..."}>
      <WasmProvider>
        <TimerProvider>
          <AppStateProvider>
            <WebsocketProvider>
              <Root />
            </WebsocketProvider>
          </AppStateProvider>
        </TimerProvider>
      </WasmProvider>
    </React.Suspense>,
    root
  );
};

bootstrap();
