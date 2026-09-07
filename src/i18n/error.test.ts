import { describe, expect, it } from "vitest";

import { errorCodes } from "../application/error";
import en from "./locales/en";
import fr from "./locales/fr";

describe("error translations", () => {
  it("provides every error code in English and French", () => {
    const expected = [...errorCodes].sort();

    expect(Object.keys(en.errors).sort()).toEqual(expected);
    expect(Object.keys(fr.errors).sort()).toEqual(expected);
  });
});
