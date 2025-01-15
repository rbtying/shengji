import { State, combineState } from "../State";
import { numberLocalStorageState } from "../localStorageState";

export interface GameStatistics {
  gamesPlayed: number;
  gamesPlayedAsDefending: number;
  gamesPlayedAsLandlord: number;
  gamesWon: number;
  gamesWonAsDefending: number;
  gamesWonAsLandlord: number;
  ranksUp: number;
}

const gamesPlayed: State<number> = numberLocalStorageState("games_played");
const gamesPlayedAsDefending: State<number> = numberLocalStorageState(
  "games_played_as_defending"
);
const gamesPlayedAsLandlord: State<number> = numberLocalStorageState(
  "games_played_as_landlord"
);
const gamesWon: State<number> = numberLocalStorageState("games_won");
const gamesWonAsDefending: State<number> = numberLocalStorageState(
  "games_won_as_defending"
);
const gamesWonAsLandlord: State<number> = numberLocalStorageState(
  "games_won_as_landlord"
);
const ranksUp: State<number> = numberLocalStorageState("ranks_up");

const gameStatistics: State<GameStatistics> = combineState({
  gamesPlayed,
  gamesPlayedAsDefending,
  gamesPlayedAsLandlord,
  gamesWon,
  gamesWonAsDefending,
  gamesWonAsLandlord,
  ranksUp,
});

export default gameStatistics;
