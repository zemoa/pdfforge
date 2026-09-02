import { describe, expect, it } from "vitest";

import {
  addWordRange,
  addZone,
  clearZones,
  moveZone,
  removeWord,
  removeZone,
  resizeZone,
  toggleWord,
} from "./selection";

describe("redaction text selections", () => {
  it("toggles an individual word", () => {
    const bounds = [{ left: 0.1, top: 0.1, width: 0.2, height: 0.2 }];
    const selected = toggleWord({}, 1, 4, "secret", bounds);

    expect(selected).toEqual({ 1: { 4: { text: "secret", bounds } } });
    expect(toggleWord(selected, 1, 4, "secret", bounds)).toEqual({});
  });

  it("adds every word in a dragged range in document order", () => {
    expect(
      addWordRange({}, 2, 3, 1, [
        { index: 0, text: "one", bounds: [] },
        { index: 1, text: "two", bounds: [] },
        { index: 2, text: "three", bounds: [] },
        { index: 3, text: "four", bounds: [] },
      ]),
    ).toEqual({
      2: {
        1: { text: "two", bounds: [] },
        2: { text: "three", bounds: [] },
        3: { text: "four", bounds: [] },
      },
    });
  });

  it("removes the final selection of a page", () => {
    expect(removeWord({ 3: { 9: { text: "private", bounds: [] } } }, 3, 9)).toEqual({});
  });
});

describe("redaction zone selections", () => {
  it("normalizes an inverted drag and clamps it to the page", () => {
    expect(addZone({}, 2, 8, { x: 1.2, y: 0.8 }, { x: 0.2, y: -0.4 })).toEqual({
      2: [{ id: 8, page: 2, rect: { left: 0.2, top: 0, width: 0.8, height: 0.8 } }],
    });
  });

  it("ignores a zone with no area", () => {
    expect(addZone({}, 1, 1, { x: 0.4, y: 0.2 }, { x: 0.4, y: 0.9 })).toEqual({});
  });

  it("keeps zones isolated by page", () => {
    const first = addZone({}, 1, 1, { x: 0.1, y: 0.1 }, { x: 0.2, y: 0.2 });
    const zones = addZone(first, 3, 2, { x: 0.3, y: 0.3 }, { x: 0.4, y: 0.4 });

    expect(Object.keys(zones)).toEqual(["1", "3"]);
    expect(zones[1][0]).toMatchObject({ id: 1, page: 1 });
    expect(zones[3][0]).toMatchObject({ id: 2, page: 3 });
    expect(zones[3][0].rect.width).toBeCloseTo(0.1);
    expect(zones[3][0].rect.height).toBeCloseTo(0.1);
  });

  it("moves and resizes a zone without leaving the page", () => {
    const zones = addZone({}, 1, 1, { x: 0.2, y: 0.2 }, { x: 0.6, y: 0.6 });
    const moved = moveZone(zones, 1, 1, zones[1][0].rect, { x: 1, y: -1 });
    const resized = resizeZone(moved, 1, 1, moved[1][0].rect, "bottom-right", { x: 2, y: 2 });

    expect(resized[1][0]).toMatchObject({ id: 1, page: 1 });
    expect(resized[1][0].rect.left).toBeCloseTo(0.6);
    expect(resized[1][0].rect.top).toBe(0);
    expect(resized[1][0].rect.width).toBeCloseTo(0.4);
    expect(resized[1][0].rect.height).toBe(1);
  });

  it("removes the final zone of a page", () => {
    const zones = addZone({}, 4, 7, { x: 0.1, y: 0.1 }, { x: 0.2, y: 0.2 });

    expect(removeZone(zones, 4, 7)).toEqual({});
  });

  it("clears zones from every page", () => {
    const first = addZone({}, 1, 1, { x: 0.1, y: 0.1 }, { x: 0.2, y: 0.2 });
    const zones = addZone(first, 2, 2, { x: 0.2, y: 0.2 }, { x: 0.3, y: 0.3 });

    expect(clearZones(zones)).toEqual({});
  });
});
