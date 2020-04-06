import {AppState} from './AppStateProvider';

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

const messageHandler: WebsocketHandler = (state: AppState, message: any) => {
  if (message.Message) {
    return {messages: truncateMessages([...state.messages, message.Message])};
  } else {
    return null;
  }
};

const broadcastHandler: WebsocketHandler = (state: AppState, message: any) => {
  if (message.Broadcast) {
    const newMessage = {
      from: 'GAME',
      message: message.Broadcast,
      from_game: true,
    };
    return {messages: truncateMessages([...state.messages, newMessage])};
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

const compose = (
  left: WebsocketHandler,
  right: WebsocketHandler,
): WebsocketHandler => {
  return (state: AppState, message: any) => {
    const newState = {...state, ...left(state, message)};
    return {...newState, ...right(newState, message)};
  };
};

const allHandlers: WebsocketHandler[] = [
  messageHandler,
  broadcastHandler,
  errorHandler,
  stateHandler,
];

export default allHandlers.reduce(compose) as WebsocketHandler;
