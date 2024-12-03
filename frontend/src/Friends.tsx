import * as React from "react";
import { GameMode } from "./gen-types";
import InlineCard from "./InlineCard";

interface IProps {
  gameMode: GameMode;
  showPlayed: boolean;
}

const Friends = (props: IProps): JSX.Element => {
  const { gameMode } = props;
  if (gameMode !== "Tractor") {
    return (
      <div className="pending-friends">
        {gameMode.FindingFriends.friends.map((friend, idx) => {
          if (friend.player_id !== null) {
            return null;
          }

          if (
            friend.card === null ||
            friend.card === undefined ||
            friend.card.length === 0
          ) {
            return null;
          }
          return (
            <p key={idx}>
              The person to play the {nth(friend.initial_skip + 1)}{" "}
              <InlineCard card={friend.card} /> is a friend.{" "}
              {props.showPlayed
                ? `${
                    friend.initial_skip - friend.skip
                  } played in previous tricks.`
                : ""}
            </p>
          );
        })}
      </div>
    );
  } else {
    return <></>;
  }
};

function nth(n: number): string {
  const suffix = ["st", "nd", "rd"][
    (((((n < 0 ? -n : n) + 90) % 100) - 10) % 10) - 1
  ];
  return `${n}${suffix !== undefined ? suffix : "th"}`;
}

export default Friends;
