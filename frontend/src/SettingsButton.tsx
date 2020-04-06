import * as React from 'react';
import * as ReactModal from 'react-modal';
import Gear from './icons/Gear';
import SettingsPane from './SettingsPane';
import {SettingsProps} from './SettingsProvider';

const contentStyle = {
  position: 'absolute',
  top: '50%',
  left: '50%',
  transform: 'translate(-50%, -50%)',
};

const SettingsButton = (props: SettingsProps) => {
  const [hover, setHover] = React.useState<boolean>(false);
  const [modalOpen, setModalOpen] = React.useState<boolean>(false);
  const fill = hover ? '#444' : '#000';
  return (
    <a
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      style={{
        height: '1em',
        width: '1em',
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
        <SettingsPane
          settings={props.settings}
          onChangeSettings={props.onChangeSettings}
        />
      </ReactModal>
    </a>
  );
};

export default SettingsButton;
