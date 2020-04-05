import * as React from 'react';

type Props = {
  children: React.ReactNode;
};
const DivWithProps = (extraProps: {}) => (props: Props) => (
  <div {...extraProps} {...props} />
);

export default DivWithProps;
