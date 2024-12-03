import * as React from "react";
import ChatInput from "./ChatInput";
import ChatMessage, { Message } from "./ChatMessage";
import { WebsocketContext } from "./WebsocketProvider";
import { SettingsContext } from "./AppStateProvider";

interface IProps {
  messages: Message[];
}

const Chat = (props: IProps): JSX.Element => {
  const anchor = React.useRef<HTMLDivElement | null>(null);
  const { send } = React.useContext(WebsocketContext);
  const settings = React.useContext(SettingsContext);

  React.useEffect(() => {
    if (anchor.current !== null) {
      const rect = anchor.current.getBoundingClientRect();
      const html = document.documentElement;
      const isVisible =
        rect.top >= 0 &&
        rect.left >= 0 &&
        rect.bottom <= (window.innerHeight || html.clientHeight) &&
        rect.right <= (window.innerWidth || html.clientWidth);
      if (isVisible) {
        anchor.current?.scrollIntoView({ block: "nearest", inline: "start" });
      }
    }
  }, [props.messages]);

  const handleSubmit = (message: string): void => send({ Message: message });

  return (
    !settings.hideChatBox && (
      <div className="chat">
        <div className="messages">
          {props.messages.map((m, idx) => (
            <ChatMessage message={m} key={idx} />
          ))}
          <div className="chat-anchor" ref={anchor} />
        </div>
        <ChatInput onSubmit={handleSubmit} />
      </div>
    )
  );
};

export default Chat;
