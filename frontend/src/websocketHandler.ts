import { AppState } from "./AppStateProvider";
import beep from "./beep";
import { IGameMessage, IGameMessageUnion } from "./types";

const truncate = (length: number) => <T>(array: T[]): T[] => {
  if (array.length > length) {
    return array.slice(array.length - length);
  } else {
    return array;
  }
};
const truncateMessages = truncate(100);

type WebsocketHandler = (
  state: AppState,
  message: IGameMessageUnion
) => Partial<AppState> | null;

const messageHandler: WebsocketHandler = (
  state: AppState,
  message: IGameMessageUnion
) => {
  if (message.Message !== undefined) {
    return { messages: truncateMessages([...state.messages, message.Message]) };
  } else {
    return null;
  }
};

const broadcastHandler: WebsocketHandler = (
  state: AppState,
  message: IGameMessageUnion
) => {
  if (message.Broadcast !== undefined) {
    const newMessage = {
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
  message: IGameMessageUnion
) => {
  if (message.Error !== undefined) {
    return { errors: [...state.errors, message.Error] };
  } else {
    return null;
  }
};

const stateHandler: WebsocketHandler = (
  _: AppState,
  message: IGameMessageUnion
) => {
  if (message.State !== undefined) {
    return { game_state: message.State.state, cards: message.State.cards };
  } else {
    return null;
  }
};

let lastBeeped = performance.now();
const beepHandler = (message: IGameMessage): void => {
  if (message === "Beep") {
    const now = performance.now();
    // Rate-limit beeps to prevent annoyance.
    if (now - lastBeeped >= 1000) {
      beep(3, 261.63, 200);
      lastBeeped = now;
    }
  }
  return null;
};

const gameFinishedHandler: WebsocketHandler = (
  state: AppState,
  message: IGameMessageUnion
) => {
  if (
    message.Broadcast !== undefined &&
    message.Broadcast.data.variant.type === "GameFinished"
  ) {
    if (state.name in message.Broadcast.data.variant.result) {
      const ownResult = message.Broadcast.data.variant.result[state.name];
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
      return { gameStatistics: newGameStatistics };
    }
  }
  return null;
};

const allHandlers: WebsocketHandler[] = [
  messageHandler,
  broadcastHandler,
  errorHandler,
  stateHandler,
  gameFinishedHandler,
];

const composedHandlers = (
  state: AppState,
  message: IGameMessage
): Partial<AppState> => {
  let partials = {};
  if (message !== "Beep") {
    allHandlers.forEach((h) => {
      const partial = h(state, message);
      partials = { ...partials, ...partial };
      state = { ...state, ...partial };
    });
  }
  beepHandler(message);
  return partials;
};

export default composedHandlers;
