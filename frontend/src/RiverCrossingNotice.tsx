import * as React from "react";
import { Player } from "./gen-types";

import type { JSX } from "react";

interface IProps {
  players: Player[];
  initiators?: number[];
}

const RiverCrossingNotice = ({
  players,
  initiators,
}: IProps): JSX.Element | null => {
  const names = players
    .filter((player) => initiators?.includes(player.id))
    .map((player) => player.name);

  if (names.length === 0) {
    return null;
  }

  return (
    <p className="river-crossing-notice" role="status" aria-live="polite">
      五主过河：<strong>{names.join("、")}</strong> 发起了过河。
      <span> / River crossing initiated by {names.join(", ")}.</span>
    </p>
  );
};

export default RiverCrossingNotice;
