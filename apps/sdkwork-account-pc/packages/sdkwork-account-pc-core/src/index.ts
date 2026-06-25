export type SdkworkAccountPcRouteSurface = "app" | "backend-admin";

export interface SdkworkAccountPcRouteContribution {
  readonly auth: "public" | "required";
  readonly capability: string;
  readonly domain: "commerce";
  readonly id: string;
  readonly packageName: string;
  readonly path: string;
  readonly permissionHint?: string;
  readonly screen: string;
  readonly surface: SdkworkAccountPcRouteSurface;
  readonly title: string;
  readonly titleKey: string;
}

export const sdkworkAccountPcRuntimeIdentity = {
  appKey: "sdkwork-account-pc",
  architecture: "pc-react",
  domain: "commerce",
  capability: "account",
  runtimeFamily: "web",
} as const;

export function createSdkworkAccountPcRouteRegistry(
  ...routeGroups: readonly (readonly SdkworkAccountPcRouteContribution[])[]
): readonly SdkworkAccountPcRouteContribution[] {
  return routeGroups.flat();
}
