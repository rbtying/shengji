import { State, combineState } from "../State";
import {
  booleanLocalStorageState,
  JSONLocalStorageState,
  numberLocalStorageState,
  stringLocalStorageState,
} from "../localStorageState";

export interface Settings {
  fourColor: boolean;
  darkMode: boolean;
  showCardLabels: boolean;
  showLastTrick: boolean;
  beepOnTurn: boolean;
  reverseCardOrder: boolean;
  unsetAutoPlayWhenWinnerChanges: boolean;
  showTrickInPlayerOrder: boolean;
  separateCardsBySuit: boolean;
  disableSuitHighlights: boolean;
  svgCards: boolean;
  playDrawCardSound: boolean;
  suitColorOverrides: ISuitOverrides;
  showDebugInfo: boolean;
  showPlayerName: boolean;
  hideChatBox: boolean;
  showPointsAboveGame: boolean;
  pointCardIcon: string;
  trumpCardIcon: string;
  autodrawSpeedMs: number | null;
}

export interface ISuitOverrides {
  "♢"?: string;
  "♡"?: string;
  "♤"?: string;
  "♧"?: string;
  "🃟"?: string;
  "🃏"?: string;
  "🂠"?: string;
}

const fourColor: State<boolean> = booleanLocalStorageState("four_color");
const darkMode: State<boolean> = booleanLocalStorageState("dark_mode");
const svgCards: State<boolean> = booleanLocalStorageState("svg_cards");
const showCardLabels: State<boolean> =
  booleanLocalStorageState("show_card_labels");
const showLastTrick: State<boolean> =
  booleanLocalStorageState("show_last_trick");
const beepOnTurn: State<boolean> = booleanLocalStorageState("beep_on_turn");
const reverseCardOrder: State<boolean> =
  booleanLocalStorageState("reverse_card_order");
const unsetAutoPlayWhenWinnerChanges: State<boolean> = booleanLocalStorageState(
  "unset_autoplay_on_winner_change",
);
const showTrickInPlayerOrder: State<boolean> = booleanLocalStorageState(
  "show_trick_in_player_order",
  true,
);
const separateCardsBySuit: State<boolean> = booleanLocalStorageState(
  "separate_cards_by_suit",
);
const disableSuitHighlights: State<boolean> = booleanLocalStorageState(
  "disable_suit_highlights",
);
const suitColorOverrides: State<ISuitOverrides> = JSONLocalStorageState(
  "suit_color_overrides",
  {},
);
const playDrawCardSound: State<boolean> = booleanLocalStorageState(
  "play_draw_card_sound",
);
const showDebugInfo: State<boolean> =
  booleanLocalStorageState("show_debug_info");
const showPlayerName: State<boolean> = booleanLocalStorageState(
  "show_player_name_in_title",
);
const hideChatBox: State<boolean> = booleanLocalStorageState("hide_chat_box");
const showPointsAboveGame: State<boolean> =
  booleanLocalStorageState("points_above_game");
export const DEFAULT_POINT_CARD_ICON: string = "💰";
const pointCardIcon: State<string> = stringLocalStorageState(
  "point_card_icon",
  DEFAULT_POINT_CARD_ICON,
);
export const DEFAULT_TRUMP_CARD_ICON: string = "👑";
const trumpCardIcon: State<string> = stringLocalStorageState(
  "trump_card_icon",
  DEFAULT_TRUMP_CARD_ICON,
);
const autodrawSpeedMs: State<number | null> = numberLocalStorageState(
  "autodrawSpeedMs",
  null,
);
const settings: State<Settings> = combineState({
  fourColor,
  darkMode,
  showCardLabels,
  showLastTrick,
  beepOnTurn,
  reverseCardOrder,
  unsetAutoPlayWhenWinnerChanges,
  showTrickInPlayerOrder,
  svgCards,
  separateCardsBySuit,
  disableSuitHighlights,
  suitColorOverrides,
  playDrawCardSound,
  showDebugInfo,
  showPlayerName,
  hideChatBox,
  showPointsAboveGame,
  pointCardIcon,
  trumpCardIcon,
  autodrawSpeedMs,
});

export default settings;
