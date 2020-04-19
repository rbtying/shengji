import * as React from 'react';
import styled from 'styled-components';

const Button = styled.button`
  border-radius: 4px;
  border: 1px solid #eee;
  padding: 0;
  margin: 0;
  width: 2.5em;
  height: 2.5em;
  padding: 0.3em;
  outline: none;
  user-select: none;
  &:hover {
    background-color: #eee;
  }
`;

const EmojiContainer = styled.span`
  font-size: 12px;
  padding-left: 2.5px;
`;

type Props = React.ComponentProps<typeof Button> & {
  emoji: string;
};
const EmojiButton = (props: Props) => {
  const {emoji, ...otherProps} = props;
  return (
    <Button {...otherProps}>
      <EmojiContainer>{props.emoji}</EmojiContainer>
    </Button>
  );
};

export default EmojiButton;
