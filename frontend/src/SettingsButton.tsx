import * as React from 'react';
import * as ReactModal from 'react-modal';
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
  const [hover, setHover] = React.useState<boolean>(false);
  const [modalOpen, setModalOpen] = React.useState<boolean>(false);
  const fill = hover ? '#444' : '#000';
  return (
    <a
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      style={{
        height: '1.25em',
        width: '1.25em',
        display: 'inline-block',
        verticalAlign: 'middle',
        paddingLeft: '0.5em',
      }}
    >
      <span style={{cursor: 'pointer'}} onClick={() => setModalOpen(true)}>
        <Gear width="1em" fill={fill} />
      </span>
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
    </a>
  );
};

export default SettingsButton;
