import * as React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { Player } from "./gen-types";
import Players from "./Players";
import RiverCrossingNotice from "./RiverCrossingNotice";

jest.mock("./WebsocketProvider", () => ({
  WebsocketContext: jest
    .requireActual("react")
    .createContext({ send: jest.fn() }),
}));

const players: Player[] = [
  { id: 1, name: "Alice", level: "2", metalevel: 1 },
  { id: 2, name: "Bob", level: "2", metalevel: 2 },
  { id: 3, name: "Carol", level: "2", metalevel: 1 },
  { id: 4, name: "Dave", level: "2", metalevel: 1 },
];

describe("river crossing announcements and deal markers", () => {
  it("announces every initiator, without naming the return-only partner", () => {
    const html = renderToStaticMarkup(
      <RiverCrossingNotice players={players} initiators={[1, 2]} />,
    );
    expect(html).toContain("Alice、Bob");
    expect(html).toContain("发起了过河");
    expect(html).toContain('role="status"');
    expect(html).not.toContain("Carol");
    expect(html).not.toContain("Dave");
  });

  it("places badges next to names while preserving rank and player descriptors", () => {
    const html = renderToStaticMarkup(
      <Players
        players={players}
        observers={[]}
        landlord={1}
        name="Alice"
        riverCrossingInitiators={[1, 2]}
      />,
    );
    expect(html.match(/class="river-crossing-badge"/g)).toHaveLength(2);
    expect(html).toContain('Alice<span class="river-crossing-badge"');
    expect(html).toContain('Bob<span class="river-crossing-badge"');
    expect(html).toContain(" (当庄) (You!)");
    expect(html).toContain(" (rank 2<sup>2</sup>)");
    expect(html).toContain("Carol (rank 2)");
  });

  it.each([undefined, []])(
    "shows no notice or badge for a fresh or legacy deal (%s)",
    (initiators) => {
      expect(
        renderToStaticMarkup(
          <RiverCrossingNotice players={players} initiators={initiators} />,
        ),
      ).toBe("");
      const html = renderToStaticMarkup(
        <Players
          players={players}
          observers={[]}
          name="Alice"
          riverCrossingInitiators={initiators}
        />,
      );
      expect(html).not.toContain("river-crossing-badge");
    },
  );
});
