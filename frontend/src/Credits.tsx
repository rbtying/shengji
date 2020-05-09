import * as React from 'react';
import * as ReactModal from 'react-modal';

const contentStyle = {
  position: 'absolute',
  top: '50%',
  left: '50%',
  transform: 'translate(-50%, -50%)',
};

const ChangeLog = () => {
  const [modalOpen, setModalOpen] = React.useState<boolean>(false);
  return (
    <>
      <a
        onClick={(evt) => {
          evt.preventDefault();
          setModalOpen(true);
        }}
        href={window.location.href}
      >
        Change Log
      </a>
      <ReactModal
        isOpen={modalOpen}
        onRequestClose={() => setModalOpen(false)}
        shouldCloseOnOverlayClick
        shouldCloseOnEsc
        style={{content: contentStyle}}
      >
        <h2>Change Log</h2>
        <p>5/8/2020:</p>
        <ul>
          <li>
            When leader is set to winner-of-bid, players bid their own levels
            rather than a random selected one.
          </li>
          <li>Card labels are not visible above the settings pane.</li>
        </ul>
        <hr />
        <p>Changes prior to 5/8/2020 not listed</p>
      </ReactModal>
    </>
  );
};

const Credits = () => (
  <p>
    Made by Robert Ying, Abra Shen, and other{' '}
    <a
      href="https://github.com/rbtying/shengji/graphs/contributors"
      target="_blank"
    >
      friends
    </a>
    . Consider buying us boba via Venmo at @Robert-Ying, or contributing on{' '}
    <a href="https://github.com/rbtying/shengji" target="_blank">
      GitHub
    </a>
    !
    <span style={{float: 'right'}}>
      <ChangeLog />
    </span>
  </p>
);

export default Credits;
