import * as React from "react";
import * as Shengji from "../shengji-wasm/pkg/index.js";

interface IProps {
  children: React.ReactNode;
}
interface Context {}

export const ShengjiContext = React.createContext<Context>({});

const ShengjiProvider = (props: IProps): JSX.Element => {
  Shengji.init();
  return (
    <ShengjiContext.Provider value={{}}>
      {props.children}
    </ShengjiContext.Provider>
  );
};
export default ShengjiProvider;
