import * as React from "react";
import { WebsocketContext } from "./WebsocketProvider";
import LabeledPlay from "./LabeledPlay";

interface Props {
  name: string;
  room_name: string;
  setName: (name: string) => void;
  setRoomName: (name: string) => void;
}

const JoinRoom = (props: Props) => {
  const [editable, setEditable] = React.useState<boolean>(false);
  const { send } = React.useContext(WebsocketContext);

  const handleChange = (event: any) => props.setName(event.target.value.trim());

  const handleRoomChange = (event: any) =>
    props.setRoomName(event.target.value.trim());

  const handleSubmit = (event: any) => {
    event.preventDefault();
    if (props.name.length > 0 && props.room_name.length === 16) {
      send({
        room_name: props.room_name,
        name: props.name,
      });
    }
  };

  const editableRoomName = (
    <input
      type="text"
      placeholder="Enter a room code"
      value={props.room_name}
      onChange={handleRoomChange}
      maxLength={16}
    />
  );
  const nonEditableRoomName = (
    <span
      onClick={(evt) => {
        evt.preventDefault();
        setEditable(true);
      }}
    >
      {props.room_name}
    </span>
  );

  return (
    <div>
      <LabeledPlay cards={["🃟", "🃟", "🃏", "🃏"]} label={null}></LabeledPlay>
      <form className="join-room" onSubmit={handleSubmit}>
        <div>
          <h2>
            <label>
              <strong>Room Name:</strong>{" "}
              {editable ? editableRoomName : nonEditableRoomName} (
              <a href="rules" target="_blank">
                rules
              </a>
              )
            </label>
          </h2>
        </div>
        <div>
          <label>
            <strong>Player Name:</strong>{" "}
            <input
              type="text"
              placeholder="Enter your name here"
              value={props.name}
              onChange={handleChange}
              autoFocus={true}
            />
          </label>
          <input
            type="submit"
            value="Join the game!"
            disabled={
              props.room_name.length !== 16 ||
              props.name.length === 0 ||
              props.name.length > 32
            }
          />
        </div>
        <div></div>
      </form>
    </div>
  );
};

export default JoinRoom;
