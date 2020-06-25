import * as React from "react";
import ArrayUtils from "./util/array";
import styled from "styled-components";

const Container = styled.div`
  position: relative;
`;
const Positioner = styled.div`
  position: absolute;
  width: 0px;
  height: 0px;
`;

const ItemContainer = styled.div`
  position: absolute;
  transform: translate(-50%, -50%);
  top: 50%;
  left: 50%;
`;

interface IProps {
  style?: React.CSSProperties;
  children: JSX.Element[];
}
const EllipseLayout: React.FunctionComponent<IProps> = (props: IProps) => {
  const childrenCount = React.Children.count(props.children);
  const step = (2 * Math.PI) / childrenCount;
  const angles = ArrayUtils.range(childrenCount, (i) => i * step);
  const arrangedChildren = React.Children.map(props.children, (child, i) => {
    const scaleFactor = 0.7;
    const toPercent = (range: number): number =>
      (range * scaleFactor * 100 + 100) / 2;
    const positionStyle = {
      left: `${toPercent(Math.sin(angles[i]))}%`,
      bottom: `${toPercent(-1 * Math.cos(angles[i]))}%`,
    };
    return (
      <Positioner style={positionStyle}>
        <ItemContainer>{child}</ItemContainer>
      </Positioner>
    );
  });

  return <Container style={props.style}>{arrangedChildren}</Container>;
};

export default EllipseLayout;
