import * as React from "react";
import { useEffect, useState } from "react";
import styled from "styled-components";
import { AppState } from "./AppStateProvider";

const Row = styled.div`
  display: table-row;
  line-height: 23px;
`;
const LabelCell = styled.div`
  display: table-cell;
  padding-right: 2em;
  font-weight: bold;
  width: 200px;
`;
const Cell = styled.div`
  display: table-cell;
`;

interface RowIProps {
  roomName: string;
  numPlayers: number;
  maxPlayers: number | null;
  setRoomName: (name: string) => void;
}

const PublicRoomRow = ({
  roomName,
  numPlayers,
  maxPlayers,
  setRoomName,
}: RowIProps): JSX.Element => {
  const playersDisplay = maxPlayers
    ? `${numPlayers}/${maxPlayers}`
    : `${numPlayers}`;
  return (
    <Row>
      <Cell>
        <button onClick={() => setRoomName(roomName)} className="normal">
          {roomName}
        </button>
      </Cell>
      <Cell>{playersDisplay}</Cell>
    </Row>
  );
};

interface IProps {
  setRoomName: (name: string) => void;
  updateState: (newState: Partial<AppState>) => void;
}

const PublicRoomsPane = (props: IProps): JSX.Element => {
  const [publicRooms, setPublicRooms] = useState<any[]>([]);

  useEffect(() => {
    loadPublicRooms();
  }, []);
  const loadPublicRooms = (): void => {
    // Clear any previous join error when refreshing
    props.updateState({ joinError: null });

    try {
      const fetchAsync = async (): Promise<void> => {
        const fetchResult = await fetch("public_games.json");
        const resultJSON = await fetchResult.json();
        setPublicRooms(resultJSON);
      };

      fetchAsync().catch((e) => {
        console.error(e);
      });
    } catch (err) {
      console.log(err);
    }
  };

  return (
    <div className="">
      <h3>Public Rooms</h3>
      <div>
        <p>
          The games listed below are open to the public. Join them to find new
          friends to play with!
        </p>
      </div>
      <div style={{ display: "table", borderSpacing: 10 }}>
        <Row>
          <LabelCell>Room Name</LabelCell>
          <LabelCell>Players</LabelCell>
          <LabelCell>
            <button onClick={loadPublicRooms} className="normal">
              Refresh
            </button>
          </LabelCell>
        </Row>
        {publicRooms.length === 0 && <Cell>No public rooms available</Cell>}
        {publicRooms.map((roomInfo) => {
          return (
            <PublicRoomRow
              key={roomInfo.name}
              roomName={roomInfo.name}
              numPlayers={roomInfo.num_players}
              maxPlayers={roomInfo.max_players}
              setRoomName={props.setRoomName}
            />
          );
        })}
      </div>
    </div>
  );
};

export default PublicRoomsPane;
