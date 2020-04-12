import * as React from 'react';

const ElementWithProps = (
  Element: string,
  extraProps: {},
): React.FunctionComponent<{}> => (props) => (
  <Element {...extraProps} {...(props as any)} />
);

export default ElementWithProps;
