import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({ check: vi.fn(), onUpdateEvent: vi.fn(), status: vi.fn() }));

vi.mock("../../application/updateClient", () => ({
  updateClient: {
    cancel: vi.fn(),
    check: mocks.check,
    onUpdateEvent: mocks.onUpdateEvent,
    restore: vi.fn(),
    start: vi.fn(),
    status: mocks.status,
  },
}));

import { useUpdateStore } from "./useUpdateStore";

describe("update store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    mocks.check.mockReset();
    mocks.onUpdateEvent.mockReset();
    mocks.status.mockReset();
  });

  it("shows an available update after an explicit search", async () => {
    const result = {
      kind: "available",
      version: "1.2.0",
      releases: [
        { version: "1.2.0", notes: "Latest notes" },
        { version: "1.1.0", notes: "Intermediate notes" },
      ],
    };
    mocks.check.mockResolvedValue(result);
    const update = useUpdateStore();

    await update.check("en");

    expect(update.result).toEqual(result);
    expect(update.phase).toBe("idle");
  });

  it("keeps a typed asynchronous update error for localized rendering", async () => {
    mocks.status.mockResolvedValue({
      installedVersion: "1.0.0",
      rollbackAvailable: false,
      updatedTo: null,
      installationError: false,
    });
    mocks.onUpdateEvent.mockImplementation(async (callback) => {
      callback({ type: "failed", error: "updateDownloadFailed" });
      return vi.fn();
    });
    const update = useUpdateStore();

    await update.initialize();

    expect(update.errorCode).toBe("updateDownloadFailed");
  });

  it("keeps a typed command error for localized rendering", async () => {
    mocks.check.mockRejectedValue({ code: "updateCheckFailed" });
    const update = useUpdateStore();

    await update.check("fr");

    expect(update.errorCode).toBe("updateCheckFailed");
  });
});
