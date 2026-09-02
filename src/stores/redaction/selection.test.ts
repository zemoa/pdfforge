import { describe, expect, it } from "vitest";

import { addWordRange, removeWord, toggleWord } from "./selection";

describe("redaction text selections", () => {
  it("toggles an individual word", () => {
    const selected = toggleWord({}, 1, 4, "secret");

    expect(selected).toEqual({ 1: { 4: "secret" } });
    expect(toggleWord(selected, 1, 4, "secret")).toEqual({});
  });

  it("adds every word in a dragged range in document order", () => {
    expect(
      addWordRange({}, 2, 3, 1, [
        { index: 0, text: "one" },
        { index: 1, text: "two" },
        { index: 2, text: "three" },
        { index: 3, text: "four" },
      ]),
    ).toEqual({ 2: { 1: "two", 2: "three", 3: "four" } });
  });

  it("removes the final selection of a page", () => {
    expect(removeWord({ 3: { 9: "private" } }, 3, 9)).toEqual({});
  });
});
