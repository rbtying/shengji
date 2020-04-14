import * as React from 'react';
import * as ReactModal from 'react-modal';
import IconButton from './IconButton';
import Gear from './icons/Gear';
import SettingsPane from './SettingsPane';
import {Settings} from './state/Settings';
import {AppStateConsumer} from './AppStateProvider';

const contentStyle = {
  position: 'absolute',
  top: '50%',
  left: '50%',
  transform: 'translate(-50%, -50%)',
};

const SettingsButton = () => {
  const [modalOpen, setModalOpen] = React.useState<boolean>(false);
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
        style={{content: contentStyle}}
      >
        <AppStateConsumer>
          {({state, updateState}) => (
            <SettingsPane
              settings={state.settings}
              onChangeSettings={(settings: Settings) => updateState({settings})}
            />
          )}
        </AppStateConsumer>
      </ReactModal>
    </>
  );
};

export default SettingsButton;
