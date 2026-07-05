const COMMERCE_RETURN_REFRESH_PARAMS = [
  ["commerceRefresh", "1"],
  ["payment", "success"],
  ["paymentStatus", "succeeded"],
  ["orderStatus", "paid"],
  ["orderStatus", "fulfilled"],
] as const;

const COMMERCE_RETURN_REFRESH_KEYS = COMMERCE_RETURN_REFRESH_PARAMS.map(([key]) => key);

export function shouldRefreshWalletAfterCommerceReturn(
  searchParams: URLSearchParams,
): boolean {
  return COMMERCE_RETURN_REFRESH_PARAMS.some(
    ([key, value]) => searchParams.get(key) === value,
  );
}

export function stripWalletCommerceReturnParams(searchParams: URLSearchParams): URLSearchParams {
  const next = new URLSearchParams(searchParams);
  for (const key of COMMERCE_RETURN_REFRESH_KEYS) {
    next.delete(key);
  }
  return next;
}

export function createWalletCommerceReturnUrl(
  walletPath: string,
  existingQuery?: URLSearchParams | string,
): string {
  const normalizedPath = walletPath.trim() || "/wallet";
  const params = new URLSearchParams(
    typeof existingQuery === "string" ? existingQuery : existingQuery?.toString() ?? "",
  );
  params.set("commerceRefresh", "1");
  const query = params.toString();
  return query ? `${normalizedPath}?${query}` : normalizedPath;
}
