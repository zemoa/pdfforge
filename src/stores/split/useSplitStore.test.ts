import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("../../application/splitClient", () => ({ splitClient: {} }));

import { useSplitStore } from "./useSplitStore";

describe("split store preparation", () => {
  beforeEach(() => setActivePinia(createPinia()));

  it("does not remove a source while a split is running", () => {
    const split = useSplitStore();
    split.source = { path: "/documents/source.pdf", name: "source.pdf", pageCount: 3 };
    split.outputName = "result.pdf";
    split.destination = "/documents";
    split.phase = "running";

    split.removeSource();

    expect(split.phase).toBe("running");
    expect(split.source).toEqual({
      path: "/documents/source.pdf",
      name: "source.pdf",
      pageCount: 3,
    });
    expect(split.outputName).toBe("result.pdf");
    expect(split.destination).toBe("/documents");
  });
});
