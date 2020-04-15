import {State, combineState} from '../State';
import {booleanLocalStorageState} from '../localStorageState';

export type Settings = {
  fourColor: boolean;
  showLastTrick: boolean;
  beepOnTurn: boolean;
  reverseCardOrder: boolean;
  unsetAutoPlayWhenWinnerChanges: boolean;
  showTrickInPlayerOrder: boolean;
};

const fourColor: State<boolean> = booleanLocalStorageState('four_color');
const showLastTrick: State<boolean> = booleanLocalStorageState(
  'show_last_trick',
);
const beepOnTurn: State<boolean> = booleanLocalStorageState('beep_on_turn');
const reverseCardOrder: State<boolean> = booleanLocalStorageState(
  'reverse_card_order',
);
const unsetAutoPlayWhenWinnerChanges: State<boolean> = booleanLocalStorageState(
  'unset_autoplay_on_winner_change',
);
const showTrickInPlayerOrder: State<boolean> = booleanLocalStorageState(
  'show_trick_in_player_order',
);

const settings: State<Settings> = combineState({
  fourColor,
  showLastTrick,
  beepOnTurn,
  reverseCardOrder,
  unsetAutoPlayWhenWinnerChanges,
  showTrickInPlayerOrder,
});

export default settings;
