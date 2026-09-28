import { createRoot } from "react-dom/client";
import * as React from "react";
import ReactModal from "react-modal";
import * as Sentry from "@sentry/react";

import "./style.css";

import AppStateProvider from "./AppStateProvider";
import WebsocketProvider from "./WebsocketProvider";
import OfflineProvider, { isOffline } from "./OfflineProvider";
import TimerProvider from "./TimerProvider";
import Root from "./Root";

const WasmProvider = React.lazy(
  async () => await import("./WasmOrRpcProvider"),
);

const bootstrap = (): void => {
  // Error reporting to the original project's Sentry account is disabled in this fork.

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
    <Sentry.ErrorBoundary fallback={fallback}>
      <React.Suspense fallback={"loading..."}>
        <WasmProvider>
          <TimerProvider>
            <AppStateProvider>
              <Connection>
                <Sentry.ErrorBoundary fallback={fallback}>
                  <Root />
                </Sentry.ErrorBoundary>
              </Connection>
            </AppStateProvider>
          </TimerProvider>
        </WasmProvider>
      </React.Suspense>
    </Sentry.ErrorBoundary>,
  );
};

bootstrap();
