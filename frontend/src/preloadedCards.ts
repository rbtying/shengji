/* eslint-disable @typescript-eslint/no-var-requires */

import { ICardInfo } from "./types";
const cardsJson = require("./generated/cards.json");

export default cardsJson.cards as ICardInfo[];
