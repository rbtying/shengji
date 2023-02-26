import * as React from "react";
import { useEffect, useState } from "react";
import styled from "styled-components";

const Row = styled.div`
  display: table-row;
  line-height: 23px;
`;
const LabelCell = styled.div`
  display: table-cell;
  padding-right: 2em;
  font-weight: bold;
  width: 125px;
`;
const Cell = styled.div`
  display: table-cell;
`;

interface RowIProps {
  roomName: string;
  numPlayers: number;
}

const PublicRoomRow = ({ roomName, numPlayers }: RowIProps): JSX.Element => {
  return (
    <Row>
      <Cell>{roomName}</Cell>
      <Cell>{numPlayers}</Cell>
    </Row>
  );
};

const PublicRoomsPane = (): JSX.Element => {
  const [publicRooms, setPublicRooms] = useState([]);

  useEffect(() => {
    loadPublicRooms();
  }, []);
  const loadPublicRooms = (): void => {
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
        <p>
          Copy the room name into the input above, fill out your player name,
          and click join to enter the room.
        </p>
      </div>
      <div style={{ display: "table", borderSpacing: 10 }}>
        <Row>
          <LabelCell>Room Name</LabelCell>
          <LabelCell>Players</LabelCell>
          <LabelCell>
            <button onClick={loadPublicRooms}> Refresh </button>
          </LabelCell>
        </Row>
        {publicRooms.length === 0 && <div>No rooms available</div>}
        {publicRooms.map((roomInfo) => {
          return (
            <PublicRoomRow
              key={roomInfo.name}
              roomName={roomInfo.name}
              numPlayers={roomInfo.num_players}
            />
          );
        })}
      </div>
    </div>
  );
};

export default PublicRoomsPane;
