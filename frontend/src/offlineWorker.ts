// Runs an offline game (engine + computer players) off the main thread, so the page stays
// responsive while bots think.
//
// Main thread -> worker: { type: "init", snapshot: string | null } then { type: "send", payload }.
// Worker -> main thread: { type: "messages", messages: GameMessage[] } and
// { type: "snapshot", snapshot: string } after changes, for saving.

type OfflineGame = import("../shengji-wasm/pkg/shengji-core.js").OfflineGame;

let game: OfflineGame | null = null;
let timer: ReturnType<typeof setTimeout> | null = null;
let idleTimer: ReturnType<typeof setTimeout> | null = null;

// While nothing is due soon (usually: the player is thinking), analyze earlier plays for the
// after-hand review in short slices, so the review is ready when the hand ends.
const IDLE_SLICE_MS = 450;
const scheduleIdle = (): void => {
  if (game === null || idleTimer !== null || !game.has_idle_work()) {
    return;
  }
  const wake = game.next_wake();
  if (wake !== undefined && wake !== null && wake - performance.now() < 1200) {
    return;
  }
  idleTimer = setTimeout(() => {
    idleTimer = null;
    const wake2 = game?.next_wake();
    if (
      game !== null &&
      (wake2 === undefined ||
        wake2 === null ||
        wake2 - performance.now() >= 1200)
    ) {
      game.idle_work(IDLE_SLICE_MS);
    }
    scheduleIdle();
  }, 250);
};
const queue: any[] = [];

const ctx = self as unknown as {
  postMessage: (msg: any) => void;
  onmessage: ((e: MessageEvent) => void) | null;
};

const emit = (json: string): void => {
  const messages = JSON.parse(json);
  if (messages.length > 0) {
    ctx.postMessage({ type: "messages", messages });
    const snapshot = game?.snapshot();
    if (snapshot !== undefined && snapshot !== null) {
      ctx.postMessage({ type: "snapshot", snapshot });
    }
  }
};

const schedule = (): void => {
  if (game === null) {
    return;
  }
  if (timer !== null) {
    clearTimeout(timer);
    timer = null;
  }
  const wake = game.next_wake();
  if (wake === undefined || wake === null) {
    scheduleIdle();
    return;
  }
  timer = setTimeout(run, Math.max(0, wake - performance.now()));
  scheduleIdle();
};

const run = (): void => {
  timer = null;
  if (game !== null) {
    emit(game.tick(performance.now()));
  }
  schedule();
};

const handle = (data: any): void => {
  if (game === null) {
    queue.push(data);
    return;
  }
  if (data.type === "send") {
    emit(game.handle(JSON.stringify(data.payload)));
    schedule();
  }
};

ctx.onmessage = (e: MessageEvent) => {
  const data = e.data;
  if (data.type === "init") {
    import("../shengji-wasm/pkg/shengji-core.js").then((wasm) => {
      let g: OfflineGame | null = null;
      if (data.snapshot) {
        try {
          g = wasm.OfflineGame.restore(data.snapshot);
        } catch (err) {
          console.error("Couldn't resume the saved offline game", err);
        }
      }
      game = g ?? new wasm.OfflineGame();
      emit(game.view());
      queue.splice(0).forEach(handle);
      schedule();
    });
  } else {
    handle(data);
  }
};
