import DivWithProps from './DivWithProps';

const Main = DivWithProps({
  style: {
    display: 'flex',
    flexDirection: 'row',
    height: '100%',
  },
});

const ScrollPane = DivWithProps({
  style: {
    height: '100%',
    overflowY: 'auto',
  },
})

export default {
  Main,
  ScrollPane,
};
