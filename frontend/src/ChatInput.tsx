import * as React from "react";
import styled from "styled-components";
import IconButton from "./IconButton";
import PaperPlane from "./icons/PaperPlane";

const ChatBox = styled.div`
  border-radius: 25px;
  border: 0px;
  outline: none;
  background-color: #eee;
  display: flex;
  flex-direction: row;
  padding-left: 1em;
  padding-right: 1em;
  margin: 0.5em;
`;
const Input = styled.input`
  outline: none;
  background-color: #eee;
  border: none;
  margin-top: 0.6em;
  margin-bottom: 0.6em;
  font-size: 14px;
  line-height: 14px;
  height: 14px;
  flex: 1;
`;

interface IProps {
  onSubmit: (value: string) => void;
}
const ChatInput = (props: IProps): JSX.Element => {
  const [draft, setDraft] = React.useState<string>("");
  const handleSubmit = (event: React.SyntheticEvent): void => {
    event.preventDefault();
    if (draft.length > 0) {
      props.onSubmit(draft);
      setDraft("");
    }
  };

  const disabled = draft === "";

  return (
    <form onSubmit={handleSubmit}>
      <ChatBox>
        <Input
          type="text"
          placeholder="type message here"
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
        />
        <IconButton
          type="submit"
          style={{
            opacity: disabled ? 0 : 1,
            transform: disabled ? "translate(1em, 0)" : "none",
            margin: "auto",
          }}
          disabled={disabled}
        >
          <PaperPlane width="16px" />
        </IconButton>
      </ChatBox>
    </form>
  );
};
export default ChatInput;
