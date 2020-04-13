import * as React from 'react';
import ArrayUtils from './util/array';

type Props = {
  style?: React.CSSProperties;
};
const EllipseLayout: React.FunctionComponent<Props> = (props) => {
  const childrenCount = React.Children.count(props.children);
  const step = (2 * Math.PI) / childrenCount;
  const angles = ArrayUtils.range(childrenCount, (i) => i * step);
  const arrangedChildren = React.Children.map(props.children, (child, i) => {
    const scaleFactor = 0.7;
    const toPercent = (range: number): number =>
      (range * scaleFactor * 100 + 100) / 2;
    const positionStyle = {
      position: 'absolute',
      left: `${toPercent(Math.sin(angles[i]))}%`,
      bottom: `${toPercent(-1 * Math.cos(angles[i]))}%`,
      width: '0px',
      height: '0px',
    };
    const containerStyle = {
      position: 'absolute',
      transform: 'translate(-50%, -50%)',
      top: '50%',
      left: '50%',
    };
    return (
      <div style={positionStyle}>
        <div style={containerStyle}>{child}</div>
      </div>
    );
  });

  return (
    <div style={{...props.style, position: 'relative'}}>{arrangedChildren}</div>
  );
};

export default EllipseLayout;
