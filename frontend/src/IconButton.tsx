import * as React from "react";
import styled from "styled-components";

const Button = styled.button`
  outline: none;
  padding: 0;
  margin: 0;
  border: 0;
  background-color: transparent;
  transition: opacity 100ms ease-in-out, color 150ms ease-in-out,
    transform 100ms ease-in-out;
  color: #111;
  &:hover {
    color: #666;
  }
`;

const IconButton = (
  props: React.ComponentProps<typeof Button>
): JSX.Element => {
  return <Button className="icon-button" {...props} />;
};
export default IconButton;
