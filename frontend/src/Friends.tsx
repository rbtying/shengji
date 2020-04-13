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
          if (friend.skip === 0) {
            return (
              <p key={idx}>
                The next person to play <InlineCard card={friend.card} /> is a
                friend
              </p>
            );
          } else {
            return (
              <p key={idx}>
                {friend.skip} <InlineCard card={friend.card} /> can be played
                before the next person to play <InlineCard card={friend.card} />{' '}
                is a friend
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
