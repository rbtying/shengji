import * as React from "react";
import { EngineContext } from "./WasmOrRpcProvider";

export function useEngine() {
  const context = React.useContext(EngineContext);
  if (!context) {
    throw new Error("useEngine must be used within a WasmOrRpcProvider");
  }
  return context;
}
