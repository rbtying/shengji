import * as React from "react";
import { renderToStaticMarkup } from "react-dom/server";

import PlayHistory, {
  OwnPlayHistoryPanel,
  PreviousTrickPanel,
} from "./PlayHistory";
import { Player, Trick, Trump } from "./gen-types";

jest.mock(
  "./LabeledPlay",
  () =>
    ({
      cards,
      className,
      label,
    }: {
      cards?: string[];
      className?: string;
      label: React.ReactNode;
    }) => (
      <div className={className} data-cards={(cards ?? []).join(",")}>
        {label}
      </div>
    ),
);

const players: Player[] = [
  { id: 1, name: "Alice", level: "2", metalevel: 1 },
  { id: 2, name: "Bob", level: "2", metalevel: 1 },
  { id: 3, name: "Carol", level: "2", metalevel: 1 },
  { id: 4, name: "Dave", level: "2", metalevel: 1 },
];
const trump: Trump = { Standard: { number: "2", suit: "♤" } };
const lastTrick: Trick = {
  trump,
  current_winner: 2,
  played_cards: [
    { id: 1, cards: ["♤A"], bad_throw_cards: [] },
    { id: 2, cards: ["♤K"], bad_throw_cards: [] },
    { id: 3, cards: ["♤Q"], bad_throw_cards: [] },
    { id: 4, cards: ["♤J"], bad_throw_cards: [] },
  ],
  player_queue: [],
  trick_format: null,
};

describe("play history controls", () => {
  it("always renders two collapsed buttons and disables unavailable data", () => {
    const html = renderToStaticMarkup(
      <PlayHistory
        currentTrick={<div>Current trick</div>}
        lastTrick={null}
        ownHistory={[]}
        ownHistoryAvailable={true}
        players={players}
        landlord={1}
        landlordSuffix="(当庄)"
        landlordsTeam={[1, 3]}
        name="Alice"
        trump={trump}
      />,
    );

    expect(html).toContain("Current trick");
    expect(html).toContain("Previous trick unavailable / 上一轮暂无");
    expect(html).toContain("My play history (0) ▼ / 我的出牌历史（0墩）");
    expect(html.match(/aria-expanded="false"/g)).toHaveLength(2);
    expect(html).not.toContain('id="previous-trick-panel"');
    expect(html).not.toContain('id="own-play-history-panel"');
  });

  it("shows all four players in seat order in the previous-trick panel", () => {
    const html = renderToStaticMarkup(
      <PreviousTrickPanel
        trick={lastTrick}
        players={players}
        landlord={1}
        landlordSuffix="(当庄)"
        landlordsTeam={[1, 3]}
        name="Alice"
        trump={trump}
      />,
    );

    expect(html.indexOf("Alice")).toBeLessThan(html.indexOf("Bob"));
    expect(html.indexOf("Bob")).toBeLessThan(html.indexOf("Carol"));
    expect(html.indexOf("Carol")).toBeLessThan(html.indexOf("Dave"));
    expect(html).toContain('data-cards="♤A"');
    expect(html).toContain('data-cards="♤K"');
    expect(html).toContain('class="winning"');
  });

  it("renders personal history newest first while preserving trick groups", () => {
    const html = renderToStaticMarkup(
      <OwnPlayHistoryPanel
        history={[["♤3"], ["♡4", "♡4"], ["♧5"]]}
        trump={trump}
      />,
    );

    expect(html.indexOf("第3墩")).toBeLessThan(html.indexOf("第2墩"));
    expect(html.indexOf("第2墩")).toBeLessThan(html.indexOf("第1墩"));
    expect(html).toContain('data-cards="♡4,♡4"');
  });

  it("keeps the personal-history button visible but disabled for observers", () => {
    const html = renderToStaticMarkup(
      <PlayHistory
        currentTrick={<div>Current trick</div>}
        lastTrick={lastTrick}
        ownHistory={[]}
        ownHistoryAvailable={false}
        players={players}
        landlord={1}
        landlordSuffix="(当庄)"
        landlordsTeam={[1, 3]}
        name="Observer"
        trump={trump}
      />,
    );

    expect(html).toContain("My play history unavailable / 观察者无个人记录");
    expect(html.match(/ disabled=""/g)).toHaveLength(1);
  });
});
