import * as React from "react";
import { AppStateContext } from "./AppStateProvider";
import Timeout from "./Timeout";

interface IProps {
  errors: string[];
}

const Errors = (props: IProps): JSX.Element => {
  const { updateState } = React.useContext(AppStateContext);
  return (
    <div className="errors">
      <Timeout timeout={5000} callback={() => updateState({ errors: [] })} />
      {props.errors.map((err, idx) => (
        <p key={idx}>
          <code>{err}</code>
        </p>
      ))}
    </div>
  );
};

export default Errors;
