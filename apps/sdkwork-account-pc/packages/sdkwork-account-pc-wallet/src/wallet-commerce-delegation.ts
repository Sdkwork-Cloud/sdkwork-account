export type SdkworkWalletCommerceCapability = "order" | "payment";

export type SdkworkWalletCommerceOperation = "recharge" | "withdraw";

export class SdkworkWalletCommerceDelegationError extends Error {
  readonly capability: SdkworkWalletCommerceCapability;

  readonly operation: SdkworkWalletCommerceOperation;

  constructor(
    operation: SdkworkWalletCommerceOperation,
    capability: SdkworkWalletCommerceCapability = "payment",
  ) {
    super(createWalletCommerceDelegationMessage(operation, capability));
    this.name = "SdkworkWalletCommerceDelegationError";
    this.capability = capability;
    this.operation = operation;
  }
}

export function createWalletCommerceDelegationMessage(
  operation: SdkworkWalletCommerceOperation,
  capability: SdkworkWalletCommerceCapability = "payment",
): string {
  if (operation === "recharge") {
    return (
      "Point recharge is owned by sdkwork-order (creates a commerce_order with subject=points_recharge). "
      + "Bootstrap @sdkwork/order-service and call recharges.orders.create via wallet-recharge-service; "
      + "account backend-api adjustments apply after payment succeeds."
    );
  }

  if (capability === "order") {
    return (
      "Cash withdrawal is owned by sdkwork-order/sdkwork-payment payout settlement. "
      + "Use payment payout checkout; account backend-api debits after payout completes."
    );
  }

  return (
    "Cash withdrawal is owned by sdkwork-payment payout settlement. "
    + "Use payment payout checkout; account backend-api debits after payout completes."
  );
}

export function assertWalletCommerceDelegated(
  operation: SdkworkWalletCommerceOperation,
  capability: SdkworkWalletCommerceCapability = "payment",
): never {
  throw new SdkworkWalletCommerceDelegationError(operation, capability);
}
