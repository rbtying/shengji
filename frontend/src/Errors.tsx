import * as React from 'react';

type Props = {
  errors: string[];
};

const Errors = (props: Props) => (
  <div className="errors">
    {props.errors.map((err, idx) => (
      <p key={idx}>
        <code>{err}</code>
      </p>
    ))}
  </div>
);

export default Errors;
