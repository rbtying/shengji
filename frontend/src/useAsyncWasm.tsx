import * as React from "react";
import { AsyncWasmContext } from "./WasmOrRpcProvider";

export function useAsyncWasm() {
  const context = React.useContext(AsyncWasmContext);
  if (!context) {
    throw new Error("useAsyncWasm must be used within a WasmOrRpcProvider");
  }
  return context;
}