import type { SdkworkAccountPcRouteContribution } from "@sdkwork/account-pc-core";

export const sdkworkAccountPcWalletRoutes = [
  {
    auth: "required",
    capability: "wallet",
    domain: "commerce",
    id: "app.commerce.wallet.dashboard",
    packageName: "@sdkwork/account-pc-wallet",
    path: "/app/wallet",
    screen: "dashboard",
    surface: "app",
    title: "Wallet",
    titleKey: "wallet.routes.dashboard.title",
  },
] as const satisfies readonly SdkworkAccountPcRouteContribution[];
