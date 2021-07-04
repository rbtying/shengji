import * as React from "react";
import * as Shengji from "../shengji-wasm/pkg/shengji-core.js";
import WasmContext from "./WasmContext";
import { ITrump, ITractorRequirements } from "./types";

interface IProps {
  children: React.ReactNode;
}
const ShengjiProvider = (props: IProps): JSX.Element => {
  (window as any).shengji = Shengji;
  return (
    <WasmContext.Provider
      value={{
        findViablePlays: (
          trump: ITrump,
          tractorRequirements: ITractorRequirements,
          cards: string[]
        ) => {
          return Shengji.find_viable_plays({
            trump,
            cards,
            tractor_requirements: tractorRequirements,
          }).results;
        },
        findValidBids: (req) => {
          return Shengji.find_valid_bids(req).results;
        },
        sortAndGroupCards: (req) => {
          return Shengji.sort_and_group_cards(req).results;
        },
        decomposeTrickFormat: (req) => {
          return Shengji.decompose_trick_format(req).results;
        },
        canPlayCards: (req) => {
          return Shengji.can_play_cards(req).playable;
        },
        explainScoring: (req) => {
          return Shengji.explain_scoring(req);
        },
        nextThresholdReachable: (req) => {
          return Shengji.next_threshold_reachable(req);
        },
        computeScore: (req) => {
          return Shengji.compute_score(req);
        },
        computeDeckLen: (req) => {
          return Shengji.compute_deck_len(req);
        },
        decodeWireFormat: (req) => {
          return JSON.parse(Shengji.zstd_decompress(req));
        },
      }}
    >
      {props.children}
    </WasmContext.Provider>
  );
};
export default ShengjiProvider;
