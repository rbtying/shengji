import * as React from "react";
import ChatMessage, { IMessage } from "./ChatMessage";
import ChatInput from "./ChatInput";
import { WebsocketContext } from "./WebsocketProvider";

interface IProps {
  messages: IMessage[];
}

const Chat = (props: IProps): JSX.Element => {
  const anchor = React.useRef<HTMLDivElement | null>(null);
  const { send } = React.useContext(WebsocketContext);

  React.useEffect(() => {
    if (anchor.current !== null) {
      const rect = anchor.current.getBoundingClientRect();
      const html = document.documentElement;
      const isVisible =
        rect.top >= 0 &&
        rect.left >= 0 &&
        // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
        rect.bottom <= (window.innerHeight || html.clientHeight) &&
        // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
        rect.right <= (window.innerWidth || html.clientWidth);
      if (isVisible) {
        anchor.current?.scrollIntoView({ block: "nearest", inline: "start" });
      }
    }
  }, [props.messages]);

  const handleSubmit = (message: string): void => send({ Message: message });

  return (
    <div className="chat">
      <div className="messages">
        {props.messages.map((m, idx) => (
          <ChatMessage message={m} key={idx} />
        ))}
        <div className="chat-anchor" ref={anchor} />
      </div>
      <ChatInput onSubmit={handleSubmit} />
    </div>
  );
};

export default Chat;
