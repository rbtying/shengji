import * as React from "react";

import classNames from "classnames";
import InlineCard from "./InlineCard";
import { cardLookup } from "./util/cardHelpers";
import { SettingsContext } from "./AppStateProvider";

const SvgCard = React.lazy(async () => await import("./SvgCard"));

interface IProps {
  card: string;
  smaller?: boolean;
  className?: string;
  onClick?: (event: React.MouseEvent) => void;
}

const Card = (props: IProps): JSX.Element => {
  const settings = React.useContext(SettingsContext);
  if (!(props.card in cardLookup)) {
    const nonSVG = (
      <span className={classNames("card", "unknown", props.className)}>
        {props.card}
      </span>
    );

    if (settings.svgCards) {
      return (
        <React.Suspense fallback={nonSVG}>
          <span
            className={classNames("card", "svg", "unknown", props.className)}
          >
            <SvgCard
              fourColor={settings.fourColor}
              smaller={props.smaller}
              card={"🂠"}
            />
          </span>
        </React.Suspense>
      );
    } else {
      return nonSVG;
    }
  } else {
    const cardInfo = cardLookup[props.card];
    const nonSVG = (
      <span
        className={classNames("card", cardInfo.typ, props.className)}
        onClick={props.onClick}
      >
        <div className="card-label">
          <InlineCard card={props.card} />
        </div>
        {cardInfo.display_value}
      </span>
    );

    if (settings.svgCards) {
      return (
        <React.Suspense fallback={nonSVG}>
          <span
            className={classNames("card", "svg", cardInfo.typ, props.className)}
            onClick={props.onClick}
          >
            <div className="card-label">
              <InlineCard card={props.card} />
            </div>
            <SvgCard
              fourColor={settings.fourColor}
              smaller={props.smaller}
              card={props.card}
            />
          </span>
        </React.Suspense>
      );
    } else {
      return nonSVG;
    }
  }
};

export default Card;
