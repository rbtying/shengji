import * as React from 'react';
import {IGameMode} from './types';

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

          const c = (window as any).CARD_LUT[friend.card];
          if (!c) {
            return null;
          }
          const card = `${c.number}${c.typ}`;
          if (friend.skip === 0) {
            return (
              <p key={idx}>
                The next person to play <span className={c.typ}>{card}</span> is
                a friend
              </p>
            );
          } else {
            return (
              <p key={idx}>
                {friend.skip} <span className={c.typ}>{card}</span> can be
                played before the next person to play{' '}
                <span className={c.typ}>{card}</span> is a friend
              </p>
            );
          }
        })}
      </div>
    );
  } else {
    return null;
  }
};

export default Friends;
