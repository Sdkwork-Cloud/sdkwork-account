import type { AuthTokenManager } from "@sdkwork/sdk-common";
import {
  createClient,
  type SdkworkAccountAppClient as AccountAppTransportClient,
  type SdkworkAppConfig,
} from "@sdkwork/account-app-sdk";
import type { AccountAppSdkClient } from "@sdkwork/account-sdk-ports";

const APP_API_SUFFIX = "/app/v3/api";

export function resolveAccountAppApiOrigin(baseUrl: string): string {
  const trimmed = baseUrl.trim().replace(/\/+$/u, "");
  if (trimmed.endsWith(APP_API_SUFFIX)) {
    return trimmed.slice(0, -APP_API_SUFFIX.length);
  }
  return trimmed;
}

export function createAccountAppSdkClientFromTransport(
  transport: AccountAppTransportClient,
): AccountAppSdkClient {
  return {
    commerce: {
      wallet: transport.wallet,
      billing: transport.billing,
      accounts: transport.accounts,
    },
  } as AccountAppSdkClient;
}

export interface BootstrapSdkworkAccountAppServiceInput {
  baseUrl: string;
  authToken?: string;
  accessToken?: string;
  tenantId?: string;
  organizationId?: string;
  platform?: string;
  tokenManager?: AuthTokenManager;
}

export function createAccountAppTransportClient(
  input: BootstrapSdkworkAccountAppServiceInput,
): AccountAppTransportClient {
  const config: SdkworkAppConfig = {
    baseUrl: resolveAccountAppApiOrigin(input.baseUrl),
    authToken: input.authToken,
    accessToken: input.accessToken,
    tenantId: input.tenantId,
    organizationId: input.organizationId,
    platform: input.platform,
    tokenManager: input.tokenManager,
  };
  return createClient(config);
}
