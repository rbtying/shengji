import * as React from "react";
import { WebsocketContext } from "./WebsocketProvider";
import { TimerContext } from "./TimerProvider";
import LabeledPlay from "./LabeledPlay";
import PublicRoomsPane from "./PublicRoomsPane";

interface IProps {
  name: string;
  room_name: string;
  setName: (name: string) => void;
  setRoomName: (name: string) => void;
}

const JoinRoom = (props: IProps): JSX.Element => {
  const [editable, setEditable] = React.useState<boolean>(false);
  const [shouldGenerate, setShouldGenerate] = React.useState<boolean>(
    props.room_name.length !== 16
  );
  const { send } = React.useContext(WebsocketContext);
  const { setTimeout } = React.useContext(TimerContext);

  const handleChange = (event: React.ChangeEvent<HTMLInputElement>): void =>
    props.setName(event.target.value.trim());

  const handleRoomChange = (event: React.ChangeEvent<HTMLInputElement>): void =>
    props.setRoomName(event.target.value.trim());

  const handleSubmit = (event: React.SyntheticEvent): void => {
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
      title="Set the room name"
      onClick={(evt) => {
        evt.preventDefault();
        setEditable(true);
      }}
    >
      {props.room_name}
    </span>
  );

  const generateRoomName = (): void => {
    const arr = new Uint8Array(8);
    window.crypto.getRandomValues(arr);
    setShouldGenerate(false);
    props.setRoomName(
      Array.from(arr, (d) => ("0" + d.toString(16)).substr(-2)).join("")
    );
  };

  if (shouldGenerate) {
    setTimeout(generateRoomName, 0);
  }

  return (
    <div>
      <LabeledPlay
        cards={["🃟", "🃟", "🃏", "🃏"]}
        trump={{ NoTrump: {} }}
        label={null}
      ></LabeledPlay>
      <form className="join-room" onSubmit={handleSubmit}>
        <div>
          <h2>
            <label>
              <strong>Room Name:</strong>{" "}
              {editable ? editableRoomName : nonEditableRoomName}{" "}
              <span
                title="Generate new room"
                onClick={() => generateRoomName()}
              >
                🎲
              </span>{" "}
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
            value="Join (or create) the game!"
            disabled={
              props.room_name.length !== 16 ||
              props.name.length === 0 ||
              props.name.length > 32
            }
          />
        </div>
      </form>
      <div>
        <p>
          Welcome to the game! Enter your name above to create a new game, or
          (re-)join the game if it already exists.
        </p>
        <p>
          If you&apos;re unfamiliar with the game, it might be helpful to{" "}
          <a href="rules" target="_blank">
            read the rules
          </a>{" "}
          and then shadow another player&mdash;you can just join with the same
          name, case-sensitive.
        </p>
        <p>
          Once you are in the game, share the room link with at least three
          friends to start playing!
        </p>
        <p>
          This is a game with many house rules, so be sure to check out the game
          settings to see if your favorite rules are implemented. There&apos;s
          also a settings gear at the top, which can change how the game looks
          to you.
        </p>
      </div>
      <PublicRoomsPane setRoomName={props.setRoomName} />
    </div>
  );
};

export default JoinRoom;
