import * as React from "react";
import * as ReactDOM from "react-dom";
import * as ReactModal from "react-modal";
import * as Sentry from "@sentry/react";

import "./style.css";

import AppStateProvider from "./AppStateProvider";
import WebsocketProvider from "./WebsocketProvider";
import TimerProvider from "./TimerProvider";
import Root from "./Root";

const WasmProvider = React.lazy(async () => await import("./WasmProvider"));

const bootstrap = (): void => {
  Sentry.init({
    dsn:
      "https://dfdd871554eb4ab48de73a6575c1117a@o476591.ingest.sentry.io/5516535",
    release: (window as any)._VERSION,
  });

  const root = document.getElementById("root");
  ReactModal.setAppElement(root);
  ReactDOM.render(
    <Sentry.ErrorBoundary
      fallback={"An error has occured, please try refreshing!"}
    >
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
      </React.Suspense>
    </Sentry.ErrorBoundary>,
    root
  );
};

bootstrap();
