import { describe, expect, it } from "vitest";

import { ApplicationError, errorCodeFrom } from "./error";

describe("application errors", () => {
  it("preserves a known IPC error code", () => {
    expect(errorCodeFrom({ code: "destinationMissing" }, "mergeFailed")).toBe("destinationMissing");
  });

  it("replaces unexpected technical errors with the caller fallback", () => {
    expect(errorCodeFrom("The destination folder does not exist.", "splitFailed")).toBe(
      "splitFailed",
    );
    expect(errorCodeFrom(new ApplicationError("redactionFailed"), "unexpected")).toBe(
      "redactionFailed",
    );
  });
});
