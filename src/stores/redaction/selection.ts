export type SelectionsByPage = Record<number, Record<number, string>>;

export interface NormalizedPoint {
  x: number;
  y: number;
}

export interface NormalizedRect {
  left: number;
  top: number;
  width: number;
  height: number;
}

export interface ZoneSelection {
  id: number;
  page: number;
  rect: NormalizedRect;
}

export type ZonesByPage = Record<number, ZoneSelection[]>;
export type ZoneResizeHandle = "top-left" | "top-right" | "bottom-left" | "bottom-right";

const MIN_COORDINATE = 0;
const MAX_COORDINATE = 1;

function clampCoordinate(value: number) {
  return Math.min(MAX_COORDINATE, Math.max(MIN_COORDINATE, value));
}

export function rectangleFromPoints(
  start: NormalizedPoint,
  end: NormalizedPoint,
): NormalizedRect | null {
  const startX = clampCoordinate(start.x);
  const startY = clampCoordinate(start.y);
  const endX = clampCoordinate(end.x);
  const endY = clampCoordinate(end.y);
  const left = Math.min(startX, endX);
  const top = Math.min(startY, endY);
  const width = Math.abs(endX - startX);
  const height = Math.abs(endY - startY);

  return width > 0 && height > 0 ? { left, top, width, height } : null;
}

export function moveRectangle(rectangle: NormalizedRect, offset: NormalizedPoint): NormalizedRect {
  return {
    ...rectangle,
    left: Math.min(1 - rectangle.width, Math.max(0, rectangle.left + offset.x)),
    top: Math.min(1 - rectangle.height, Math.max(0, rectangle.top + offset.y)),
  };
}

export function resizeRectangle(
  rectangle: NormalizedRect,
  handle: ZoneResizeHandle,
  point: NormalizedPoint,
): NormalizedRect | null {
  const opposite =
    handle === "top-left"
      ? { x: rectangle.left + rectangle.width, y: rectangle.top + rectangle.height }
      : handle === "top-right"
        ? { x: rectangle.left, y: rectangle.top + rectangle.height }
        : handle === "bottom-left"
          ? { x: rectangle.left + rectangle.width, y: rectangle.top }
          : { x: rectangle.left, y: rectangle.top };
  return rectangleFromPoints(opposite, point);
}

export function addZone(
  zones: ZonesByPage,
  page: number,
  id: number,
  start: NormalizedPoint,
  end: NormalizedPoint,
): ZonesByPage {
  const rect = rectangleFromPoints(start, end);
  if (!rect) return zones;
  const zone = { id, page, rect };
  return { ...zones, [page]: [...(zones[page] ?? []), zone] };
}

export function moveZone(
  zones: ZonesByPage,
  page: number,
  id: number,
  original: NormalizedRect,
  offset: NormalizedPoint,
): ZonesByPage {
  return updateZone(zones, page, id, moveRectangle(original, offset));
}

export function resizeZone(
  zones: ZonesByPage,
  page: number,
  id: number,
  original: NormalizedRect,
  handle: ZoneResizeHandle,
  point: NormalizedPoint,
): ZonesByPage {
  const rect = resizeRectangle(original, handle, point);
  return rect ? updateZone(zones, page, id, rect) : zones;
}

export function removeZone(zones: ZonesByPage, page: number, id: number): ZonesByPage {
  const currentPage = zones[page];
  if (!currentPage?.some((zone) => zone.id === id)) return zones;
  const remaining = currentPage.filter((zone) => zone.id !== id);
  return remaining.length === 0
    ? Object.fromEntries(Object.entries(zones).filter(([key]) => Number(key) !== page))
    : { ...zones, [page]: remaining };
}

export function clearZones(zones: ZonesByPage): ZonesByPage {
  return Object.keys(zones).length === 0 ? zones : {};
}

function updateZone(
  zones: ZonesByPage,
  page: number,
  id: number,
  rect: NormalizedRect,
): ZonesByPage {
  const currentPage = zones[page];
  if (!currentPage?.some((zone) => zone.id === id)) return zones;
  return {
    ...zones,
    [page]: currentPage.map((zone) => (zone.id === id ? { ...zone, rect } : zone)),
  };
}

export function toggleWord(
  selections: SelectionsByPage,
  page: number,
  wordIndex: number,
  text: string,
): SelectionsByPage {
  const currentPage = selections[page] ?? {};
  if (currentPage[wordIndex] !== undefined) {
    const remaining = { ...currentPage };
    delete remaining[wordIndex];
    return Object.keys(remaining).length === 0
      ? Object.fromEntries(Object.entries(selections).filter(([key]) => Number(key) !== page))
      : { ...selections, [page]: remaining };
  }
  return { ...selections, [page]: { ...currentPage, [wordIndex]: text } };
}

export function addWordRange(
  selections: SelectionsByPage,
  page: number,
  first: number,
  last: number,
  words: readonly { index: number; text: string }[],
): SelectionsByPage {
  const lower = Math.min(first, last);
  const upper = Math.max(first, last);
  const additions = Object.fromEntries(
    words
      .filter((word) => word.index >= lower && word.index <= upper)
      .map((word) => [word.index, word.text]),
  );
  return { ...selections, [page]: { ...(selections[page] ?? {}), ...additions } };
}

export function removeWord(
  selections: SelectionsByPage,
  page: number,
  wordIndex: number,
): SelectionsByPage {
  const currentPage = selections[page];
  if (!currentPage || currentPage[wordIndex] === undefined) return selections;
  const remaining = { ...currentPage };
  delete remaining[wordIndex];
  return Object.keys(remaining).length === 0
    ? Object.fromEntries(Object.entries(selections).filter(([key]) => Number(key) !== page))
    : { ...selections, [page]: remaining };
}
