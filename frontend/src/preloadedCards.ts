import {ICardInfo} from './types';
// tslint:disable no-var-requires
const cardsJson = require('./generated/cards.json');

export default cardsJson.cards as ICardInfo[];
