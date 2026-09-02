export type SelectionsByPage = Record<number, Record<number, string>>;

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
