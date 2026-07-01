import { useEffect } from "react";
import { SdkworkThemeProvider } from "@sdkwork/ui-pc-react/theme";
import {
  bootstrapSdkworkAccountPcOrderSdk,
  bootstrapSdkworkAccountPcSdk,
} from "@sdkwork/account-pc-core/sdk";
import { SdkworkWalletPage } from "@sdkwork/account-pc-wallet";

const env = (import.meta as ImportMeta & { env?: Record<string, string | undefined> }).env;

const DEFAULT_ACCOUNT_API_BASE = env?.VITE_SDKWORK_ACCOUNT_API_BASE ?? "http://127.0.0.1:18095";
const DEFAULT_ORDER_API_BASE = env?.VITE_SDKWORK_ORDER_API_BASE ?? "http://127.0.0.1:18079";
const DEFAULT_PAYMENT_CHECKOUT_BASE = env?.VITE_SDKWORK_PAYMENT_CHECKOUT_BASE ?? "/checkout";
const DEFAULT_PAYMENT_PAYOUT_BASE = env?.VITE_SDKWORK_PAYMENT_PAYOUT_BASE ?? "/payments/payout";

function navigateCommerceRoute(route: string): void {
  if (/^https?:\/\//u.test(route)) {
    window.location.assign(route);
    return;
  }

  window.location.assign(route);
}

export function AccountAppShell() {
  useEffect(() => {
    bootstrapSdkworkAccountPcSdk({
      baseUrl: DEFAULT_ACCOUNT_API_BASE,
    });
    bootstrapSdkworkAccountPcOrderSdk({
      baseUrl: DEFAULT_ORDER_API_BASE,
    });
  }, []);

  return (
    <SdkworkThemeProvider defaultTheme="light">
      <SdkworkWalletPage
        checkoutBasePath={DEFAULT_PAYMENT_CHECKOUT_BASE}
        onNavigate={navigateCommerceRoute}
        payoutBasePath={DEFAULT_PAYMENT_PAYOUT_BASE}
        payoutFlow="checkout"
        rechargeFlow="checkout"
      />
    </SdkworkThemeProvider>
  );
}
