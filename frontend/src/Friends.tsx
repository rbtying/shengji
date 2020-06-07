import * as React from 'react';
import {IGameMode} from './types';
import InlineCard from './InlineCard';

type Props = {gameMode: IGameMode};

const Friends = (props: Props) => {
  const {gameMode} = props;
  if (gameMode !== 'Tractor') {
    return (
      <div className="pending-friends">
        {gameMode.FindingFriends.friends.map((friend, idx) => {
          if (friend.player_id !== null) {
            return null;
          }

          if (!friend.card) {
            return null;
          }
          return (
            <p key={idx}>
              The person to play {nth(friend.skip+1)} <InlineCard card={friend.card} /> is a
              friend
            </p>
          );
        })}
      </div>
    );
  } else {
    return null;
  }
};

function nth(n:number){return n+["st","nd","rd"][(((n<0?-n:n)+90)%100-10)%10-1]||"th"}

export default Friends;
