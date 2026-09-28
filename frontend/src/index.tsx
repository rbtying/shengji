import { createRoot } from "react-dom/client";
import * as React from "react";
import ReactModal from "react-modal";

import "./style.css";

import AppStateProvider from "./AppStateProvider";
import ErrorBoundary from "./ErrorBoundary";
import WebsocketProvider from "./WebsocketProvider";
import OfflineProvider, { isOffline } from "./OfflineProvider";
import TimerProvider from "./TimerProvider";
import Root from "./Root";
import AppRoot from "./app/AppRoot";

const WasmProvider = React.lazy(
  async () => await import("./WasmOrRpcProvider"),
);

const bootstrap = (): void => {
  const root = document.getElementById("root");
  const fallback = (
    <>
      An error has occured, please try refreshing! If that doesn&apos;t resolve
      the issue, consider using the latest version of Mozilla Firefox or Google
      Chrome browsers.
    </>
  );
  ReactModal.setAppElement(root!);
  const root_ = createRoot(root!);

  const Connection = isOffline() ? OfflineProvider : WebsocketProvider;

  root_.render(
    <ErrorBoundary fallback={fallback}>
      <React.Suspense fallback={"loading..."}>
        <WasmProvider>
          <TimerProvider>
            <AppStateProvider>
              <Connection>
                <ErrorBoundary fallback={fallback}>
                  {isOffline() ? <AppRoot /> : <Root />}
                </ErrorBoundary>
              </Connection>
            </AppStateProvider>
          </TimerProvider>
        </WasmProvider>
      </React.Suspense>
    </ErrorBoundary>,
  );
};

bootstrap();
