import * as React from 'react';
import {IPlayer} from './types';
import styled from 'styled-components';
import EmojiButton from './EmojiButton';
import {WebsocketContext} from './WebsocketProvider';

const Container = styled.div`
  width: 80%;
  max-width: 35em;
  border: 1px solid #eee;
  border-radius: 4px;
  padding: 1em;
`;

const Row = styled.div`
  display: table-row;
  border-collapse: separate;
`;

const Cell = styled.div`
  display: table-cell;
  border-style: solid;
  border-color: transparent;
  border-width: 0.2em 1em;
  white-space: nowrap;
`;

const FullWidthCell = styled.div`
  text-align: center;
`;

const Heading = styled(FullWidthCell)`
  font-size: 1.2em;
  padding: 0.5em;
  font-weight: 200;
`;

const Empty = styled(FullWidthCell)`
  font-style: italic;
  color: #555;
`;

const ButtonContainer = styled.div`
  display: flex;
  flex-direciton: row;
  justify-content: flex-end;
  width: 10em;
  & > * {
    margin: 0px 0.3em;
  }
`;

type RowProps = {
  player: IPlayer;
  actions: JSX.Element;
};
const UserRow = (props: RowProps) => (
  <Row key={props.player.name}>
    <Cell style={{width: '100%'}}>{props.player.name}</Cell>
    <Cell>Rank {props.player.level}</Cell>
    <Cell>{props.actions}</Cell>
  </Row>
);

type Props = {
  currentPlayer: IPlayer;
  players: IPlayer[];
  observers: IPlayer[];
};

const UserList = (props: Props) => {
  const {players, observers} = props;
  const {send} = React.useContext(WebsocketContext);

  const movePlayer = (player: IPlayer, relative: number) => () => {
    const index = players.findIndex((p) => p === player);
    const newIndex = (index + relative + players.length) % players.length;
    const withoutPlayer = players.filter((p) => p !== player);
    const newPlayers = [
      ...withoutPlayer.slice(0, newIndex),
      player,
      ...withoutPlayer.slice(newIndex, withoutPlayer.length),
    ];
    send({Action: {ReorderPlayers: newPlayers.map((p) => p.id)}});
  };

  const renderPlayerActions = (player: IPlayer) => (
    <ButtonContainer>
      <EmojiButton emoji="🔼" onClick={movePlayer(player, -1)} />
      <EmojiButton emoji="🔽" onClick={movePlayer(player, 1)} />
      <EmojiButton
        emoji="💤"
        onClick={() => send({Action: {MakeObserver: player.id}})}
      />
      <EmojiButton emoji="🚫" onClick={() => send({Kick: player.id})} />
    </ButtonContainer>
  );

  const renderObserverActions = (observer: IPlayer) => (
    <ButtonContainer>
      <EmojiButton
        emoji="✔️"
        onClick={() => send({Action: {MakePlayer: observer.id}})}
      />
      <EmojiButton emoji="🚫" onClick={() => send({Kick: observer.id})} />
    </ButtonContainer>
  );

  return (
    <Container>
      <Heading>Players</Heading>
      {players.map((player) => (
        <UserRow
          player={player}
          key={player.name}
          actions={renderPlayerActions(player)}
        />
      ))}
      {players.length === 0 && <Empty>(No players)</Empty>}

      {observers.length > 0 && (
        <Heading style={{paddingTop: '1em'}}>Spectators</Heading>
      )}
      {observers.map((observer) => (
        <UserRow
          player={observer}
          key={observer.name}
          actions={renderObserverActions(observer)}
        />
      ))}
    </Container>
  );
};

export default UserList;
