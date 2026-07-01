import type { AuthTokenManager } from "@sdkwork/sdk-common";
import {
  bootstrapSdkworkAccountAppService,
  bootstrapSdkworkAccountBackendService,
  configureSdkworkAccountSessionTokenProvider,
  type BootstrapSdkworkAccountAppServiceInput,
  type BootstrapSdkworkAccountBackendServiceInput,
  type SdkworkAccountAppService,
  type SdkworkAccountBackendService,
} from "@sdkwork/account-service";
import {
  bootstrapSdkworkOrderAppService,
  configureSdkworkOrderSessionTokenProvider,
  type BootstrapSdkworkOrderAppServiceInput,
  type SdkworkOrderAppService,
} from "@sdkwork/order-service";

export interface BootstrapSdkworkAccountPcSdkInput extends BootstrapSdkworkAccountAppServiceInput {
  tokenManager?: AuthTokenManager;
}

export function bootstrapSdkworkAccountPcSdk(
  input: BootstrapSdkworkAccountPcSdkInput,
): SdkworkAccountAppService {
  configureSdkworkAccountSessionTokenProvider(() => ({
    accessToken: input.accessToken,
    authToken: input.authToken,
  }));
  return bootstrapSdkworkAccountAppService(input);
}

export function bootstrapSdkworkAccountPcBackendSdk(
  input: BootstrapSdkworkAccountBackendServiceInput,
): SdkworkAccountBackendService {
  return bootstrapSdkworkAccountBackendService(input);
}

export interface BootstrapSdkworkAccountPcOrderSdkInput extends BootstrapSdkworkOrderAppServiceInput {
  tokenManager?: AuthTokenManager;
}

export function bootstrapSdkworkAccountPcOrderSdk(
  input: BootstrapSdkworkAccountPcOrderSdkInput,
): SdkworkOrderAppService {
  configureSdkworkOrderSessionTokenProvider(() => ({
    accessToken: input.accessToken,
    authToken: input.authToken,
  }));
  return bootstrapSdkworkOrderAppService(input);
}

export {
  bootstrapSdkworkAccountAppService,
  bootstrapSdkworkAccountBackendService,
  createAccountAppSdkClientFromTransport,
  createAccountAppTransportClient,
  createAccountBackendSdkClientFromTransport,
  createAccountBackendTransportClient,
  resolveAccountAppApiOrigin,
  resolveAccountBackendApiOrigin,
  type BootstrapSdkworkAccountBackendServiceInput,
} from "@sdkwork/account-service";
export {
  bootstrapSdkworkOrderAppService,
  configureSdkworkOrderSessionTokenProvider,
  createOrderAppSdkClientFromTransport,
  createOrderAppTransportClient,
  resolveOrderAppApiOrigin,
  type BootstrapSdkworkOrderAppServiceInput,
  type SdkworkOrderAppService,
} from "@sdkwork/order-service";
