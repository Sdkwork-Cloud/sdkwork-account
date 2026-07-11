export type SdkworkWalletCommerceCapability = "order";

export type SdkworkWalletCommerceOperation = "recharge" | "withdraw";

export class SdkworkWalletCommerceDelegationError extends Error {
  readonly capability: SdkworkWalletCommerceCapability;

  readonly operation: SdkworkWalletCommerceOperation;

  constructor(
    operation: SdkworkWalletCommerceOperation,
    capability: SdkworkWalletCommerceCapability = "order",
  ) {
    super(createWalletCommerceDelegationMessage(operation, capability));
    this.name = "SdkworkWalletCommerceDelegationError";
    this.capability = capability;
    this.operation = operation;
  }
}

export function createWalletCommerceDelegationMessage(
  operation: SdkworkWalletCommerceOperation,
  capability: SdkworkWalletCommerceCapability = "order",
): string {
  void capability;

  if (operation === "recharge") {
    return (
      "Account recharge is owned by sdkwork-order account-value flows. "
      + "Inject an order-compatible recharge service and call recharges.orders.create or Token Bank/package/plan/coupon order APIs; "
      + "account backend-api ledger commands apply only after order-owned fulfillment evidence is ready."
    );
  }

  return (
    "Cash withdrawal is owned by sdkwork-order withdrawal request flows. "
    + "Create withdrawals.requests through sdkwork-order; account backend-api holds, settles, or releases cash only when order orchestrates the lifecycle."
  );
}

export function assertWalletCommerceDelegated(
  operation: SdkworkWalletCommerceOperation,
  capability: SdkworkWalletCommerceCapability = "order",
): never {
  throw new SdkworkWalletCommerceDelegationError(operation, capability);
}
