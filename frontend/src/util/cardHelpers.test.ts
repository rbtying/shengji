import { unicodeToCard } from "./cardHelpers";

describe("Card helpers", () => {
  describe("unicodeToCard", () => {
    it("throws with invalid strings", () => {
      expect(() => unicodeToCard("")).toThrow();
      expect(() => unicodeToCard("a")).toThrow();
      expect(() => unicodeToCard("🂷 ")).toThrow();
    });

    it("works with various cards", () => {
      expect(unicodeToCard("🂤")).toEqual({
        type: "suit_card",
        rank: "4",
        suit: "spades",
      });
      expect(unicodeToCard("🂾")).toEqual({
        type: "suit_card",
        rank: "K",
        suit: "hearts",
      });
      expect(unicodeToCard("🃞")).toEqual({
        type: "suit_card",
        rank: "K",
        suit: "clubs",
      });
      expect(unicodeToCard("🃂")).toEqual({
        type: "suit_card",
        rank: "2",
        suit: "diamonds",
      });
    });

    it("ignores knight cards", () => {
      expect(() => unicodeToCard("🂬")).toThrow();
      expect(() => unicodeToCard("🂼")).toThrow();
      expect(() => unicodeToCard("🃌")).toThrow();
      expect(() => unicodeToCard("🃜")).toThrow();
    });

    it("works with jokers", () => {
      expect(unicodeToCard("🃟")).toEqual({ type: "little_joker" });
      expect(unicodeToCard("🃏")).toEqual({ type: "big_joker" });
    });
  });
});
