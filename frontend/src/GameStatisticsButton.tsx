import * as React from "react";
import * as ReactModal from "react-modal";
import IconButton from "./IconButton";
import BarChart from "./icons/BarChart";
import GameStatisticsPane from "./GameStatisticsPane";
import { AppStateContext } from "./AppStateProvider";

const contentStyle: React.CSSProperties = {
  position: "absolute",
  top: "50%",
  left: "50%",
  transform: "translate(-50%, -50%)",
};

const GameStatisticsButton = (): JSX.Element => {
  const [modalOpen, setModalOpen] = React.useState<boolean>(false);
  const { state } = React.useContext(AppStateContext);
  return (
    <>
      <IconButton
        style={{ paddingLeft: "10px" }}
        onClick={() => setModalOpen(true)}
      >
        <BarChart width="2em" />
      </IconButton>
      <ReactModal
        isOpen={modalOpen}
        onRequestClose={() => setModalOpen(false)}
        shouldCloseOnOverlayClick
        shouldCloseOnEsc
        style={{ content: contentStyle }}
      >
        <GameStatisticsPane gameStatistics={state.gameStatistics} />
      </ReactModal>
    </>
  );
};

export default GameStatisticsButton;
