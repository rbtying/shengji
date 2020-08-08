import { State, combineState } from "../State";
import {
  booleanLocalStorageState,
  JSONLocalStorageState,
} from "../localStorageState";

export interface Settings {
  fourColor: boolean;
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
const svgCards: State<boolean> = booleanLocalStorageState("svg_cards");
const showCardLabels: State<boolean> = booleanLocalStorageState(
  "show_card_labels"
);
const showLastTrick: State<boolean> = booleanLocalStorageState(
  "show_last_trick"
);
const beepOnTurn: State<boolean> = booleanLocalStorageState("beep_on_turn");
const reverseCardOrder: State<boolean> = booleanLocalStorageState(
  "reverse_card_order"
);
const unsetAutoPlayWhenWinnerChanges: State<boolean> = booleanLocalStorageState(
  "unset_autoplay_on_winner_change"
);
const showTrickInPlayerOrder: State<boolean> = booleanLocalStorageState(
  "show_trick_in_player_order"
);
const separateCardsBySuit: State<boolean> = booleanLocalStorageState(
  "separate_cards_by_suit"
);
const disableSuitHighlights: State<boolean> = booleanLocalStorageState(
  "disable_suit_highlights"
);
const suitColorOverrides: State<ISuitOverrides> = JSONLocalStorageState(
  "suit_color_overrides",
  {}
);
const playDrawCardSound: State<boolean> = booleanLocalStorageState(
  "play_draw_card_sound"
);
const settings: State<Settings> = combineState({
  fourColor,
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
});

export default settings;
