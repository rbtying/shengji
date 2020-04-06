import * as React from 'react';
import {AppStateConsumer} from './AppStateProvider';
import Timeout from './Timeout';

type Props = {
  errors: string[];
};

const Errors = (props: Props) => (
  <div className="errors">
    <AppStateConsumer>
      {({updateState}) => (
        <Timeout timeout={5000} callback={() => updateState({errors: []})} />
      )}
    </AppStateConsumer>
    {props.errors.map((err, idx) => (
      <p key={idx}>
        <code>{err}</code>
      </p>
    ))}
  </div>
);

export default Errors;
