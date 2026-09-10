import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("../../application/mergeClient", () => ({
  mergeClient: {
    inspectSources: vi.fn(),
    previewOutput: vi.fn(),
    start: vi.fn(),
  },
}));

import { mergeClient, type MergeSource, type OutputPreview } from "../../application/mergeClient";
import { useMergeStore } from "./useMergeStore";

const preview: OutputPreview = { outputPath: "/output/merged.pdf", normalizedName: "merged.pdf" };

function source(name: string): MergeSource {
  return { path: `/sources/${name}.pdf`, name: `${name}.pdf`, pageCount: 1, warnings: [] };
}

async function prepare(names = ["a", "b", "c"]) {
  const merge = useMergeStore();
  vi.mocked(mergeClient.inspectSources).mockResolvedValue({
    accepted: names.map(source),
    incidents: [],
    ignoredNonPdfs: [],
    warnings: [],
  });
  await merge.addSelectedPaths(names.map((name) => source(name).path));
  merge.renameOutput("merged.pdf");
  merge.chooseDestination("/output");
  await merge.requestSummary();
  return merge;
}

describe("merge source ordering", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.resetAllMocks();
    vi.mocked(mergeClient.previewOutput).mockResolvedValue(preview);
    vi.mocked(mergeClient.start).mockResolvedValue(undefined);
  });

  it("moves a source to either end while preserving the other documents' order", async () => {
    const merge = await prepare();
    const originalIds = merge.sources.map((entry) => entry.entryId);

    merge.reorderSource(0, 2);
    expect(merge.sources.map((entry) => entry.name)).toEqual(["b.pdf", "c.pdf", "a.pdf"]);
    expect(merge.outputPreview).toBeNull();
    expect(merge.destination).toBe("/output");
    expect(merge.outputName).toBe("merged.pdf");

    merge.reorderSource(2, 0);
    expect(merge.sources.map((entry) => entry.entryId)).toEqual(originalIds);
  });

  it("moves one position up or down through the button intention", async () => {
    const merge = await prepare();
    merge.moveSource(1, -1);
    expect(merge.sources.map((entry) => entry.name)).toEqual(["b.pdf", "a.pdf", "c.pdf"]);
    merge.moveSource(1, 1);
    expect(merge.sources.map((entry) => entry.name)).toEqual(["b.pdf", "c.pdf", "a.pdf"]);
  });

  it.each([
    [-1, 0],
    [3, 0],
    [0, -1],
    [0, 3],
    [0.5, 1],
    [0, 1.5],
    [NaN, 0],
    [0, Infinity],
    [1, 1],
  ])("ignores an invalid or unchanged move from %s to %s", async (from, to) => {
    const merge = await prepare();
    const original = [...merge.sources];
    merge.reorderSource(from, to);
    expect(merge.sources).toEqual(original);
    expect(merge.outputPreview).toEqual(preview);
  });

  it("keeps a valid summary when a button cannot move past an endpoint", async () => {
    const merge = await prepare();
    merge.moveSource(0, -1);
    merge.moveSource(2, 1);
    merge.moveSource(-1, 1);
    merge.moveSource(3, -1);
    expect(merge.sources.map((entry) => entry.name)).toEqual(["a.pdf", "b.pdf", "c.pdf"]);
    expect(merge.outputPreview).toEqual(preview);
  });

  it("keeps duplicate occurrences distinct when moving and removing them", async () => {
    const merge = await prepare(["a", "b", "a"]);
    const [first, middle, duplicate] = merge.sources;
    expect(new Set(merge.sources.map((entry) => entry.entryId)).size).toBe(3);
    merge.reorderSource(2, 0);
    expect(merge.sources).toEqual([duplicate, first, middle]);
    merge.removeSource(1);
    expect(merge.sources).toEqual([duplicate, middle]);
  });

  it("also assigns distinct IDs to sources accepted after an incident", async () => {
    const merge = await prepare(["a", "b"]);
    vi.mocked(mergeClient.inspectSources).mockResolvedValue({
      accepted: [source("a"), source("a")],
      ignoredNonPdfs: [],
      warnings: [],
      incidents: [{ path: "/bad.pdf", name: "bad.pdf", kind: "unreadable" }],
    });
    await merge.addSelectedPaths(["/sources/a.pdf", "/bad.pdf"]);
    merge.ignoreInvalidSources();
    expect(new Set(merge.sources.map((entry) => entry.entryId)).size).toBe(4);
    expect(merge.outputPreview).toBeNull();
  });

  it("requires a fresh summary and sends the chosen order including duplicates to the client", async () => {
    const merge = await prepare(["a", "b", "a"]);
    merge.moveSource(1, -1);
    await merge.confirmMerge();
    expect(mergeClient.start).not.toHaveBeenCalled();

    await merge.requestSummary();
    await merge.confirmMerge();
    expect(mergeClient.start).toHaveBeenCalledExactlyOnceWith(
      ["/sources/b.pdf", "/sources/a.pdf", "/sources/a.pdf"],
      "/output",
      "merged.pdf",
    );
  });

  it("cannot change the confirmed order during processing", async () => {
    const merge = await prepare();
    await merge.confirmMerge();
    const confirmed = [...merge.sources];
    merge.moveSource(0, 1);
    merge.reorderSource(2, 0);
    merge.removeSource(0);
    expect(merge.phase).toBe("running");
    expect(merge.sources).toEqual(confirmed);
  });

  it("discards a summary that arrives after the order changes", async () => {
    const merge = await prepare();
    let resolvePreview!: (value: OutputPreview) => void;
    vi.mocked(mergeClient.previewOutput).mockReturnValue(
      new Promise((resolve) => {
        resolvePreview = resolve;
      }),
    );
    const pendingSummary = merge.requestSummary();
    merge.reorderSource(0, 2);
    resolvePreview(preview);
    expect(await pendingSummary).toBeNull();
    expect(merge.outputPreview).toBeNull();
    await merge.confirmMerge();
    expect(mergeClient.start).not.toHaveBeenCalled();
  });
});
