import * as React from "react";
import * as Shengji from "../shengji-wasm/pkg/shengji-core.js";
import WasmContext from "./WasmContext";
import { ITrump } from "./types";

interface IProps {
  children: React.ReactNode;
}
const ShengjiProvider = (props: IProps): JSX.Element => {
  (window as any).shengji = Shengji;
  return (
    <WasmContext.Provider
      value={{
        findViablePlays: (trump: ITrump, cards: string[]) => {
          return Shengji.find_viable_plays({ trump, cards }).results;
        },
        findValidBids: (req) => {
          return Shengji.find_valid_bids(req).results;
        },
        sortAndGroupCards: (req) => {
          return Shengji.sort_and_group_cards(req).results;
        },
      }}
    >
      {props.children}
    </WasmContext.Provider>
  );
};
export default ShengjiProvider;
