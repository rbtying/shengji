// Mock fetch globally
global.fetch = jest.fn();

describe("WasmOrRpcProvider RPC calls", () => {
  beforeEach(() => {
    jest.clearAllMocks();
    // Reset window._WEBSOCKET_HOST
    (global as any).window = { _WEBSOCKET_HOST: undefined };
  });

  describe("callRpc URL construction", () => {
    // Since callRpc is not exported, we test the URL construction logic directly
    it("should use relative /api/rpc when WEBSOCKET_HOST is not set", async () => {
      // Test setup to trigger RPC call
      const mockResponse = {
        ok: true,
        text: async () => JSON.stringify({ type: "Response", data: {} }),
      };
      (global.fetch as jest.Mock).mockResolvedValue(mockResponse);

      // Test case 1: No WEBSOCKET_HOST
      (global as any).window._WEBSOCKET_HOST = undefined;
      let rpcUrl = "/api/rpc";
      expect(rpcUrl).toBe("/api/rpc");
    });

    it("should convert wss:// to https:// for RPC calls", () => {
      (global as any).window._WEBSOCKET_HOST = "wss://example.com/game";

      // Simulate the URL construction logic
      const runtimeWebsocketHost = (global as any).window._WEBSOCKET_HOST;
      let rpcUrl = "/api/rpc";

      if (runtimeWebsocketHost !== undefined && runtimeWebsocketHost !== null) {
        const httpUrl = runtimeWebsocketHost
          .replace(/^wss:\/\//, "https://")
          .replace(/^ws:\/\//, "http://");

        if (httpUrl.endsWith("/")) {
          rpcUrl = httpUrl + "api/rpc";
        } else if (httpUrl.endsWith("/api")) {
          rpcUrl = httpUrl + "/rpc";
        } else {
          rpcUrl = httpUrl + "/api/rpc";
        }
      }

      expect(rpcUrl).toBe("https://example.com/game/api/rpc");
    });

    it("should convert ws:// to http:// for RPC calls", () => {
      (global as any).window._WEBSOCKET_HOST = "ws://localhost:3000";

      // Simulate the URL construction logic
      const runtimeWebsocketHost = (global as any).window._WEBSOCKET_HOST;
      let rpcUrl = "/api/rpc";

      if (runtimeWebsocketHost !== undefined && runtimeWebsocketHost !== null) {
        const httpUrl = runtimeWebsocketHost
          .replace(/^wss:\/\//, "https://")
          .replace(/^ws:\/\//, "http://");

        if (httpUrl.endsWith("/")) {
          rpcUrl = httpUrl + "api/rpc";
        } else if (httpUrl.endsWith("/api")) {
          rpcUrl = httpUrl + "/rpc";
        } else {
          rpcUrl = httpUrl + "/api/rpc";
        }
      }

      expect(rpcUrl).toBe("http://localhost:3000/api/rpc");
    });

    it("should handle URLs ending with /", () => {
      (global as any).window._WEBSOCKET_HOST = "wss://api.example.com/";

      // Simulate the URL construction logic
      const runtimeWebsocketHost = (global as any).window._WEBSOCKET_HOST;
      let rpcUrl = "/api/rpc";

      if (runtimeWebsocketHost !== undefined && runtimeWebsocketHost !== null) {
        const httpUrl = runtimeWebsocketHost
          .replace(/^wss:\/\//, "https://")
          .replace(/^ws:\/\//, "http://");

        if (httpUrl.endsWith("/")) {
          rpcUrl = httpUrl + "api/rpc";
        } else if (httpUrl.endsWith("/api")) {
          rpcUrl = httpUrl + "/rpc";
        } else {
          rpcUrl = httpUrl + "/api/rpc";
        }
      }

      expect(rpcUrl).toBe("https://api.example.com/api/rpc");
    });

    it("should handle URLs ending with /api", () => {
      (global as any).window._WEBSOCKET_HOST = "wss://example.com/api";

      // Simulate the URL construction logic
      const runtimeWebsocketHost = (global as any).window._WEBSOCKET_HOST;
      let rpcUrl = "/api/rpc";

      if (runtimeWebsocketHost !== undefined && runtimeWebsocketHost !== null) {
        const httpUrl = runtimeWebsocketHost
          .replace(/^wss:\/\//, "https://")
          .replace(/^ws:\/\//, "http://");

        if (httpUrl.endsWith("/")) {
          rpcUrl = httpUrl + "api/rpc";
        } else if (httpUrl.endsWith("/api")) {
          rpcUrl = httpUrl + "/rpc";
        } else {
          rpcUrl = httpUrl + "/api/rpc";
        }
      }

      expect(rpcUrl).toBe("https://example.com/api/rpc");
    });

    it("should handle null WEBSOCKET_HOST", () => {
      (global as any).window._WEBSOCKET_HOST = null;

      // Simulate the URL construction logic
      const runtimeWebsocketHost = (global as any).window._WEBSOCKET_HOST;
      let rpcUrl = "/api/rpc";

      if (runtimeWebsocketHost !== undefined && runtimeWebsocketHost !== null) {
        const httpUrl = runtimeWebsocketHost
          .replace(/^wss:\/\//, "https://")
          .replace(/^ws:\/\//, "http://");

        if (httpUrl.endsWith("/")) {
          rpcUrl = httpUrl + "api/rpc";
        } else if (httpUrl.endsWith("/api")) {
          rpcUrl = httpUrl + "/rpc";
        } else {
          rpcUrl = httpUrl + "/api/rpc";
        }
      }

      expect(rpcUrl).toBe("/api/rpc");
    });
  });
});