import {AppState} from './AppStateProvider';
import convertApiUnion from './util/convertApiUnion';

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
  message: any,
) => Partial<AppState> | null;

const transformMessage = (rawMessage: any) => {
  const data = rawMessage.data
    ? {...rawMessage.data, variant: convertApiUnion(rawMessage.data.variant)}
    : rawMessage.data;
  return {...rawMessage, data};
};
const messageHandler: WebsocketHandler = (state: AppState, message: any) => {
  if (message.Message) {
    return {
      messages: truncateMessages([
        ...state.messages,
        transformMessage(message.Message),
      ]),
    };
  } else {
    return null;
  }
};

const broadcastHandler: WebsocketHandler = (state: AppState, message: any) => {
  if (message.Broadcast) {
    const newMessage = {
      from: 'GAME',
      message: message.Broadcast.message,
      data: message.Broadcast.data,
      from_game: true,
    };
    return {
      messages: truncateMessages([
        ...state.messages,
        transformMessage(newMessage),
      ]),
    };
  } else {
    return null;
  }
};

const errorHandler: WebsocketHandler = (state: AppState, message: any) => {
  if (message.Error) {
    return {errors: [...state.errors, message.Error]};
  } else {
    return null;
  }
};

const stateHandler: WebsocketHandler = (state: AppState, message: any) => {
  if (message.State) {
    return {game_state: message.State.state, cards: message.State.cards};
  } else {
    return null;
  }
};

const allHandlers: WebsocketHandler[] = [
  messageHandler,
  broadcastHandler,
  errorHandler,
  stateHandler,
];

const composedHandlers: WebsocketHandler = (state: AppState, message: any) => {
  let partials = {};
  allHandlers.forEach((h) => {
    const partial = h(state, message);
    partials = {...partials, ...partial};
    state = {...state, ...partial};
  });
  return partials;
};

export default composedHandlers as WebsocketHandler;
