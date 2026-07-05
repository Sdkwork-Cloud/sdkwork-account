import { describe, expect, it } from "vitest";
import {
  createWalletCommerceReturnUrl,
  shouldRefreshWalletAfterCommerceReturn,
  stripWalletCommerceReturnParams,
} from "../src/wallet-commerce-return";

describe("wallet-commerce-return", () => {
  it("detects commerce return refresh query params", () => {
    expect(shouldRefreshWalletAfterCommerceReturn(new URLSearchParams("commerceRefresh=1"))).toBe(true);
    expect(shouldRefreshWalletAfterCommerceReturn(new URLSearchParams("payment=success"))).toBe(true);
    expect(shouldRefreshWalletAfterCommerceReturn(new URLSearchParams("orderStatus=paid"))).toBe(true);
    expect(shouldRefreshWalletAfterCommerceReturn(new URLSearchParams("section=holds"))).toBe(false);
  });

  it("strips commerce return params while preserving unrelated query keys", () => {
    const cleaned = stripWalletCommerceReturnParams(
      new URLSearchParams("commerceRefresh=1&section=holds&payment=success"),
    );
    expect(cleaned.toString()).toBe("section=holds");
  });

  it("builds a wallet return URL for payment checkout redirects", () => {
    expect(createWalletCommerceReturnUrl("/wallet", "section=holds")).toBe(
      "/wallet?section=holds&commerceRefresh=1",
    );
  });
});
