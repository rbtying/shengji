import * as React from 'react';
import ChatMessage, {Message} from './ChatMessage';

type Props = {
  messages: Message[];
};

let latestMessage: Message | null = null;

const Chat = (props: Props) => {
  const [draft, setDraft] = React.useState<string>('');
  const anchor = React.useRef(null);

  React.useEffect(() => {
    // TODO: messages is mutable, so can't use reference equality to check for
    // new state. After making messages immutable, update to only rely on
    // props.messages
    const {messages} = props;
    if (
      messages.length > 0 &&
      messages[messages.length - 1] !== latestMessage
    ) {
      latestMessage = messages[messages.length - 1];
      anchor.current?.scrollIntoView({block: 'nearest', inline: 'start'});
    }
  });

  const handleSubmit = (event: React.SyntheticEvent) => {
    event.preventDefault();
    if (draft.length > 0) {
      (window as any).send({Message: draft});
    }
    setDraft('');
  };

  return (
    <div className="chat">
      <div className="messages">
        {props.messages.map((m, idx) => (
          <ChatMessage message={m} key={idx} />
        ))}
        <div className="chat-anchor" ref={anchor} />
      </div>
      <form onSubmit={handleSubmit}>
        <input
          type="text"
          placeholder="type message here"
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
        />
        <input type="submit" value="submit" />
      </form>
    </div>
  );
};

export default Chat;
