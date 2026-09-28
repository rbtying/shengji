import * as React from "react";
import { WebsocketContext } from "./WebsocketProvider";
import { Difficulty } from "./gen-types";

import type { JSX } from "react";

const DIFFICULTIES: { value: Difficulty; label: string }[] = [
  { value: "Beginner", label: "Beginner (learning)" },
  { value: "Easy", label: "Easy" },
  { value: "Intermediate", label: "Intermediate" },
  { value: "Hard", label: "Hard" },
  { value: "Expert", label: "Expert" },
];

const STORAGE_KEY = "bot_difficulty";

interface IProps {
  numPlayers: number;
}

const AddBots = (props: IProps): JSX.Element => {
  const { send } = React.useContext(WebsocketContext);
  const [difficulty, setDifficulty] = React.useState<Difficulty>(() => {
    try {
      const saved = window.localStorage.getItem(STORAGE_KEY);
      if (DIFFICULTIES.some((d) => d.value === saved)) {
        return saved as Difficulty;
      }
    } catch {
      // Storage may be unavailable; fall back to the default.
    }
    return "Intermediate";
  });

  const choose = (e: React.ChangeEvent<HTMLSelectElement>): void => {
    const d = e.target.value as Difficulty;
    setDifficulty(d);
    try {
      window.localStorage.setItem(STORAGE_KEY, d);
    } catch {
      // Not remembering the choice is fine.
    }
  };

  const addBots = (count: number): void => {
    for (let i = 0; i < count; i++) {
      // Stagger joins so seats fill in order.
      setTimeout(() => send({ AddBot: difficulty }), i * 400);
    }
  };

  const toFour = Math.max(0, 4 - props.numPlayers);

  return (
    <div className="add-bots">
      <label>
        Computer players:{" "}
        <select value={difficulty} onChange={choose}>
          {DIFFICULTIES.map((d) => (
            <option value={d.value} key={d.value}>
              {d.label}
            </option>
          ))}
        </select>
      </label>{" "}
      <button className="normal" onClick={() => addBots(1)}>
        Add a bot
      </button>{" "}
      {toFour > 0 && (
        <button className="normal" onClick={() => addBots(toFour)}>
          Fill to 4 players with bots
        </button>
      )}
      <p className="add-bots-help">
        To remove a bot, kick it below. Bots draw, bid and play on their own,
        and finish each trick after a short pause.
      </p>
    </div>
  );
};

export default AddBots;
