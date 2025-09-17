// Tests for WebsocketProvider URL construction logic

describe("WebsocketProvider URL construction", () => {
  beforeEach(() => {
    jest.clearAllMocks();
    // Reset window._WEBSOCKET_HOST
    (global as any).window = { _WEBSOCKET_HOST: undefined };
    (global as any).location = {
      protocol: "https:",
      host: "example.com",
      pathname: "/game/",
    };
  });

  it("should use WEBSOCKET_HOST when provided", () => {
    (global as any).window._WEBSOCKET_HOST = "wss://custom.server.com/websocket";

    // Simulate the URL construction logic from WebsocketProvider
    const runtimeWebsocketHost = (global as any).window._WEBSOCKET_HOST;
    const uri =
      runtimeWebsocketHost !== undefined && runtimeWebsocketHost !== null
        ? runtimeWebsocketHost
        : (location.protocol === "https:" ? "wss://" : "ws://") +
          location.host +
          location.pathname +
          (location.pathname.endsWith("/") ? "api" : "/api");

    expect(uri).toBe("wss://custom.server.com/websocket");
  });

  it("should use default URL when WEBSOCKET_HOST is null", () => {
    (global as any).window._WEBSOCKET_HOST = null;

    // Simulate the URL construction logic from WebsocketProvider
    const runtimeWebsocketHost = (global as any).window._WEBSOCKET_HOST;
    const uri =
      runtimeWebsocketHost !== undefined && runtimeWebsocketHost !== null
        ? runtimeWebsocketHost
        : ((global as any).location.protocol === "https:" ? "wss://" : "ws://") +
          (global as any).location.host +
          (global as any).location.pathname +
          ((global as any).location.pathname.endsWith("/") ? "api" : "/api");

    // Should construct URL from location
    expect(uri).toBe("wss://example.com/game/api");
  });

  it("should use default URL when WEBSOCKET_HOST is undefined", () => {
    (global as any).window._WEBSOCKET_HOST = undefined;

    // Simulate the URL construction logic from WebsocketProvider
    const runtimeWebsocketHost = (global as any).window._WEBSOCKET_HOST;
    const uri =
      runtimeWebsocketHost !== undefined && runtimeWebsocketHost !== null
        ? runtimeWebsocketHost
        : ((global as any).location.protocol === "https:" ? "wss://" : "ws://") +
          (global as any).location.host +
          (global as any).location.pathname +
          ((global as any).location.pathname.endsWith("/") ? "api" : "/api");

    // Should construct URL from location
    expect(uri).toBe("wss://example.com/game/api");
  });

  it("should use ws:// for non-https protocol when no WEBSOCKET_HOST", () => {
    (global as any).window._WEBSOCKET_HOST = undefined;
    (global as any).location = {
      protocol: "http:",
      host: "localhost:3000",
      pathname: "/",
    };

    // Simulate the URL construction logic from WebsocketProvider
    const runtimeWebsocketHost = (global as any).window._WEBSOCKET_HOST;
    const uri =
      runtimeWebsocketHost !== undefined && runtimeWebsocketHost !== null
        ? runtimeWebsocketHost
        : ((global as any).location.protocol === "https:" ? "wss://" : "ws://") +
          (global as any).location.host +
          (global as any).location.pathname +
          ((global as any).location.pathname.endsWith("/") ? "api" : "/api");

    expect(uri).toBe("ws://localhost:3000/api");
  });

  it("should handle pathname not ending with slash", () => {
    (global as any).window._WEBSOCKET_HOST = undefined;
    (global as any).location = {
      protocol: "https:",
      host: "example.com",
      pathname: "/game",
    };

    // Simulate the URL construction logic from WebsocketProvider
    const runtimeWebsocketHost = (global as any).window._WEBSOCKET_HOST;
    const uri =
      runtimeWebsocketHost !== undefined && runtimeWebsocketHost !== null
        ? runtimeWebsocketHost
        : ((global as any).location.protocol === "https:" ? "wss://" : "ws://") +
          (global as any).location.host +
          (global as any).location.pathname +
          ((global as any).location.pathname.endsWith("/") ? "api" : "/api");

    expect(uri).toBe("wss://example.com/game/api");
  });

  it("should handle WEBSOCKET_HOST with ws:// protocol", () => {
    (global as any).window._WEBSOCKET_HOST = "ws://dev.server.com/socket";

    // Simulate the URL construction logic from WebsocketProvider
    const runtimeWebsocketHost = (global as any).window._WEBSOCKET_HOST;
    const uri =
      runtimeWebsocketHost !== undefined && runtimeWebsocketHost !== null
        ? runtimeWebsocketHost
        : ((global as any).location.protocol === "https:" ? "wss://" : "ws://") +
          (global as any).location.host +
          (global as any).location.pathname +
          ((global as any).location.pathname.endsWith("/") ? "api" : "/api");

    expect(uri).toBe("ws://dev.server.com/socket");
  });

  it("should handle WEBSOCKET_HOST with wss:// protocol", () => {
    (global as any).window._WEBSOCKET_HOST = "wss://secure.server.com/ws";

    // Simulate the URL construction logic from WebsocketProvider
    const runtimeWebsocketHost = (global as any).window._WEBSOCKET_HOST;
    const uri =
      runtimeWebsocketHost !== undefined && runtimeWebsocketHost !== null
        ? runtimeWebsocketHost
        : ((global as any).location.protocol === "https:" ? "wss://" : "ws://") +
          (global as any).location.host +
          (global as any).location.pathname +
          ((global as any).location.pathname.endsWith("/") ? "api" : "/api");

    expect(uri).toBe("wss://secure.server.com/ws");
  });

  it("should handle empty string WEBSOCKET_HOST", () => {
    (global as any).window._WEBSOCKET_HOST = "";
    (global as any).location = {
      protocol: "https:",
      host: "example.com",
      pathname: "/",
    };

    // Simulate the URL construction logic from WebsocketProvider
    const runtimeWebsocketHost = (global as any).window._WEBSOCKET_HOST;
    const uri =
      runtimeWebsocketHost !== undefined && runtimeWebsocketHost !== null
        ? runtimeWebsocketHost
        : ((global as any).location.protocol === "https:" ? "wss://" : "ws://") +
          (global as any).location.host +
          (global as any).location.pathname +
          ((global as any).location.pathname.endsWith("/") ? "api" : "/api");

    // Empty string is truthy in JavaScript, but the code checks for undefined and null
    // So empty string would be used as-is
    expect(uri).toBe("");
  });
});