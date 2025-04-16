import { AppState } from "./AppStateProvider";
import beep from "./beep";
import { GameMessage } from "./gen-types";
import { Message } from "./ChatMessage";

const truncate =
  (length: number) =>
  <T>(array: T[]): T[] => {
    if (array.length > length) {
      return array.slice(array.length - length);
    } else {
      return array;
    }
  };
const truncateMessages = truncate(300);

type WebsocketHandler = (
  state: AppState,
  message: GameMessage,
) => Partial<AppState> | null;

const messageHandler: WebsocketHandler = (
  state: AppState,
  message: GameMessage,
) => {
  if ("Message" in message) {
    return { messages: truncateMessages([...state.messages, message.Message]) };
  } else {
    return null;
  }
};

const broadcastHandler: WebsocketHandler = (
  state: AppState,
  message: GameMessage,
) => {
  if ("Broadcast" in message) {
    const newMessage: Message = {
      from: "GAME",
      message: message.Broadcast.message,
      data: message.Broadcast.data,
      from_game: true,
    };
    return { messages: truncateMessages([...state.messages, newMessage]) };
  } else {
    return null;
  }
};

const errorHandler: WebsocketHandler = (
  state: AppState,
  message: GameMessage,
) => {
  if ("Error" in message) {
    const errorMsg = message.Error;
    const isJoinError = errorMsg.includes("Maximum number of players reached");
    return {
      errors: [...state.errors, errorMsg],
      gameState: null,
      joinError: isJoinError ? errorMsg : state.joinError,
      connectionNonce: isJoinError ? state.connectionNonce + 1 : state.connectionNonce,
    };
  } else {
    return null;
  }
};

const stateHandler: WebsocketHandler = (_: AppState, message: GameMessage) => {
  if ("State" in message) {
    return { gameState: message.State.state };
  } else {
    return null;
  }
};

const headerMessageHandler: WebsocketHandler = (
  _: AppState,
  message: GameMessage,
) => {
  if ("Header" in message) {
    return { headerMessages: message.Header.messages };
  } else {
    return null;
  }
};

let lastBeeped = performance.now();
const beepHandler = (message: GameMessage): void => {
  if ("Beep" in message) {
    const now = performance.now();
    // Rate-limit beeps to prevent annoyance.
    if (now - lastBeeped >= 1000) {
      beep(3, 261.63, 200);
      lastBeeped = now;
    }
  }
};

let lastReadyChecked = performance.now();
const readyCheckHandler = (
  message: GameMessage,
  send: (msg: any) => void,
): void => {
  if ("ReadyCheck" in message) {
    const now = performance.now();
    // Rate-limit beeps to prevent annoyance.
    if (now - lastReadyChecked >= 1000) {
      beep(3, 261.63, 200);
      lastReadyChecked = now;
      if (confirm("Are you ready to start the game?")) {
        send("Ready");
      }
    }
  }
};

const gameFinishedHandler: WebsocketHandler = (
  state: AppState,
  message: GameMessage,
) => {
  if (
    "Broadcast" in message &&
    message.Broadcast.data.variant.type === "GameFinished"
  ) {
    const result = message.Broadcast.data.variant.result;
    const updates: Partial<AppState> = {};
    if (state.name in result) {
      const ownResult = result[state.name];
      const gameStatistics = state.gameStatistics;

      const newGameStatistics = { ...gameStatistics };
      newGameStatistics.gamesPlayed++;
      if (ownResult.is_defending) {
        newGameStatistics.gamesPlayedAsDefending++;
        if (ownResult.is_landlord) {
          newGameStatistics.gamesPlayedAsLandlord++;
        }
      }

      if (ownResult.won_game) {
        newGameStatistics.gamesWon++;
        if (ownResult.is_defending) {
          newGameStatistics.gamesWonAsDefending++;
          if (ownResult.is_landlord) {
            newGameStatistics.gamesWonAsLandlord++;
          }
        }
      }

      newGameStatistics.ranksUp += ownResult.ranks_up;
      updates.gameStatistics = newGameStatistics;
    }
    const gameWinners = Object.entries(result)
      .filter((r) => r[1].confetti)
      .map((r) => r[0]);
    if (gameWinners.length > 0) {
      const group = gameWinners
        .join(", ")
        .replace(/, ((?:.(?!, ))+)$/, " and $1");
      updates.confetti = `${group} successfully defended on the final level!`;
    }
    if (
      updates.gameStatistics !== undefined ||
      updates.confetti !== undefined
    ) {
      return updates;
    }
  }
  return null;
};

const allHandlers: WebsocketHandler[] = [
  messageHandler,
  broadcastHandler,
  errorHandler,
  stateHandler,
  headerMessageHandler,
  gameFinishedHandler,
];

const composedHandlers = (
  state: AppState,
  message: GameMessage,
  send: (msg: any) => void,
): Partial<AppState> => {
  let partials = {};
  allHandlers.forEach((h) => {
    const partial = h(state, message);
    partials = { ...partials, ...partial };
    state = { ...state, ...partial };
  });
  beepHandler(message);
  readyCheckHandler(message, send);
  return partials;
};

export default composedHandlers;
