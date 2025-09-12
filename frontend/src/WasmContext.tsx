import * as React from "react";

interface Context {
  decodeWireFormat: (req: Uint8Array) => any;
}

export const WasmContext = React.createContext<Context>({
  decodeWireFormat: (_) => {
    throw new Error("cannot decode wire format");
  },
});

export default WasmContext;
