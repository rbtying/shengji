import * as React from "react";
import * as ReactModal from "react-modal";
import IconButton from "./IconButton";
import Gear from "./icons/Gear";
import SettingsPane from "./SettingsPane";
import { Settings } from "./state/Settings";
import { AppStateContext } from "./AppStateProvider";

const contentStyle: React.CSSProperties = {
  position: "absolute",
  top: "50%",
  left: "50%",
  transform: "translate(-50%, -50%)",
};

const SettingsButton = (): JSX.Element => {
  const [modalOpen, setModalOpen] = React.useState<boolean>(false);
  const { state, updateState } = React.useContext(AppStateContext);
  return (
    <>
      <IconButton onClick={() => setModalOpen(true)}>
        <Gear width="2em" />
      </IconButton>
      <ReactModal
        isOpen={modalOpen}
        onRequestClose={() => setModalOpen(false)}
        shouldCloseOnOverlayClick
        shouldCloseOnEsc
        style={{ content: contentStyle }}
      >
        <SettingsPane
          settings={state.settings}
          onChangeSettings={(settings: Settings) => updateState({ settings })}
        />
      </ReactModal>
    </>
  );
};

export default SettingsButton;
