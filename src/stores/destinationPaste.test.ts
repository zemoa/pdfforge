import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("../application/destinationClient", () => ({
  destinationClient: { resolvePastedFolder: vi.fn() },
}));
vi.mock("../application/mergeClient", () => ({ mergeClient: {} }));
vi.mock("../application/redactionClient", () => ({ redactionClient: {} }));
vi.mock("../application/splitClient", () => ({ splitClient: {} }));

import { destinationClient } from "../application/destinationClient";
import { useMergeStore } from "./merge/useMergeStore";
import { useRedactionStore } from "./redaction/useRedactionStore";
import { useSplitStore } from "./split/useSplitStore";

type DestinationStore = {
  destination: string;
  pasteDestination: (pastedText: string) => Promise<void>;
};

function stores(): DestinationStore[] {
  return [useMergeStore(), useSplitStore(), useRedactionStore()];
}

describe("destination paste", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.mocked(destinationClient.resolvePastedFolder).mockReset();
  });

  it("uses the native Windows folder path in every PDF tool", async () => {
    vi.mocked(destinationClient.resolvePastedFolder).mockResolvedValue(
      "C:\\Users\\Olivier\\Downloads",
    );

    for (const store of stores()) {
      await store.pasteDestination("Téléchargements");
      expect(store.destination).toBe("C:\\Users\\Olivier\\Downloads");
    }
  });

  it("keeps the pasted text when no copied folder is available", async () => {
    vi.mocked(destinationClient.resolvePastedFolder).mockResolvedValue(null);

    for (const store of stores()) {
      await store.pasteDestination("C:\\Users\\Olivier\\Downloads");
      expect(store.destination).toBe("C:\\Users\\Olivier\\Downloads");
    }
  });

  it("keeps the pasted text when the native clipboard cannot be read", async () => {
    vi.mocked(destinationClient.resolvePastedFolder).mockRejectedValue(new Error("clipboard busy"));

    for (const store of stores()) {
      await store.pasteDestination("Téléchargements");
      expect(store.destination).toBe("Téléchargements");
    }
  });
});
