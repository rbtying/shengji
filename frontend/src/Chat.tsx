import * as React from 'react';
import ChatMessage, {Message} from './ChatMessage';
import ChatInput from './ChatInput';
import {WebsocketContext} from './WebsocketProvider';

type Props = {
  messages: Message[];
};

const Chat = (props: Props) => {
  const anchor = React.useRef(null);
  const {send} = React.useContext(WebsocketContext);

  React.useEffect(() => {
    if (anchor.current) {
      const rect = anchor.current.getBoundingClientRect();
      const html = document.documentElement;
      const isVisible =
        rect.top >= 0 &&
        rect.left >= 0 &&
        rect.bottom <= (window.innerHeight || html.clientHeight) &&
        rect.right <= (window.innerWidth || html.clientWidth);
      if (isVisible) {
        anchor.current?.scrollIntoView({block: 'nearest', inline: 'start'});
      }
    }
  }, [props.messages]);

  const handleSubmit = (message: string) => send({Message: message});

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
