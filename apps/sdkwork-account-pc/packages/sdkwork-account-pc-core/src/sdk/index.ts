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
