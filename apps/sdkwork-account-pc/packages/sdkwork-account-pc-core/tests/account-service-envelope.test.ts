import { describe, expect, it } from "vitest";
import {
  unwrapSdkworkAccountListPage,
  unwrapSdkworkAccountResponse,
} from "@sdkwork/account-service";

describe("@sdkwork/account-service response envelope helpers", () => {
  it("accepts SDKWork v3 numeric success envelopes", () => {
    expect(unwrapSdkworkAccountResponse<{ ok: true }>({ code: 0, data: { ok: true } })).toEqual({
      ok: true,
    });
  });

  it("rejects non-v3 success code and message fields", () => {
    expect(() => unwrapSdkworkAccountResponse({ code: "0", data: { ok: true } })).toThrow(
      /numeric code 0/u,
    );

    expect(() =>
      unwrapSdkworkAccountResponse(
        {
          code: 40001,
          data: null,
          msg: "non-v3 business failure",
        },
        "Request failed.",
      ),
    ).toThrow("Request failed.");
  });

  it("requires standard list payload shape instead of bare arrays", () => {
    expect(() => unwrapSdkworkAccountListPage([{ id: "entry-1" }])).toThrow(/items and pageInfo/u);
  });
});
