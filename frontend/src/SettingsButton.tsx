import * as React from "react";
import ReactModal from "react-modal";
import IconButton from "./IconButton";
import Gear from "./icons/Gear";
import SettingsPane from "./SettingsPane";
import ReactTooltip from "react-tooltip";
import { Settings } from "./state/Settings";
import { AppStateContext } from "./AppStateProvider";

const contentStyle: React.CSSProperties = {
  position: "absolute",
  top: "50%",
  left: "50%",
  width: "80%",
  transform: "translate(-50%, -50%)",
};

const SettingsButton = (): JSX.Element => {
  const [modalOpen, setModalOpen] = React.useState<boolean>(false);
  const { state, updateState } = React.useContext(AppStateContext);
  return (
    <>
      <ReactTooltip id="settingsTip" place="top" effect="solid">
        Change user interface settings
      </ReactTooltip>
      <IconButton
        onClick={() => setModalOpen(true)}
        data-tip
        data-for="settingsTip"
      >
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
