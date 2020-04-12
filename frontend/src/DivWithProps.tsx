import * as React from 'react';

const DivWithProps = (extraProps: {
  style: React.CSSProperties;
}): React.FunctionComponent<
  React.DetailedHTMLProps<React.HTMLAttributes<HTMLDivElement>, HTMLDivElement>
> => (props) => {
  const style = {
    ...extraProps.style,
    ...props.style,
  };
  return <div {...extraProps} {...props} style={style} />;
};

export default DivWithProps;
