import {State, combineState} from '../State';
import {booleanLocalStorageState} from '../localStorageState';

export type Settings = {
  fourColor: boolean;
  showLastTrick: boolean;
  beepOnTurn: boolean;
  reverseCardOrder: boolean;
};

const fourColor: State<boolean> = booleanLocalStorageState('four_color');
const showLastTrick: State<boolean> = booleanLocalStorageState(
  'show_last_trick',
);
const beepOnTurn: State<boolean> = booleanLocalStorageState('beep_on_turn');
const reverseCardOrder: State<boolean> = booleanLocalStorageState(
  'reverse_card_order',
);

const settings: State<Settings> = combineState({
  fourColor,
  showLastTrick,
  beepOnTurn,
  reverseCardOrder,
});

export default settings;
