import type { AuthTokenManager } from "@sdkwork/sdk-common";
import {
  createClient,
  type SdkworkAccountBackendClient as AccountBackendTransportClient,
  type SdkworkBackendConfig,
} from "@sdkwork/account-backend-sdk";
import type { AccountBackendSdkClient } from "@sdkwork/account-sdk-ports";

const BACKEND_API_SUFFIX = "/backend/v3/api";

export function resolveAccountBackendApiOrigin(baseUrl: string): string {
  const trimmed = baseUrl.trim().replace(/\/+$/u, "");
  if (trimmed.endsWith(BACKEND_API_SUFFIX)) {
    return trimmed.slice(0, -BACKEND_API_SUFFIX.length);
  }
  return trimmed;
}

export function createAccountBackendSdkClientFromTransport(
  transport: AccountBackendTransportClient,
): AccountBackendSdkClient {
  return {
    commerce: {
      wallet: transport.wallet,
    },
  } as AccountBackendSdkClient;
}

export interface BootstrapSdkworkAccountBackendServiceInput {
  baseUrl: string;
  authToken?: string;
  accessToken?: string;
  tenantId?: string;
  organizationId?: string;
  platform?: string;
  tokenManager?: AuthTokenManager;
}

export function createAccountBackendTransportClient(
  input: BootstrapSdkworkAccountBackendServiceInput,
): AccountBackendTransportClient {
  const config: SdkworkBackendConfig = {
    baseUrl: resolveAccountBackendApiOrigin(input.baseUrl),
    authToken: input.authToken,
    accessToken: input.accessToken,
    tenantId: input.tenantId,
    organizationId: input.organizationId,
    platform: input.platform,
    tokenManager: input.tokenManager,
  };
  return createClient(config);
}
