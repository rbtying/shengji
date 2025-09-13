export function isWasmAvailable(): boolean {
  // Check for URL parameter to force no-WASM mode
  const urlParams = new URLSearchParams(window.location.search);
  if (urlParams.get("no-wasm") === "true") {
    console.log("No-WASM mode enabled via URL parameter");
    return false;
  }

  try {
    if (
      typeof WebAssembly === "object" &&
      typeof WebAssembly.instantiate === "function"
    ) {
      const module = new WebAssembly.Module(
        Uint8Array.of(0x0, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00),
      );
      if (module instanceof WebAssembly.Module) {
        return new WebAssembly.Instance(module) instanceof WebAssembly.Instance;
      }
    }
  } catch (e) {
    console.warn("WebAssembly not available:", e);
  }
  return false;
}
